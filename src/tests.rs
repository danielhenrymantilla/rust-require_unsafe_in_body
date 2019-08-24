use super::*;
use ::pretty_assertions::assert_eq;

macro_rules! gen_tests {(
    $(
        $test_name:ident:
        stringify! {
            #[$function:ident $(($($attrs:tt)*))?]
            $($input:tt)*
        } == $output:expr;
    )*
) => (
    $(
        #[test]
        fn $test_name ()
        {
            let input: TokenStream =
                stringify!($($input)*)
                    .parse()
                    .expect("Syntax error in test")
            ;
            let output: TokenStream =
                $output
                    .parse()
                    .expect("Syntax error in test")
            ;
            let attrs: TokenStream =
                stringify!($($($attrs)*)?)
                    .parse()
                    .expect("Syntax error in test");
            assert_eq!(
                $function(attrs, input).to_string(),
                output.to_string(),
            )
        }
    )*
)}

gen_tests! {
    identity_for_no_unsafe:
    stringify! {
        #[require_unsafe_in_body]
        #[cold]
        const
        fn add (x: i32, y: i32) -> i32
        {
            x + y
        }
    } == stringify! {
        #[cold]
        const
        fn add (x: i32, y: i32) -> i32
        {
            x + y
        }
    };

    basic_expansion:
    stringify! {
        #[require_unsafe_in_body]
        unsafe
        fn foo () {}
    } == stringify! {
        unsafe
        fn foo ()
        {
            #[inline(always)]
            fn __require_unsafe__inner () {}
            __require_unsafe__inner()
        }
    };
}
