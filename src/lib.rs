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
    ::func_wrap::parse_and_func_wrap_with(input, edit_function_in_place)
        .map_or_else(
            |err| err.to_compile_error(),
            |item| {
                if let Item::Fn(_) = item {
                    item.into_token_stream()
                } else {
                    Error::new(Span::call_site(), "\
                        `#[require_unsafe_in_body]` must be applied to \
                        a function.\
                    ").to_compile_error()
                }
            },
        )
        .into()
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
    ::func_wrap::parse_and_func_wrap_with(input, edit_function_in_place)
        .map_or_else(
            |err| err.to_compile_error(),
            |item| {
                if let Item::Trait(_) | Item::Impl(_) = item {
                    item.into_token_stream()
                } else {
                    Error::new(Span::call_site(), "\
                    `#[require_unsafe_in_bodies]` must be applied to \
                    an `impl` block or a `trait` definition.\
                    ").to_compile_error()
                }
            },
        )
        .into()
}

fn edit_function_in_place (
    func: &'_ mut ImplItemMethod,
    wrapped_func_call: Option<::func_wrap::WrappedFuncCall<'_>>,
) -> Result<()>
{
    let mut wrapped_func_call =
        if let Some(it) = wrapped_func_call {
            it
        } else {
            return Err(Error::new(Span::call_site(), "\
                Missing `#[require_unsafe_in_bodies]` on the enscoping \
                `trait` or `impl` block.
            "));
        }
    ;
    func.block = if func.sig.unsafety.is_none() {
        // If the function is not tagged `unsafe`, there is already no `unsafe`
        // hygiene in the function's body, so there is nothing to do.
        func.sig = wrapped_func_call.sig; // Unrename the function params.
        wrapped_func_call.block // Get back the original function's body.
    } else {
        // Otherwise:
        // 1 - Mark the inner function (the one with the original function's
        //     body) as non-`unsafe`, so that `unsafe { ... }` blocks are
        //     required inside.
        wrapped_func_call.sig.unsafety = None;
        // 2 - In exchange, expect a dummy `unsafe` parameter
        wrapped_func_call.sig.inputs.push(parse_quote!(
            _: r#unsafe
        ));
        wrapped_func_call.call_site_args.push(parse_quote!( r#unsafe ));
        // 3 - Rename it so that recursive calls call the public `unsafe fn`
        //     rather than this internal one.
        let fname = &wrapped_func_call.sig.ident;
        let renamed = ::quote::format_ident!("__unsafe_{}", fname);
        wrapped_func_call.sig.ident = renamed;

        // 3 - Finally, have the public-facing function call that inner one.
        parse_quote!({
            #[allow(nonstandard_style)]
            struct r#unsafe;
            #wrapped_func_call
        })
    };
    Ok(())
}

#[cfg(all(test, feature = "unit-tests"))]
mod tests;
