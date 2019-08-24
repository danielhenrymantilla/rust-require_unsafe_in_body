#[require_unsafe_in_body]
async
unsafe
fn read<T> (p: *const T) -> T
{
    unsafe {
        ::core::ptr::read(p)
    }
}
