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
};
use ::quote::{
    ToTokens,
};
use ::syn::*;

use ::func_wrap::{func_wrap, parse_and_func_wrap_with};

// Reimplement parse_macro_input to use the imported `parse`
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

#[cfg_attr(feature = "nightly",
    doc(include = "docs/require_unsafe_in_body.md")
)]
#[proc_macro_attribute] pub
fn require_unsafe_in_body (
    attrs: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    let _: parse::Nothing = parse_macro_input!(attrs);
    let mut func: ItemFn = parse_macro_input!(input as ItemFn);
    let wrapped_func_call = func_wrap(
        &mut func.sig,
        ::core::mem::replace(&mut func.block, parse_quote!( {} )),
        None,
    );
    let mut wrapped_func_call =
        if let Some(it) = wrapped_func_call {
            it
        } else {
            return Error::new(Span::call_site(), "\
                Missing `#[require_unsafe_in_bodies]` on the enscoping `trait` \
                or `impl` block.
            ").to_compile_error().into();
        }
    ;
    // If the function is not tagged `unsafe`, there is already no `unsafe`
    // hygiene in the function's body, so there is nothing to do.
    *func.block = if func.sig.unsafety.is_none() {
        wrapped_func_call.block
    } else {
        wrapped_func_call.sig.unsafety = None;
        parse_quote!({ #wrapped_func_call })
    };
    func.into_token_stream().into()
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
    let _: parse::Nothing = parse_macro_input!(attrs);
    parse_and_func_wrap_with(input, |func, wrapped_func_call| Ok({
        let mut wrapped_func_call =
            if let Some(it) = wrapped_func_call {
                it
            } else {
                return Err(Error::new(Span::call_site(), "\
                    Missing `#[require_unsafe_in_bodies]` on the enscoping
                    `trait` or `impl` block.\
                "));
            }
        ;
        // if the function is not tagged `unsafe`, there is already no `unsafe`
        // hygiene in the function's body, so there is nothing to do.
        func.block = if func.sig.unsafety.is_none() {
            wrapped_func_call.block
        } else {
            wrapped_func_call.sig.unsafety = None;
            parse_quote!({ #wrapped_func_call })
        };
    })).map_or_else(
        |err| err.to_compile_error(),
        |item| {
            if let Item::Fn(_) | Item::Trait(_) | Item::Impl(_) = item {
                item.into_token_stream()
            } else {
                Error::new(Span::call_site(), "\
                    Expected an `fn` item, a `trait` block, or an `impl` block.
                ").to_compile_error()
            }
        },
    ).into()
}

#[cfg(all(test, feature = "unit-tests"))]
mod tests;
