#[derive(Clone, Copy, Default)]
struct Foo<T> (
    ::core::marker::PhantomData<T>,
);

#[require_unsafe_in_bodies]
pub
trait MyRead {
    unsafe
    fn my_read<U> (self: &'_ Self, p: *const U) -> U
    where
        U : Copy,
    {
        unsafe {
            ::core::ptr::read(p)
        }
    }

    fn unused ();
}

impl<T> MyRead for Foo<T> {
    fn unused ()
    {}
}

#[test]
fn main ()
{
    let x = 42;
    let foo: Foo<()> = Foo::default();
    assert_eq!(
        unsafe { foo.my_read(&x) },
        42,
    );
}
