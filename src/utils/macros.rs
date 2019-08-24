#![allow(unused_macros)]

macro_rules! s {
    (
        $expr:expr $(,)?
    ) => (
        ::std::borrow::Cow::from($expr)
    );

    (
        $($tt:tt)*
    ) => (
        ::std::borrow::Cow::from(format!($($tt)*))
    );
}

macro_rules! error {
    (
        $span:expr =>
        $($tt:tt)*
    ) => ({
        let mut span: Span = $span;
        if format!("{:?}", span).ends_with("bytes(0..0)") {
            span = Span::call_site();
        }
        let message = LitStr::new(&*s!($($tt)*), span);
        TokenStream::from(quote_spanned! { span=>
            compile_error!(#message);
        })
    });

    (
        $($tt:tt)*
    ) => (
        error!(Span::call_site()=>
            $($tt)*
        )
    );
}

macro_rules! unwrap {(
    $expr:expr
) => (
    match $expr {
        | Ok(inner) => inner,
        | Err(error) => return error,
    }
)}

macro_rules! dbg_parse_quote {(
    $($tt:tt)*
) => ({
    eprintln!(
        "[{file}:{line}] {s}",
        s = stringify!($($tt)*),
        file = file!(),
        line = line!(),
    );
    eprintln!("  ==> {}", quote!($($tt)*));
    parse_quote!($($tt)*)
})}

macro_rules! dbg_quote {(
    $($tt:tt)*
) => ({
    let ret = quote!($($tt)*);
    eprintln!("{}", ret);
    ret
})}
