#[macro_use]
extern crate require_unsafe_in_body;

#[derive(Clone, Copy, Default)]
struct Foo<T> (
    ::core::marker::PhantomData<T>,
);

impl<T> Foo<T> {
    #[require_unsafe_in_body]
    pub
    unsafe
    fn my_read<U> (self: &'_ Self, p: *const U) -> U
    where
        U : Copy,
    {
        unsafe {
            ::core::ptr::read(p)
        }
    }
}

fn main ()
{
    let x = 42;
    let foo: Foo<()> = Foo::default();
    assert_eq!(
        unsafe { foo.my_read(&x) },
        42,
    );
}
