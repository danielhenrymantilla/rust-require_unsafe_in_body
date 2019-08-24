_Function_ attribute to require `unsafe` in the function's body no matter the `unsafe`-ness of its API.

# Example

```rust
# use ::require_unsafe_in_body::require_unsafe_in_body;
#
/// Swaps two values of a mutable slice, without checking that the indices are valid.
///
/// # Safety
///
/// The indices must be valid:
///
///   - `i ≠ j`
///
///   - `i < slice.len()`
///
///   - `j < slice.len()`
#[require_unsafe_in_body]
pub
unsafe // narrow contract
fn swap_indices_unchecked<T> (slice: &'_ mut [T], i: usize, j: usize)
{
    let at_i: *mut T = unsafe {
        // Safety: `i < slice.len()`
        debug_assert!(i < slice.len());
        slice.get_unchecked_mut(i)
    };
    let at_j: *mut T = unsafe {
        // Safety: `j < slice.len()`
        debug_assert!(j < slice.len());
        slice.get_unchecked_mut(j)
    };
    unsafe {
        // Safety: `at_i` and `at_j` do not alias since `i ≠ j`
        debug_assert_ne!(i, j);
        ::core::ptr::swap_nonoverlapping(at_i, at_j, 1);
    }
}
```

# Limitation

This is very likely to fail on methods, given technical limitations regarding `Self` and the generated code.

```rust,compile_fail
# use ::require_unsafe_in_body::require_unsafe_in_body;
#
trait CloneInto : Clone {
    #[require_unsafe_in_body]
    unsafe
    fn clone_into (self: &'_ Self, out: *mut Self)
    {
        let clone = self.clone();
        unsafe {
            out.write(clone)
        }
    }
}
```

In that case you need to use `#[`[`require_unsafe_in_bodies`]`]` on the enclosing `impl` or `trait` block:

```rust
# use ::require_unsafe_in_body::require_unsafe_in_bodies;
#
#[require_unsafe_in_bodies]
trait CloneInto : Clone {
    unsafe
    fn clone_into (self: &'_ Self, out: *mut Self)
    {
        let clone = self.clone();
        unsafe {
            out.write(clone)
        }
    }
}
```

___

# How does the macro work? / what does it expand to?

```rust
# use ::require_unsafe_in_body::require_unsafe_in_body;
#
#[require_unsafe_in_body]
unsafe
fn foo (x: i32, y: i32)
{
    // body of foo
}

#[require_unsafe_in_body]
fn bar ()
{
    // body of bar
}
```

expands to:

```rust
unsafe
fn foo (arg_0: i32, arg_1: i32)
{
    #[inline(always)]
    fn inner (x: i32, y: i32)
    {
        // body of foo
    }
    inner(arg_0, arg_1)
}

fn bar ()
{
    // body of bar
}
```

  - **`foo()`**

    The `inner` function can be marked non-`unsafe` since it is private; and by not being marked `unsafe`, the `body of foo` no longer has `unsafe` block hygiene.

  - **`bar()`**

    Since `bar()` is not marked `unsafe`, it already naturally requires `unsafe` in its body, thus not requiring any change.
