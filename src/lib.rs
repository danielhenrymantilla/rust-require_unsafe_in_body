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
};

#[macro_use]
mod utils;

// reimplement parse_macro_input to use the imported `parse`
// function. This way parse_macro_input will parse a TokenStream2 when
// unit-testing.
macro_rules! parse_macro_input {
    (
        $token_stream:ident as $T:ty
    ) => (
        match parse::<$T>($token_stream) {
            | Ok(data) => data,
            | Err(err) => {
                return TokenStream::from(err.to_compile_error());
            }
        }
    );

    (
        $token_stream:ident
    ) => (
        parse_macro_input!($token_stream as _)
    );
}

/// Abstract over common attributes of:
///
///   - a function,
///
///   - an impl method (associated function)
///
///   - a trait's default method (associated function).
struct FunctionLike<'a> {
    attrs: &'a [Attribute],
    sig: &'a mut Signature,
    block: &'a mut Block,
}

/// Metadata when a method is involved
///
///   - `impl ...`
///
///   - a trait's default method (associated function)
#[derive(Clone, Copy)]
struct SelfTy<'a> {
    /// generics from the containing impl / trait block;
    outer_generics: &'a Generics,

    /// self type: _e.g._, `Foo<T>` in `impl<T : Copy> $(... for)? Foo<T>`
    ///
    ///   - in the case of a trait's default method, this will be our added
    ///     `__Self` from the `<__Self : ?Sized + Trait>` type parameter.
    self_ty: &'a Type,

    /// metadata when a trait is involved
    mb_trait_meta: Option<TraitMeta<'a>>,
}

/// metadata when a trait is involved
///
///   - `impl Trait for ...`
///
///   - a trait's default method (associated function)
#[derive(Clone, Copy)]
struct TraitMeta<'a> {
    /// we need to use it as supertraits of our inner scope, so that
    /// `Self::T`-like associated items do not cause lookup errors when
    /// our generated code is compiled by the user.
    mb_supertrait: Option<&'a TypeParamBound>,

    /// `<__Self : ?Sized + Trait>`, when dealing with a trait's default method
    mb_self_ty_param: Option<&'a TypeParam>,
}

/// This is the main function that will handle transforming the body of our
/// "function"
fn edit_function_in_place (
    fn_item: FunctionLike<'_>,
    self_ty: Option<SelfTy<'_>>,
) -> Result<(), TokenStream>
{
    if self_ty.is_none() {
        if utils::is_method(&fn_item) {
            return Err(error!(concat!(
                "`#[require_unsafe_in_body]` does not support directly ",
                "decorating a method; you need to decorate the whole ",
                "`impl` or `trait` block with `#[require_unsafe_in_bodies]`",
            )));
        }
    }
    // if the function is not tagged `unsafe`, there is already no `unsafe`
    // hygiene in the function's body, so there is nothing to do.
    if fn_item.sig.unsafety.is_none() {
        return Ok(());
    }
    // fn_item's body will be replaced, starting with the definition of `inner`,
    // a private non-unsafe function (so that there is no `unsafe` hygiene
    // in it), and then forwarding the args for a call to that function.
    //
    // It turns out that `ImplItemMethod` will allow to be rendered both in a
    // function and method scenario.
    let inner: ImplItemMethod = {
        let mut inner = ImplItemMethod {
            attrs: fn_item.attrs.to_vec(),
            sig: fn_item.sig.clone(),
            block: fn_item.block.clone(),

            vis: Visibility::Inherited,
            defaultness: None,
        };
        inner.sig.ident = parse_quote!(
            __require_unsafe__inner
        );
        // strip unsafe
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
            .map(|_| { let _await = ::syn::token::Await::default(); quote! { . #_await } })
            .unwrap_or_default()
    ;
    // is this a(n associated) function or a true method?
    let mut with_receiver = false;
    // (self, _: __require_unsafe__arg_1, _: __require_unsafe__arg_2, ...)
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
                    with_receiver = true;
                    parse_quote!(self)
                },
                | &mut FnArg::Typed(PatType {
                    ref mut pat,
                    ..
                }) => {
                    if let Pat::Ident(ref pat_ident) = **pat {
                        if pat_ident.ident == "self" {
                            with_receiver = true;
                            return parse_quote!(self);
                        }
                    }
                    **pat = parse_quote!(#arg_name); // Pat
                    parse_quote!(#arg_name) // Expr
                },
            })
            .collect()
    ;
    // also forward the (non-lifetime) function generics, with turbofish
    let mut fn_generics: Generics =
        fn_item
            .sig
            .generics
            .clone()
    ;
    fn_generics.params =
        mem::replace(&mut fn_generics.params, Default::default())
            .into_iter()
            .filter(|generic| match generic {
                | &GenericParam::Lifetime(_) => false,
                | &GenericParam::Type(_) => true,
                // not very sure about this one...
                | &GenericParam::Const(_) => true,
            })
            .collect()
    ;
    let (_, fn_generics, _) = fn_generics.split_for_impl();
    let fn_generics: Turbofish = fn_generics.as_turbofish();

    // Finally, rewrite the function's body
    *fn_item.block = match self_ty {
        // Classic function
        | None => {
            parse_quote!({
                // inner function definition
                #inner

                // forwarded call
                __require_unsafe__inner #fn_generics (
                    #(#args),*
                ) #_await
            })
        },

        // Method
        | Some(SelfTy {
            outer_generics,
            self_ty,
            mb_trait_meta,
        }) => {
            let (
                trait_generics,
                ty_generics,
                where_clause,
            ) = {
                outer_generics
                    .split_for_impl()
            };
            let ref inner_sig = inner.sig;
            let mut supertraits_bound = TokenStream2::new();
            let mut generics;
            let impl_generics;
            let impl_generics: &ImplGenerics =
                if let Some(trait_meta) = mb_trait_meta {
                    let TraitMeta {
                        mb_supertrait,
                        mb_self_ty_param,
                    } = trait_meta;
                    if let Some(supertrait) = mb_supertrait {
                        supertraits_bound = quote! {
                            : #supertrait
                        };
                    }
                    if let Some(self_ty_param) = mb_self_ty_param {
                        generics = outer_generics.clone();
                        generics.params.push(
                            self_ty_param.clone().into()
                        );
                        impl_generics = generics.split_for_impl().0;
                        &impl_generics
                    } else {
                        &trait_generics
                    }
                } else {
                    &trait_generics
                }
            ;
            parse_quote!({
                trait __require_unsafe__trait #trait_generics
                #supertraits_bound
                {
                    #inner_sig;
                }
                impl #impl_generics __require_unsafe__trait #ty_generics
                    for #self_ty
                #where_clause
                {
                    #inner
                }

                <Self as __require_unsafe__trait #ty_generics>
                    ::__require_unsafe__inner #fn_generics (
                        #(#args),*
                    ) #_await
            })
        },
    };
    Ok(())
}

// This function will "just" call `edit_function_in_place` on the fed
// function defintion, without any method metadata.
#[cfg_attr(feature = "nightly",
    doc(include = "docs/require_unsafe_in_body.md")
)]
#[proc_macro_attribute] pub
fn require_unsafe_in_body (
    attrs: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    // Try to give useful error messages
    let attrs = TokenStream2::from(attrs);
    if attrs.is_empty().not() {
        return error!(attrs.span()=>
            "Unexpected parameter(s)"
        );
    }
    let mut item_fn: ItemFn = match parse(input.clone()) {
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

    // if the function is not tagged `unsafe`,
    // return the input TokenStream as is.
    if item_fn.sig.unsafety.is_none() {
        return input;
    }
    let ItemFn {
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
        edit_function_in_place(function_like, None)
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
    // Try to give useful error messages
    let attrs = TokenStream2::from(attrs);
    if attrs.is_empty().not() {
        return error!(attrs.span()=>
            "Unexpected parameter(s)"
        );
    }

    // 2 possibilities: `impl` block, or `trait` block (default methods)
    match parse_macro_input!(input as Either<ItemImpl, ItemTrait>) {
        // `impl` block
        | Either::Left(mut item_impl) => {
            let type_param_bound;
            // extract trait metadata:
            //
            //   - no need for a __Self type parameter,
            //
            //   - the trait being implemented will be the supertrait of our
            //     inner helper trait.
            let mb_trait_meta: Option<TraitMeta> =
                if let Some((_, ref trait_, _)) = item_impl.trait_ {
                    Some(TraitMeta {
                        mb_supertrait: Some({
                            type_param_bound = parse_quote! {
                                #trait_
                            };
                            &type_param_bound
                        }),
                        mb_self_ty_param: None,
                    })
                } else {
                    None
                }
            ;
            let ItemImpl {
                ref generics,
                ref self_ty,
                ref mut items,
                ..
            } = item_impl;
            let self_ty_params = SelfTy {
                outer_generics: generics,
                self_ty,
                mb_trait_meta,
            };
            unwrap!(
                items
                    .iter_mut()
                    .try_for_each(|impl_item| Ok(match impl_item {
                        | &mut ImplItem::Method(ImplItemMethod {
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
                            edit_function_in_place(
                                function_like,
                                Some(self_ty_params),
                            )?;
                        },
                        | _ => {},
                    }))
            );
            TokenStream::from(quote! {
                #item_impl
            })
        },

        // `trait` block
        | Either::Right(mut item_trait) => {
            let ItemTrait {
                ref generics,
                ref mut items,
                ..
            } = item_trait;
            // extract trait metadata:
            //
            //   - no need for a __Self type parameter,
            //
            //   - the trait being implemented will be the supertrait of our
            let ref self_ty: Box<Type> = parse_quote! {
                // use a less mangled name for more readable lifetime bounds
                // errors
                /* __require_unsafe */ __Self
            };
            let ref supertrait: TypeParamBound = {
                let trait_name = &item_trait.ident;
                let (_, ty_generics, _) = generics.split_for_impl();
                parse_quote! {
                    #trait_name #ty_generics
                }
            };
            let ref self_ty_param: TypeParam = {
                let mut self_ty_param: TypeParam = parse_quote! {
                    #self_ty : ?Sized
                };
                self_ty_param.bounds.push(supertrait.clone());
                self_ty_param
            };
            let trait_meta = TraitMeta {
                mb_self_ty_param: Some(self_ty_param),
                mb_supertrait: Some(supertrait),
            };
            let self_ty_params = SelfTy {
                outer_generics: generics,
                self_ty,
                mb_trait_meta: Some(trait_meta),
            };

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
                    .try_for_each(|function| {
                        edit_function_in_place(
                            function,
                            Some(self_ty_params),
                        )
                    })
            );
            TokenStream::from(quote! {
                #item_trait
            })
        },
    }
}

#[cfg(all(test, feature = "unit-tests"))]
mod tests;
