#[macro_use]
extern crate require_unsafe_in_body;

/// Swaps two values of a mutable slice, without checking that the indices are
/// valid.
#[require_unsafe_in_body]
pub
unsafe // narrow contract
fn swap_indices_unchecked<T> (slice: &'_ mut [T], i: usize, j: usize)
{
    let at_i: *mut T = slice.get_unchecked_mut(i);
    let at_j: *mut T = slice.get_unchecked_mut(j);
    ::core::ptr::swap_nonoverlapping(at_i, at_j, 1);
}

fn main ()
{}
