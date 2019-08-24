use super::*;

#[macro_use]
mod macros;

pub(in super)
enum Either<Left : Parse, Right : Parse> {
    Left(Left),
    Right(Right),
}

impl<Left : Parse, Right : Parse> Parse for Either<Left, Right> {
    fn parse (input: ParseStream<'_>) -> syn::Result<Self>
    {
        let ref fork = input.fork();
        Ok(if let Ok(left) = fork.parse() {
            use ::syn::parse::discouraged::Speculative;
            input.advance_to(fork);
            Either::Left(left)
        } else {
            Either::Right(input.parse()?)
        })
    }
}

pub(in super)
fn is_receiver (fn_arg: &'_ FnArg) -> bool
{
    match fn_arg {
        | &FnArg::Receiver(_) => true,
        | &FnArg::Typed(PatType {
            ref pat,
            ..
        }) => {
            match **pat {
                | Pat::Ident(ref pat_ident)
                    if pat_ident.ident == "self"
                => {
                    true
                },
                | _ => {
                    false
                },
            }
        },
    }
}

pub(in super)
fn is_method (fn_item: &'_ FunctionLike<'_>) -> bool
{
    fn_item
        .sig
        .inputs
        .iter()
        .next()
        .map(is_receiver)
        .unwrap_or(false)
}
