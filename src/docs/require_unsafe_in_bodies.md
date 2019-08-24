_`impl` or `trait` block_ attribute to require `unsafe` in the associated functions' bodies no matter the `unsafe`-ness of their APIs.

# Example

```rust
# use ::require_unsafe_in_body::require_unsafe_in_bodies;
#
pub
trait SwapKeys<Key>
where
    Key : Copy + Eq,
{
    fn contains_key (self: &'_ Self, key: Key) -> bool;

    /// Swaps two values within `self`, without checking that the keys are valid.
    ///
    /// # Safety
    ///
    /// The keys must be valid:
    ///
    ///   - `self.contains_key(k1)`,
    ///
    ///   - `self.contains_key(k2)`,
    ///
    ///   - `k1 ≠ k2`
    unsafe
    fn swap_keys_unchecked (self: &'_ mut Self, k1: Key, k2: Key);
}

#[require_unsafe_in_bodies]
impl<T> SwapKeys<usize> for [T] {
    #[inline]
    fn contains_key (self: &'_ Self, idx: usize) -> bool
    {
        idx < self.len()
    }

    #[inline]
    unsafe
    fn swap_keys_unchecked (self: &'_ mut Self, i: usize, j: usize)
    {
        let at_i: *mut T = unsafe {
            // Safety: `i < self.len()`
            debug_assert!(i < self.len());
            self.get_unchecked_mut(i)
        };
        let at_j: *mut T = unsafe {
            // Safety: `j < self.len()`
            debug_assert!(j < self.len());
            self.get_unchecked_mut(j)
        };
        unsafe {
            // Safety: `at_i` and `at_j` do not alias since `i ≠ j`
            debug_assert_ne!(i, j);
            ::core::ptr::swap_nonoverlapping(at_i, at_j, 1);
        }
    }
}
```

___

# How does the macro work? / what does it expand to?

```rust
# use ::require_unsafe_in_body::require_unsafe_in_bodies;
#
struct Foo;

#[require_unsafe_in_bodies]
impl Foo {
    unsafe
    fn foo (&self, x: i32, _: i32)
    {
        // body of foo
    }

    fn bar (&self)
    {
        // body of bar
    }
}
```

expands to:

```rust
struct Foo;

impl Foo {
    unsafe
    fn foo (&self, arg_1: i32, arg_2: i32)
    {
        trait Helper {
            fn inner (&self, x: i32, _: i32);
        }
        impl Helper for Foo {
            #[inline(always)]
            fn inner (&self, x: i32, _: i32)
            {
                // body of foo
            }
        }
        <Self as Helper>::inner(self, arg_1, arg_2)
    }

    fn bar (&self)
    {
        // body of bar
    }
}
```

  - **`foo()`**

    The `inner` function can be marked non-`unsafe` since it is private; and by not being marked `unsafe`, the `body of foo` no longer has `unsafe` block hygiene.

  - **`bar()`**

    Since `bar()` is not marked `unsafe`, it already naturally requires `unsafe` in its body, thus not requiring any change.
