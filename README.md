# `#[require_unsafe_in_body]`

A procedural macro attribute to make an `unsafe fn` still require `unsafe` blocks in its body.

[![Latest version](https://img.shields.io/crates/v/require_unsafe_in_body.svg)](https://crates.io/crates/require_unsafe_in_body)
[![Documentation](https://docs.rs/require_unsafe_in_body/badge.svg)](https://docs.rs/require_unsafe_in_body)
![License](https://img.shields.io/crates/l/require_unsafe_in_body.svg)

## Motivation

Imagine having a function with a _narrow contract_, _i.e._, a function that can
trigger Undefined Behavior in some situations (incorrect inputs or call
context).
Rust safety design requires that this function be annotated `unsafe` if it is
part of a public API, and even when it is not, it is _highly advisable_ to do
so (code will be easier to maintain):

```rust
/// Swaps two values of a mutable slice, without checking that the indices are valid.
pub
unsafe // narrow contract
fn swap_indices_unchecked<T> (slice: &'_ mut [T], i: usize, j: usize)
{
    let at_i: *mut T = slice.get_unchecked_mut(i);
    let at_j: *mut T = slice.get_unchecked_mut(j);
    ::core::ptr::swap_nonoverlapping(at_i, at_j, 1)
}
```

As you can see, when a function is annotated `unsafe`, the body of the function
no longer requires special `unsafe` blocks around the most subtle things.
For instance, in this case, it may not be obvious that there are _two_ distinct
`unsafe` things happening:

  - we are performing unchecked indexing, which would break if `i ≥ len` or
    `j ≥ len`;

  - we are asserting that `at_i` and `at_j` do not alias, which would break if
    `i = j`.

Since misusing any of these invariants is wildly unsound, it would be better
if we could explicit _where_ each invariant is or may be used:

```rust
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
#[allow(unused_unsafe)]
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

Sadly, since these `unsafe` blocks are not required, not only do they trigger
`unused_unsafe` warnings, they can also be mistakenly missed without Rust
complaining whatsoever.

That's what `#[`[`require_unsafe_in_body`]`]` solves:

> **`#[`[`require_unsafe_in_body`]`]` "automagically removes" the intrinsic `unsafe`-ness (hygiene) of an `unsafe fn` body, thus making it _necessary_ to use `unsafe` scopes inside.**

## Example

The code

```rust,compile_fail
#[macro_use]
extern crate require_unsafe_in_body;

/// Swaps two values of a mutable slice, without checking that the indices are valid.
#[require_unsafe_in_body]
pub
unsafe // narrow contract
fn swap_indices_unchecked<T> (slice: &'_ mut [T], i: usize, j: usize)
{
    let at_i: *mut T = slice.get_unchecked_mut(i);
    let at_j: *mut T = slice.get_unchecked_mut(j);
    ::core::ptr::swap_nonoverlapping(at_i, at_j, 1);
}
```

yields the following compiler error:

```text
error[E0133]: call to unsafe function is unsafe and requires unsafe function or block
  --> example.rs:11:24
   |
11 |     let at_i: *mut T = slice.get_unchecked_mut(i);
   |                        ^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior

error[E0133]: call to unsafe function is unsafe and requires unsafe function or block
  --> example.rs:12:24
   |
12 |     let at_j: *mut T = slice.get_unchecked_mut(j);
   |                        ^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior

error[E0133]: call to unsafe function is unsafe and requires unsafe function or block
  --> example.rs:13:5
   |
13 |     ::core::ptr::swap_nonoverlapping(at_i, at_j, 1);
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior

For more information about this error, try `rustc --explain E0133`.
```

## Usage

 1. Add this to your `Cargo.toml`:

    ```toml
    [dependencies]
    require_unsafe_in_body = "0.1.1"
    ```

 2. Add this to your `lib.rs` (or `main.rs`):

    ```rust,ignore
    #[macro_use]
    extern crate require_unsafe_in_body;
    ```

 3. You can then decorate:

      - functions, with the `#[`[`require_unsafe_in_body`]`]` attribute;

      - methods, with the `#[`[`require_unsafe_in_bodies`]`]` attribute **applied to the enclosing `impl` or `trait` block**.

[`require_unsafe_in_bodies`]: https://docs.rs/require_unsafe_in_body/0.1.1/require_unsafe_in_body/attr.require_unsafe_in_body.html
