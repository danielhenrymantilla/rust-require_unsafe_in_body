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
            #[allow(nonstandard_style)]
            struct r#unsafe;

            ({
                #[inline(always)]
                fn __unsafe_foo (_: r#unsafe) {}
                __unsafe_foo
            })(r#unsafe)
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
                x: T,
                y: U,
                arg: Self::Arg,
            )
            {
                #[allow(nonstandard_style)]
                struct r#unsafe;
                ({
                    trait __FuncWrap<T> : Foo<T> {
                        #[inline(always)]
                        fn __unsafe_foo<U> (
                            self: &'_ Self,
                            x: T,
                            y: U,
                            arg: Self::Arg,
                            _: r#unsafe
                        )
                        {}
                    }

                    impl<T, __Self : ?Sized + Foo<T> >
                        __FuncWrap<T>
                        for __Self
                    {}

                    <Self as __FuncWrap<T> >::__unsafe_foo::<U>
                })(self,
                    x,
                    y,
                    arg,
                    r#unsafe)
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
            fn make<K> (arg_0: Self::U) -> Self
            {
                #[allow(nonstandard_style)]
                struct r#unsafe;

                ({
                    trait __FuncWrap<T : Copy> : Trait {
                        fn __unsafe_make<K> (_: Self::U, _: r#unsafe) -> Self;
                    }

                    impl<T : Copy> __FuncWrap<T> for Foo<T> {
                        #[inline(always)]
                        fn __unsafe_make<K> (_: Self::U, _: r#unsafe) -> Self
                        {
                            unsafe {
                                ::core::ptr::read(::core::ptr::null::<Self>())
                            }
                        }

                    }
                    <Self as __FuncWrap<T> >::__unsafe_make::<K>
                })(
                    arg_0, r#unsafe
                )
            }
        }
    };
}
