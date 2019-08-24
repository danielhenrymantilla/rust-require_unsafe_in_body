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
            let ret = $function(attrs, input).to_string();
            eprintln!("{}", ret);
            assert_eq!(
                ret,
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

    basic_require_unsafe_in_bodies_default_method:
    stringify! {
        #[require_unsafe_in_bodies]
        trait Foo<T> {
            type Arg;

            unsafe
            fn foo<U> (
                self: &'_ Self,
                x: T,
                y: U,
                arg: Self::Arg,
            )
            {}
        }
    } == stringify! {
        trait Foo<T> {
            type Arg;

            unsafe
            fn foo<U> (
                self: &'_ Self,
                __require_unsafe__arg_1: T,
                __require_unsafe__arg_2: U,
                __require_unsafe__arg_3: Self::Arg,
            )
            {
                trait __require_unsafe__trait<T> : Foo<T> {
                    fn __require_unsafe__inner<U> (
                        self: &'_ Self,
                        x: T,
                        y: U,
                        arg: Self::Arg,
                    );
                }

                impl<T, __Self : ?Sized + Foo<T> >
                    __require_unsafe__trait<T>
                    for __Self
                {

                    #[inline(always)]
                    fn __require_unsafe__inner<U> (
                        self: &'_ Self,
                        x: T,
                        y: U,
                        arg: Self::Arg,
                    )
                    {}
                }

                <Self as __require_unsafe__trait<T> >
                    ::__require_unsafe__inner::<U>(
                        self,
                        __require_unsafe__arg_1,
                        __require_unsafe__arg_2,
                        __require_unsafe__arg_3
                    )
            }
        }
    };

    basic_require_unsafe_in_bodies_impl_trait:
    stringify! {
        #[require_unsafe_in_bodies]
        impl<T : Copy> Trait for Foo<T> {
            type U = i32;

            unsafe
            fn make<K> (_: Self::U) -> Self
            {
                unsafe {
                    ::core::ptr::read(::core::ptr::null::<Self>())
                }
            }
        }
    } == stringify! {
        impl<T : Copy> Trait for Foo<T> {
            type U = i32;

            unsafe
            fn make<K> (__require_unsafe__arg_0: Self::U) -> Self
            {
                trait __require_unsafe__trait<T : Copy> : Trait {
                    fn __require_unsafe__inner<K> (_: Self::U) -> Self;
                }

                impl<T : Copy> __require_unsafe__trait<T> for Foo<T> {
                    #[inline(always)]
                    fn __require_unsafe__inner<K> (_: Self::U) -> Self
                    {
                        unsafe {
                            ::core::ptr::read(::core::ptr::null::<Self>())
                        }
                    }

                }
                <Self as __require_unsafe__trait<T> >::
                __require_unsafe__inner::<K>(
                    __require_unsafe__arg_0
                )
            }
        }
    };
}
