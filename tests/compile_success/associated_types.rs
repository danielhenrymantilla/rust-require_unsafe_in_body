#[require_unsafe_in_bodies]
trait Trait<X> {
    type Arg;
    type Ret;

    unsafe
    fn foo (self: &'_ Self, x: X, arg: Self::Arg)
        -> Self::Ret
    {
        unsafe {
            ::core::ptr::read(::core::ptr::null())
        }
    }

    unsafe
    fn bar (this: &'_ Self, x: X, arg: Self::Arg)
        -> Self::Ret
    {
        unsafe {
            ::core::ptr::read(::core::ptr::null())
        }
    }

    unsafe
    fn baz (x: X, arg: Self::Arg)
        -> Self::Ret
    {
        unsafe {
            ::core::ptr::read(::core::ptr::null())
        }
    }
}
