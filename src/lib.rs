#![doc(test(attr(
    deny(warnings),
    allow(unused),
    deny(unused_unsafe),
)))]
#![cfg_attr(feature = "nightly",
    feature(external_doc),
)]
#![cfg_attr(feature = "nightly",
    doc(include = "../README.md")
)]

#[cfg(not(all(test, feature = "unit-tests")))]
extern crate proc_macro;
#[cfg(not(all(test, feature = "unit-tests")))]
use ::proc_macro::TokenStream;
#[cfg(not(all(test, feature = "unit-tests")))]
use ::syn::parse;
#[cfg(all(test, feature = "unit-tests"))]
use ::proc_macro2::TokenStream;
#[cfg(all(test, feature = "unit-tests"))]
use ::syn::parse2 as parse;

macro_rules! parse_macro_input {
    (
        $tokenstream:ident as $ty:ty
    ) => (
        match parse::<$ty>($tokenstream) {
            Ok(data) => data,
            Err(err) => {
                return TokenStream::from(err.to_compile_error());
            }
        }
    );

    (
        $tokenstream:ident
    ) => (
        parse_macro_input!($tokenstream as _)
    );
}

use ::proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use ::quote::{
    quote,
    quote_spanned,
};
use ::syn::{*,
    parse::{
        Parse,
        ParseStream,
    },
    spanned::{
        Spanned,
    },
};
use ::std::{*,
    ops::Not,
    result::Result::{self, *},
};

use self::utils::{
    Either,
    is_method,
};

#[macro_use]
mod utils;

struct FunctionLike<'a> {
    attrs: &'a Vec<Attribute>,
    sig: &'a mut Signature,
    block: &'a mut Block,
}

fn edit_method_in_place (
    fn_item: FunctionLike<'_>,
    self_ty: Option<(&'_ Generics, &'_ Box<Type>)>,
) -> Result<(), TokenStream>
{
    if self_ty.is_none() {
        if let Some(_) = is_method(&fn_item) {
            return Err(error!(concat!(
                "`#[require_unsafe_in_body]` does not support directly ",
                "decorating a method; you need to decorate the whole ",
                "`impl` or `trait` block with `#[require_unsafe_in_bodies]`",
            )));
        }
    }
    if fn_item.sig.unsafety.is_none() {
        return Ok(());
    }
    let inner: ImplItemMethod = {
        let mut inner = ImplItemMethod {
            attrs: fn_item.attrs.clone(),
            sig: fn_item.sig.clone(),
            block: fn_item.block.clone(),

            vis: Visibility::Inherited,
            defaultness: None,
        };
        inner.sig.ident = parse_quote!(
            __require_unsafe__inner
        );
        // strip unsafe <= This is the point of the whole macro
        inner.sig.unsafety = None;
        // strip extern
        inner.sig.abi = None;
        // strip (some) special attributes
        inner
            .attrs
            .retain(|&Attribute { ref path, ..}| bool::not(
                path.is_ident("inline") ||
                path.is_ident("cold") ||
                path.is_ident("no_mangle")
            ))
        ;
        inner.attrs.push(parse_quote! {
            #[inline(always)]
        });
        inner
    };
    // Append an `.await` to the forwarded call if the function is `async`
    let _await =
        inner
            .sig
            .asyncness
            .map(|_| quote!( .await ))
            .unwrap_or_default()
    ;
    let args: Vec<Expr> =
        (0 ..)
            .map(|i| {
                let s: &str = &*format!("__require_unsafe__arg_{}", i);
                Ident::new(s, Span::call_site())
            })
            .zip(fn_item.sig.inputs.iter_mut())
            // replace input parameter patterns by the numbered `arg_name`s,
            // unless the parameter is `self`,
            // while outputing these numbered `arg_name`s.
            .map(|(arg_name, at_fn_arg)| match at_fn_arg {
                | &mut FnArg::Receiver(_) => {
                    parse_quote!(self)
                },
                | &mut FnArg::Typed(PatType {
                    ref mut pat,
                    ..
                }) => {
                    if let Pat::Ident(ref pat_ident) = **pat {
                        if pat_ident.ident == "self" {
                            return parse_quote!(self);
                        }
                    }
                    **pat = parse_quote!(#arg_name);
                    parse_quote!(#arg_name)
                },
            })
            .collect()
    ;
    *fn_item.block = match self_ty {
        | Some((generics, self_ty)) => {
            let (
                impl_generics,
                _ty_generics,
                where_clause,
            ) = generics.split_for_impl();
            let inner_sig = &inner.sig;
            parse_quote!({
                trait __require_unsafe__trait {
                    #inner_sig ;
                }
                impl #impl_generics __require_unsafe__trait
                    for #self_ty
                #where_clause
                {
                    #inner
                }

                <Self as __require_unsafe__trait>::__require_unsafe__inner(
                    #(#args),*
                ) #_await
            })
        },

        | _ => {
            parse_quote!({
                #inner

                __require_unsafe__inner(
                    #(#args),*
                ) #_await
            })
        },
    };
    Ok(())
}

#[cfg_attr(feature = "nightly",
    doc(include = "docs/require_unsafe_in_body.md")
)]
#[proc_macro_attribute] pub
fn require_unsafe_in_body (
    attrs: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    let attrs = TokenStream2::from(attrs);
    if attrs.is_empty().not() {
        return error!(attrs.span()=>
            "Unexpected parameter(s)"
        );
    }
    let mut item_fn: ImplItemMethod = match parse(input.clone()) {
        | Ok(item_fn) => item_fn,
        | Err(error) => {
            if  parse::<ItemImpl>(input.clone()).is_ok() ||
                parse::<ItemTrait>(input).is_ok()
            {
                return error!(
                    concat!(
                        "To decorate an `impl` or `trait` block, ",
                        "you need to use `#[require_unsafe_in_bodies]`",
                    ),
                );
            } else {
                return error.to_compile_error().into();
            }
        },
    };
    if item_fn.sig.unsafety.is_none() {
        return input;
    }
    let ImplItemMethod {
        ref attrs,
        ref mut sig,
        ref mut block,
        ..
    } = item_fn;
    let function_like = FunctionLike {
        attrs,
        sig,
        block,
    };
    unwrap!(
        edit_method_in_place(function_like, None)
    );
    TokenStream::from(quote! {
        #item_fn
    })
}

#[cfg_attr(feature = "nightly",
    doc(include = "docs/require_unsafe_in_bodies.md")
)]
#[proc_macro_attribute] pub
fn require_unsafe_in_bodies (
    attrs: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    let attrs = TokenStream2::from(attrs);
    if attrs.is_empty().not() {
        return error!(attrs.span()=>
            "Unexpected parameter(s)"
        );
    }
    match parse_macro_input!(input as Either<ItemImpl, ItemTrait>) {
        | Either::Left(mut item_impl) => {
            let ItemImpl {
                ref generics,
                ref self_ty,
                ref mut items,
                ..
            } = item_impl;
            unwrap!(
                items
                    .iter_mut()
                    .try_for_each(|impl_item| Ok(match *impl_item {
                        | ImplItem::Method(ImplItemMethod {
                            ref attrs,
                            ref mut sig,
                            ref mut block,
                            ..
                        }) => {
                            let function_like = FunctionLike {
                                attrs,
                                sig,
                                block,
                            };
                            edit_method_in_place(
                                function_like,
                                Some((generics, self_ty)),
                            )?;
                        },
                        | _ => {},
                    }))
            );
            TokenStream::from(quote! {
                #item_impl
            })
        },

        | Either::Right(mut item_trait) => {
            let ItemTrait {
                ref generics,
                ref mut items,
                ..
            } = item_trait;
            let ref self_ty: Box<Type> = parse_quote! {
                __require_unsafe__Self
            };
            let ref mut generics = generics.clone();
            generics.params.push({
                let mut type_param: TypeParam = parse_quote! {
                    #self_ty : ?Sized
                };
                type_param.bounds.extend(
                    item_trait
                        .supertraits
                        .iter()
                        .cloned()
                );
                type_param.into()
            });

            unwrap!(
                items
                    .iter_mut()
                    .filter_map(|trait_item| match *trait_item {
                        | TraitItem::Method(TraitItemMethod {
                            ref attrs,
                            ref mut sig,
                            default: Some(ref mut block),
                            ..
                        }) => {
                            Some(FunctionLike {
                                attrs,
                                sig,
                                block,
                            })
                        },

                        | _ => None,
                    })
                    .try_for_each(|method|
                        edit_method_in_place(
                            method,
                            Some((generics, self_ty)),
                        )
                    )
            );
            TokenStream::from(quote! {
                #item_trait
            })
        },
    }
}

#[cfg(all(test, feature = "unit-tests"))]
mod tests;
