trait Bound1 {}
trait Bound2 {}

struct Foo<'lt1, T : ?Sized + Bound1 + 'lt1> (&'lt1 T);

trait Bar<'lt2> : Bound2 {
    unsafe
    fn read<U : ?Sized> (self: &'_ Self, p: *const *const U) -> *const U;
}

#[require_unsafe_in_bodies]
impl<'lt1, 'lt2, T : ?Sized + Bound1 + 'lt1> Bar<'lt2>
    for Foo<'lt1, T>
where
    for<'a> Foo<'a, T> : Bound2,
{
    unsafe
    fn read<U : ?Sized> (self: &'_ Self, p: *const *const U) -> *const U
    {
        unsafe {
            ::core::ptr::read(p)
        }
    }
}
