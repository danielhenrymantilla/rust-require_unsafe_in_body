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
fn is_method (fn_item: &'_ FunctionLike<'_>) -> Option<Span>
{
    let first_arg =
        fn_item
            .sig
            .inputs
            .iter()
            .next()
    ;
    #[allow(bad_style)]
    let Self_: Type = parse_quote!( Self );
    match first_arg {
        | Some(&FnArg::Receiver(ref receiver)) => {
            return Some(receiver.self_token.span());
        },
        | Some(&FnArg::Typed(PatType {
            ref pat,
            ref ty,
            ..
        })) => {
            if **ty == Self_ {
                return Some(ty.span());
            }
            if let Pat::Ident(ref pat_ident) = **pat {
                if pat_ident.ident == "self" {
                    return Some(pat_ident.ident.span());
                }
            }
        },
        | _ => {},
    }
    None
}
