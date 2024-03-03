// Copyright 2022 The Fuchsia Authors
//
// Licensed under a BSD-style license <LICENSE-BSD>, Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0>, or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your option.
// This file may not be copied, modified, or distributed except according to
// those terms.

//! Utilities used by macros and by `zerocopy-derive`.
//!
//! These are defined here `zerocopy` rather than in code generated by macros or
//! by `zerocopy-derive` so that they can be compiled once rather than
//! recompiled for every invocation (e.g., if they were defined in generated
//! code, then deriving `IntoBytes` and `FromBytes` on three different types
//! would result in the code in question being emitted and compiled six
//! different times).

#![allow(missing_debug_implementations)]

use core::{marker::PhantomData, mem::ManuallyDrop};

// TODO(#29), TODO(https://github.com/rust-lang/rust/issues/69835): Remove this
// `cfg` when `size_of_val_raw` is stabilized.
#[cfg(__INTERNAL_USE_ONLY_NIGHLTY_FEATURES_IN_TESTS)]
use core::ptr::{self, NonNull};

/// A compile-time check that should be one particular value.
pub trait ShouldBe<const VALUE: bool> {}

/// A struct for checking whether `T` contains padding.
pub struct HasPadding<T: ?Sized, const VALUE: bool>(PhantomData<T>);

impl<T: ?Sized, const VALUE: bool> ShouldBe<VALUE> for HasPadding<T, VALUE> {}

/// A type whose size is equal to `align_of::<T>()`.
#[repr(C)]
pub struct AlignOf<T> {
    // This field ensures that:
    // - The size is always at least 1 (the minimum possible alignment).
    // - If the alignment is greater than 1, Rust has to round up to the next
    //   multiple of it in order to make sure that `Align`'s size is a multiple
    //   of that alignment. Without this field, its size could be 0, which is a
    //   valid multiple of any alignment.
    _u: u8,
    _a: [T; 0],
}

impl<T> AlignOf<T> {
    #[inline(never)] // Make `missing_inline_in_public_items` happy.
    pub fn into_t(self) -> T {
        unreachable!()
    }
}

/// A type whose size is equal to `max(align_of::<T>(), align_of::<U>())`.
#[repr(C)]
pub union MaxAlignsOf<T, U> {
    _t: ManuallyDrop<AlignOf<T>>,
    _u: ManuallyDrop<AlignOf<U>>,
}

impl<T, U> MaxAlignsOf<T, U> {
    #[inline(never)] // Make `missing_inline_in_public_items` happy.
    pub fn new(_t: T, _u: U) -> MaxAlignsOf<T, U> {
        unreachable!()
    }
}

const _64K: usize = 1 << 16;

// TODO(#29), TODO(https://github.com/rust-lang/rust/issues/69835): Remove this
// `cfg` when `size_of_val_raw` is stabilized.
#[cfg(__INTERNAL_USE_ONLY_NIGHLTY_FEATURES_IN_TESTS)]
#[repr(C, align(65536))]
struct Aligned64kAllocation([u8; _64K]);

/// A pointer to an aligned allocation of size 2^16.
///
/// # Safety
///
/// `ALIGNED_64K_ALLOCATION` is guaranteed to point to the entirety of an
/// allocation with size and alignment 2^16, and to have valid provenance.
// TODO(#29), TODO(https://github.com/rust-lang/rust/issues/69835): Remove this
// `cfg` when `size_of_val_raw` is stabilized.
#[cfg(__INTERNAL_USE_ONLY_NIGHLTY_FEATURES_IN_TESTS)]
pub const ALIGNED_64K_ALLOCATION: NonNull<[u8]> = {
    const REF: &Aligned64kAllocation = &Aligned64kAllocation([0; _64K]);
    let ptr: *const Aligned64kAllocation = REF;
    let ptr: *const [u8] = ptr::slice_from_raw_parts(ptr.cast(), _64K);
    // SAFETY:
    // - `ptr` is derived from a Rust reference, which is guaranteed to be
    //   non-null.
    // - `ptr` is derived from an `&Aligned64kAllocation`, which has size and
    //   alignment `_64K` as promised. Its length is initialized to `_64K`,
    //   which means that it refers to the entire allocation.
    // - `ptr` is derived from a Rust reference, which is guaranteed to have
    //   valid provenance.
    //
    // TODO(#429): Once `NonNull::new_unchecked` docs document that it preserves
    // provenance, cite those docs.
    // TODO: Replace this `as` with `ptr.cast_mut()` once our MSRV >= 1.65
    #[allow(clippy::as_conversions)]
    unsafe {
        NonNull::new_unchecked(ptr as *mut _)
    }
};

/// Computes the offset of the base of the field `$trailing_field_name` within
/// the type `$ty`.
///
/// `trailing_field_offset!` produces code which is valid in a `const` context.
// TODO(#29), TODO(https://github.com/rust-lang/rust/issues/69835): Remove this
// `cfg` when `size_of_val_raw` is stabilized.
#[cfg(__INTERNAL_USE_ONLY_NIGHLTY_FEATURES_IN_TESTS)]
#[doc(hidden)] // `#[macro_export]` bypasses this module's `#[doc(hidden)]`.
#[macro_export]
macro_rules! trailing_field_offset {
    ($ty:ty, $trailing_field_name:tt) => {{
        let min_size = {
            let zero_elems: *const [()] =
                $crate::macro_util::core_reexport::ptr::slice_from_raw_parts(
                    #[allow(clippy::incompatible_msrv)] // Work around https://github.com/rust-lang/rust-clippy/issues/12280
                    $crate::macro_util::core_reexport::ptr::NonNull::<()>::dangling()
                        .as_ptr()
                        .cast_const(),
                    0,
                );
            // SAFETY:
            // - If `$ty` is `Sized`, `size_of_val_raw` is always safe to call.
            // - Otherwise:
            //   - If `$ty` is not a slice DST, this pointer conversion will
            //     fail due to "mismatched vtable kinds", and compilation will
            //     fail.
            //   - If `$ty` is a slice DST, the safety requirement is that "the
            //     length of the slice tail must be an initialized integer, and
            //     the size of the entire value (dynamic tail length +
            //     statically sized prefix) must fit in isize." The length is
            //     initialized to 0 above, and Rust guarantees that no type's
            //     minimum size may overflow `isize`. [1]
            //
            // [1] TODO(#429),
            // TODO(https://github.com/rust-lang/unsafe-code-guidelines/issues/465#issuecomment-1782206516):
            // Citation for this?
            unsafe {
                #[allow(clippy::as_conversions)]
                $crate::macro_util::core_reexport::mem::size_of_val_raw(zero_elems as *const $ty)
            }
        };

        assert!(min_size <= _64K);

        #[allow(clippy::as_conversions)]
        let ptr = ALIGNED_64K_ALLOCATION.as_ptr() as *const $ty;

        // SAFETY:
        // - Thanks to the preceding `assert!`, we know that the value with zero
        //   elements fits in `_64K` bytes, and thus in the allocation addressed
        //   by `ALIGNED_64K_ALLOCATION`. The offset of the trailing field is
        //   guaranteed to be no larger than this size, so this field projection
        //   is guaranteed to remain in-bounds of its allocation.
        // - Because the minimum size is no larger than `_64K` bytes, and
        //   because an object's size must always be a multiple of its alignment
        //   [1], we know that `$ty`'s alignment is no larger than `_64K`. The
        //   allocation addressed by `ALIGNED_64K_ALLOCATION` is guaranteed to
        //   be aligned to `_64K`, so `ptr` is guaranteed to satisfy `$ty`'s
        //   alignment.
        //
        //   Note that, as of [2], this requirement is technically unnecessary
        //   for Rust versions >= 1.75.0, but no harm in guaranteeing it anyway
        //   until we bump our MSRV.
        //
        // [1] Per https://doc.rust-lang.org/reference/type-layout.html:
        //
        //   The size of a value is always a multiple of its alignment.
        //
        // [2] https://github.com/rust-lang/reference/pull/1387
        let field = unsafe {
            $crate::macro_util::core_reexport::ptr::addr_of!((*ptr).$trailing_field_name)
        };
        // SAFETY:
        // - Both `ptr` and `field` are derived from the same allocated object.
        // - By the preceding safety comment, `field` is in bounds of that
        //   allocated object.
        // - The distance, in bytes, between `ptr` and `field` is required to be
        //   a multiple of the size of `u8`, which is trivially true because
        //   `u8`'s size is 1.
        // - The distance, in bytes, cannot overflow `isize`. This is guaranteed
        //   because no allocated object can have a size larger than can fit in
        //   `isize`. [1]
        // - The distance being in-bounds cannot rely on wrapping around the
        //   address space. This is guaranteed because the same is guaranteed of
        //   allocated objects. [1]
        //
        // [1] TODO(#429), TODO(https://github.com/rust-lang/rust/pull/116675):
        //     Once these are guaranteed in the Reference, cite it.
        let offset = unsafe { field.cast::<u8>().offset_from(ptr.cast::<u8>()) };
        // Guaranteed not to be lossy: `field` comes after `ptr`, so the offset
        // from `ptr` to `field` is guaranteed to be positive.
        assert!(offset >= 0);
        Some(
            #[allow(clippy::as_conversions)]
            {
                offset as usize
            },
        )
    }};
}

/// Computes alignment of `$ty: ?Sized`.
///
/// `align_of!` produces code which is valid in a `const` context.
// TODO(#29), TODO(https://github.com/rust-lang/rust/issues/69835): Remove this
// `cfg` when `size_of_val_raw` is stabilized.
#[cfg(__INTERNAL_USE_ONLY_NIGHLTY_FEATURES_IN_TESTS)]
#[doc(hidden)] // `#[macro_export]` bypasses this module's `#[doc(hidden)]`.
#[macro_export]
macro_rules! align_of {
    ($ty:ty) => {{
        // SAFETY: `OffsetOfTrailingIsAlignment` is `repr(C)`, and its layout is
        // guaranteed [1] to begin with the single-byte layout for `_byte`,
        // followed by the padding needed to align `_trailing`, then the layout
        // for `_trailing`, and finally any trailing padding bytes needed to
        // correctly-align the entire struct.
        //
        // This macro computes the alignment of `$ty` by counting the number of
        // bytes preceeding `_trailing`. For instance, if the alignment of `$ty`
        // is `1`, then no padding is required align `_trailing` and it will be
        // located immediately after `_byte` at offset 1. If the alignment of
        // `$ty` is 2, then a single padding byte is required before
        // `_trailing`, and `_trailing` will be located at offset 2.

        // This correspondence between offset and alignment holds for all valid
        // Rust alignments, and we confirm this exhaustively (or, at least up to
        // the maximum alignment supported by `trailing_field_offset!`) in
        // `test_align_of_dst`.
        //
        // [1]: https://doc.rust-lang.org/nomicon/other-reprs.html#reprc

        #[repr(C)]
        struct OffsetOfTrailingIsAlignment {
            _byte: u8,
            _trailing: $ty,
        }

        trailing_field_offset!(OffsetOfTrailingIsAlignment, _trailing)
    }};
}

/// Does the struct type `$t` have padding?
///
/// `$ts` is the list of the type of every field in `$t`. `$t` must be a
/// struct type, or else `struct_has_padding!`'s result may be meaningless.
///
/// Note that `struct_has_padding!`'s results are independent of `repr` since
/// they only consider the size of the type and the sizes of the fields.
/// Whatever the repr, the size of the type already takes into account any
/// padding that the compiler has decided to add. Structs with well-defined
/// representations (such as `repr(C)`) can use this macro to check for padding.
/// Note that while this may yield some consistent value for some `repr(Rust)`
/// structs, it is not guaranteed across platforms or compilations.
#[doc(hidden)] // `#[macro_export]` bypasses this module's `#[doc(hidden)]`.
#[macro_export]
macro_rules! struct_has_padding {
    ($t:ty, $($ts:ty),*) => {
        core::mem::size_of::<$t>() > 0 $(+ core::mem::size_of::<$ts>())*
    };
}

/// Does the union type `$t` have padding?
///
/// `$ts` is the list of the type of every field in `$t`. `$t` must be a
/// union type, or else `union_has_padding!`'s result may be meaningless.
///
/// Note that `union_has_padding!`'s results are independent of `repr` since
/// they only consider the size of the type and the sizes of the fields.
/// Whatever the repr, the size of the type already takes into account any
/// padding that the compiler has decided to add. Unions with well-defined
/// representations (such as `repr(C)`) can use this macro to check for padding.
/// Note that while this may yield some consistent value for some `repr(Rust)`
/// unions, it is not guaranteed across platforms or compilations.
#[doc(hidden)] // `#[macro_export]` bypasses this module's `#[doc(hidden)]`.
#[macro_export]
macro_rules! union_has_padding {
    ($t:ty, $($ts:ty),*) => {
        false $(|| core::mem::size_of::<$t>() != core::mem::size_of::<$ts>())*
    };
}

/// Does `t` have alignment greater than or equal to `u`?  If not, this macro
/// produces a compile error. It must be invoked in a dead codepath. This is
/// used in `transmute_ref!` and `transmute_mut!`.
#[doc(hidden)] // `#[macro_export]` bypasses this module's `#[doc(hidden)]`.
#[macro_export]
macro_rules! assert_align_gt_eq {
    ($t:ident, $u: ident) => {{
        // The comments here should be read in the context of this macro's
        // invocations in `transmute_ref!` and `transmute_mut!`.
        if false {
            // The type wildcard in this bound is inferred to be `T` because
            // `align_of.into_t()` is assigned to `t` (which has type `T`).
            let align_of: $crate::macro_util::AlignOf<_> = unreachable!();
            $t = align_of.into_t();
            // `max_aligns` is inferred to have type `MaxAlignsOf<T, U>` because
            // of the inferred types of `t` and `u`.
            let mut max_aligns = $crate::macro_util::MaxAlignsOf::new($t, $u);

            // This transmute will only compile successfully if
            // `align_of::<T>() == max(align_of::<T>(), align_of::<U>())` - in
            // other words, if `align_of::<T>() >= align_of::<U>()`.
            //
            // SAFETY: This code is never run.
            max_aligns = unsafe { $crate::macro_util::core_reexport::mem::transmute(align_of) };
        } else {
            loop {}
        }
    }};
}

/// Do `t` and `u` have the same size?  If not, this macro produces a compile
/// error. It must be invoked in a dead codepath. This is used in
/// `transmute_ref!` and `transmute_mut!`.
#[doc(hidden)] // `#[macro_export]` bypasses this module's `#[doc(hidden)]`.
#[macro_export]
macro_rules! assert_size_eq {
    ($t:ident, $u: ident) => {{
        // The comments here should be read in the context of this macro's
        // invocations in `transmute_ref!` and `transmute_mut!`.
        if false {
            // SAFETY: This code is never run.
            $u = unsafe {
                // Clippy: It's okay to transmute a type to itself.
                #[allow(clippy::useless_transmute)]
                $crate::macro_util::core_reexport::mem::transmute($t)
            };
        } else {
            loop {}
        }
    }};
}

/// Transmutes a reference of one type to a reference of another type.
///
/// # Safety
///
/// The caller must guarantee that:
/// - `Src: IntoBytes + NoCell`
/// - `Dst: FromBytes + NoCell`
/// - `size_of::<Src>() == size_of::<Dst>()`
/// - `align_of::<Src>() >= align_of::<Dst>()`
#[inline(always)]
pub const unsafe fn transmute_ref<'dst, 'src: 'dst, Src: 'src, Dst: 'dst>(
    src: &'src Src,
) -> &'dst Dst {
    let src: *const Src = src;
    let dst = src.cast::<Dst>();
    // SAFETY:
    // - We know that it is sound to view the target type of the input reference
    //   (`Src`) as the target type of the output reference (`Dst`) because the
    //   caller has guaranteed that `Src: IntoBytes`, `Dst: FromBytes`, and
    //   `size_of::<Src>() == size_of::<Dst>()`.
    // - We know that there are no `UnsafeCell`s, and thus we don't have to
    //   worry about `UnsafeCell` overlap, because `Src: NoCell` and `Dst:
    //   NoCell`.
    // - The caller has guaranteed that alignment is not increased.
    // - We know that the returned lifetime will not outlive the input lifetime
    //   thanks to the lifetime bounds on this function.
    //
    // TODO(#67): Once our MSRV is 1.58, replace this `transmute` with `&*dst`.
    #[allow(clippy::transmute_ptr_to_ref)]
    unsafe {
        core::mem::transmute(dst)
    }
}

/// Transmutes a mutable reference of one type to a mutable reference of another
/// type.
///
/// # Safety
///
/// The caller must guarantee that:
/// - `Src: FromBytes + IntoBytes + NoCell`
/// - `Dst: FromBytes + IntoBytes + NoCell`
/// - `size_of::<Src>() == size_of::<Dst>()`
/// - `align_of::<Src>() >= align_of::<Dst>()`
// TODO(#686): Consider removing the `NoCell` requirement.
#[inline(always)]
pub unsafe fn transmute_mut<'dst, 'src: 'dst, Src: 'src, Dst: 'dst>(
    src: &'src mut Src,
) -> &'dst mut Dst {
    let src: *mut Src = src;
    let dst = src.cast::<Dst>();
    // SAFETY:
    // - We know that it is sound to view the target type of the input reference
    //   (`Src`) as the target type of the output reference (`Dst`) and
    //   vice-versa because the caller has guaranteed that `Src: FromBytes +
    //   IntoBytes`, `Dst: FromBytes + IntoBytes`, and `size_of::<Src>() ==
    //   size_of::<Dst>()`.
    // - We know that there are no `UnsafeCell`s, and thus we don't have to
    //   worry about `UnsafeCell` overlap, because `Src: NoCell`
    //   and `Dst: NoCell`.
    // - The caller has guaranteed that alignment is not increased.
    // - We know that the returned lifetime will not outlive the input lifetime
    //   thanks to the lifetime bounds on this function.
    unsafe { &mut *dst }
}

// NOTE: We can't change this to a `pub use core as core_reexport` until [1] is
// fixed or we update to a semver-breaking version (as of this writing, 0.8.0)
// on the `main` branch.
//
// [1] https://github.com/obi1kenobi/cargo-semver-checks/issues/573
pub mod core_reexport {
    pub use core::*;

    pub mod mem {
        pub use core::mem::*;
    }
}

#[cfg(test)]
mod tests {
    use core::mem;

    use super::*;
    use crate::util::testutil::*;

    #[test]
    fn test_align_of() {
        macro_rules! test {
            ($ty:ty) => {
                assert_eq!(mem::size_of::<AlignOf<$ty>>(), mem::align_of::<$ty>());
            };
        }

        test!(());
        test!(u8);
        test!(AU64);
        test!([AU64; 2]);
    }

    #[test]
    fn test_max_aligns_of() {
        macro_rules! test {
            ($t:ty, $u:ty) => {
                assert_eq!(
                    mem::size_of::<MaxAlignsOf<$t, $u>>(),
                    core::cmp::max(mem::align_of::<$t>(), mem::align_of::<$u>())
                );
            };
        }

        test!(u8, u8);
        test!(u8, AU64);
        test!(AU64, u8);
    }

    #[test]
    fn test_typed_align_check() {
        // Test that the type-based alignment check used in
        // `assert_align_gt_eq!` behaves as expected.

        macro_rules! assert_t_align_gteq_u_align {
            ($t:ty, $u:ty, $gteq:expr) => {
                assert_eq!(
                    mem::size_of::<MaxAlignsOf<$t, $u>>() == mem::size_of::<AlignOf<$t>>(),
                    $gteq
                );
            };
        }

        assert_t_align_gteq_u_align!(u8, u8, true);
        assert_t_align_gteq_u_align!(AU64, AU64, true);
        assert_t_align_gteq_u_align!(AU64, u8, true);
        assert_t_align_gteq_u_align!(u8, AU64, false);
    }

    // TODO(#29), TODO(https://github.com/rust-lang/rust/issues/69835): Remove
    // this `cfg` when `size_of_val_raw` is stabilized.
    #[allow(clippy::decimal_literal_representation)]
    #[cfg(__INTERNAL_USE_ONLY_NIGHLTY_FEATURES_IN_TESTS)]
    #[test]
    fn test_trailing_field_offset() {
        assert_eq!(mem::align_of::<Aligned64kAllocation>(), _64K);

        macro_rules! test {
            (#[$cfg:meta] ($($ts:ty),* ; $trailing_field_ty:ty) => $expect:expr) => {{
                #[$cfg]
                struct Test($(#[allow(dead_code)] $ts,)* #[allow(dead_code)] $trailing_field_ty);
                assert_eq!(test!(@offset $($ts),* ; $trailing_field_ty), $expect);
            }};
            (#[$cfg:meta] $(#[$cfgs:meta])* ($($ts:ty),* ; $trailing_field_ty:ty) => $expect:expr) => {
                test!(#[$cfg] ($($ts),* ; $trailing_field_ty) => $expect);
                test!($(#[$cfgs])* ($($ts),* ; $trailing_field_ty) => $expect);
            };
            (@offset ; $_trailing:ty) => { trailing_field_offset!(Test, 0) };
            (@offset $_t:ty ; $_trailing:ty) => { trailing_field_offset!(Test, 1) };
        }

        test!(#[repr(C)] #[repr(transparent)] #[repr(packed)](; u8) => Some(0));
        test!(#[repr(C)] #[repr(transparent)] #[repr(packed)](; [u8]) => Some(0));
        test!(#[repr(C)] #[repr(packed)] (u8; u8) => Some(1));
        test!(#[repr(C)] (; AU64) => Some(0));
        test!(#[repr(C)] (; [AU64]) => Some(0));
        test!(#[repr(C)] (u8; AU64) => Some(8));
        test!(#[repr(C)] (u8; [AU64]) => Some(8));
        test!(#[repr(C)] (; Nested<u8, AU64>) => Some(0));
        test!(#[repr(C)] (; Nested<u8, [AU64]>) => Some(0));
        test!(#[repr(C)] (u8; Nested<u8, AU64>) => Some(8));
        test!(#[repr(C)] (u8; Nested<u8, [AU64]>) => Some(8));

        // Test that `packed(N)` limits the offset of the trailing field.
        test!(#[repr(C, packed(        1))] (u8; elain::Align<        2>) => Some(        1));
        test!(#[repr(C, packed(        2))] (u8; elain::Align<        4>) => Some(        2));
        test!(#[repr(C, packed(        4))] (u8; elain::Align<        8>) => Some(        4));
        test!(#[repr(C, packed(        8))] (u8; elain::Align<       16>) => Some(        8));
        test!(#[repr(C, packed(       16))] (u8; elain::Align<       32>) => Some(       16));
        test!(#[repr(C, packed(       32))] (u8; elain::Align<       64>) => Some(       32));
        test!(#[repr(C, packed(       64))] (u8; elain::Align<      128>) => Some(       64));
        test!(#[repr(C, packed(      128))] (u8; elain::Align<      256>) => Some(      128));
        test!(#[repr(C, packed(      256))] (u8; elain::Align<      512>) => Some(      256));
        test!(#[repr(C, packed(      512))] (u8; elain::Align<     1024>) => Some(      512));
        test!(#[repr(C, packed(     1024))] (u8; elain::Align<     2048>) => Some(     1024));
        test!(#[repr(C, packed(     2048))] (u8; elain::Align<     4096>) => Some(     2048));
        test!(#[repr(C, packed(     4096))] (u8; elain::Align<     8192>) => Some(     4096));
        test!(#[repr(C, packed(     8192))] (u8; elain::Align<    16384>) => Some(     8192));
        test!(#[repr(C, packed(    16384))] (u8; elain::Align<    32768>) => Some(    16384));
        test!(#[repr(C, packed(    32768))] (u8; elain::Align<    65536>) => Some(    32768));
        test!(#[repr(C, packed(    65536))] (u8; elain::Align<   131072>) => Some(    65536));
        /* Alignments above 65536 are not yet supported.
        test!(#[repr(C, packed(   131072))] (u8; elain::Align<   262144>) => Some(   131072));
        test!(#[repr(C, packed(   262144))] (u8; elain::Align<   524288>) => Some(   262144));
        test!(#[repr(C, packed(   524288))] (u8; elain::Align<  1048576>) => Some(   524288));
        test!(#[repr(C, packed(  1048576))] (u8; elain::Align<  2097152>) => Some(  1048576));
        test!(#[repr(C, packed(  2097152))] (u8; elain::Align<  4194304>) => Some(  2097152));
        test!(#[repr(C, packed(  4194304))] (u8; elain::Align<  8388608>) => Some(  4194304));
        test!(#[repr(C, packed(  8388608))] (u8; elain::Align< 16777216>) => Some(  8388608));
        test!(#[repr(C, packed( 16777216))] (u8; elain::Align< 33554432>) => Some( 16777216));
        test!(#[repr(C, packed( 33554432))] (u8; elain::Align< 67108864>) => Some( 33554432));
        test!(#[repr(C, packed( 67108864))] (u8; elain::Align< 33554432>) => Some( 67108864));
        test!(#[repr(C, packed( 33554432))] (u8; elain::Align<134217728>) => Some( 33554432));
        test!(#[repr(C, packed(134217728))] (u8; elain::Align<268435456>) => Some(134217728));
        test!(#[repr(C, packed(268435456))] (u8; elain::Align<268435456>) => Some(268435456));
        */

        // Test that `align(N)` does not limit the offset of the trailing field.
        test!(#[repr(C, align(        1))] (u8; elain::Align<        2>) => Some(        2));
        test!(#[repr(C, align(        2))] (u8; elain::Align<        4>) => Some(        4));
        test!(#[repr(C, align(        4))] (u8; elain::Align<        8>) => Some(        8));
        test!(#[repr(C, align(        8))] (u8; elain::Align<       16>) => Some(       16));
        test!(#[repr(C, align(       16))] (u8; elain::Align<       32>) => Some(       32));
        test!(#[repr(C, align(       32))] (u8; elain::Align<       64>) => Some(       64));
        test!(#[repr(C, align(       64))] (u8; elain::Align<      128>) => Some(      128));
        test!(#[repr(C, align(      128))] (u8; elain::Align<      256>) => Some(      256));
        test!(#[repr(C, align(      256))] (u8; elain::Align<      512>) => Some(      512));
        test!(#[repr(C, align(      512))] (u8; elain::Align<     1024>) => Some(     1024));
        test!(#[repr(C, align(     1024))] (u8; elain::Align<     2048>) => Some(     2048));
        test!(#[repr(C, align(     2048))] (u8; elain::Align<     4096>) => Some(     4096));
        test!(#[repr(C, align(     4096))] (u8; elain::Align<     8192>) => Some(     8192));
        test!(#[repr(C, align(     8192))] (u8; elain::Align<    16384>) => Some(    16384));
        test!(#[repr(C, align(    16384))] (u8; elain::Align<    32768>) => Some(    32768));
        test!(#[repr(C, align(    32768))] (u8; elain::Align<    65536>) => Some(    65536));
        /* Alignments above 65536 are not yet supported.
        test!(#[repr(C, align(    65536))] (u8; elain::Align<   131072>) => Some(   131072));
        test!(#[repr(C, align(   131072))] (u8; elain::Align<   262144>) => Some(   262144));
        test!(#[repr(C, align(   262144))] (u8; elain::Align<   524288>) => Some(   524288));
        test!(#[repr(C, align(   524288))] (u8; elain::Align<  1048576>) => Some(  1048576));
        test!(#[repr(C, align(  1048576))] (u8; elain::Align<  2097152>) => Some(  2097152));
        test!(#[repr(C, align(  2097152))] (u8; elain::Align<  4194304>) => Some(  4194304));
        test!(#[repr(C, align(  4194304))] (u8; elain::Align<  8388608>) => Some(  8388608));
        test!(#[repr(C, align(  8388608))] (u8; elain::Align< 16777216>) => Some( 16777216));
        test!(#[repr(C, align( 16777216))] (u8; elain::Align< 33554432>) => Some( 33554432));
        test!(#[repr(C, align( 33554432))] (u8; elain::Align< 67108864>) => Some( 67108864));
        test!(#[repr(C, align( 67108864))] (u8; elain::Align< 33554432>) => Some( 33554432));
        test!(#[repr(C, align( 33554432))] (u8; elain::Align<134217728>) => Some(134217728));
        test!(#[repr(C, align(134217728))] (u8; elain::Align<268435456>) => Some(268435456));
        */
    }

    // TODO(#29), TODO(https://github.com/rust-lang/rust/issues/69835): Remove
    // this `cfg` when `size_of_val_raw` is stabilized.
    #[allow(clippy::decimal_literal_representation)]
    #[cfg(__INTERNAL_USE_ONLY_NIGHLTY_FEATURES_IN_TESTS)]
    #[test]
    fn test_align_of_dst() {
        // Test that `align_of!` correctly computes the alignment of DSTs.
        assert_eq!(align_of!([elain::Align<1>]), Some(1));
        assert_eq!(align_of!([elain::Align<2>]), Some(2));
        assert_eq!(align_of!([elain::Align<4>]), Some(4));
        assert_eq!(align_of!([elain::Align<8>]), Some(8));
        assert_eq!(align_of!([elain::Align<16>]), Some(16));
        assert_eq!(align_of!([elain::Align<32>]), Some(32));
        assert_eq!(align_of!([elain::Align<64>]), Some(64));
        assert_eq!(align_of!([elain::Align<128>]), Some(128));
        assert_eq!(align_of!([elain::Align<256>]), Some(256));
        assert_eq!(align_of!([elain::Align<512>]), Some(512));
        assert_eq!(align_of!([elain::Align<1024>]), Some(1024));
        assert_eq!(align_of!([elain::Align<2048>]), Some(2048));
        assert_eq!(align_of!([elain::Align<4096>]), Some(4096));
        assert_eq!(align_of!([elain::Align<8192>]), Some(8192));
        assert_eq!(align_of!([elain::Align<16384>]), Some(16384));
        assert_eq!(align_of!([elain::Align<32768>]), Some(32768));
        assert_eq!(align_of!([elain::Align<65536>]), Some(65536));
        /* Alignments above 65536 are not yet supported.
        assert_eq!(align_of!([elain::Align<131072>]), Some(131072));
        assert_eq!(align_of!([elain::Align<262144>]), Some(262144));
        assert_eq!(align_of!([elain::Align<524288>]), Some(524288));
        assert_eq!(align_of!([elain::Align<1048576>]), Some(1048576));
        assert_eq!(align_of!([elain::Align<2097152>]), Some(2097152));
        assert_eq!(align_of!([elain::Align<4194304>]), Some(4194304));
        assert_eq!(align_of!([elain::Align<8388608>]), Some(8388608));
        assert_eq!(align_of!([elain::Align<16777216>]), Some(16777216));
        assert_eq!(align_of!([elain::Align<33554432>]), Some(33554432));
        assert_eq!(align_of!([elain::Align<67108864>]), Some(67108864));
        assert_eq!(align_of!([elain::Align<33554432>]), Some(33554432));
        assert_eq!(align_of!([elain::Align<134217728>]), Some(134217728));
        assert_eq!(align_of!([elain::Align<268435456>]), Some(268435456));
        */
    }

    #[test]
    fn test_struct_has_padding() {
        // Test that, for each provided repr, `struct_has_padding!` reports the
        // expected value.
        macro_rules! test {
            (#[$cfg:meta] ($($ts:ty),*) => $expect:expr) => {{
                #[$cfg]
                struct Test($(#[allow(dead_code)] $ts),*);
                assert_eq!(struct_has_padding!(Test, $($ts),*), $expect);
            }};
            (#[$cfg:meta] $(#[$cfgs:meta])* ($($ts:ty),*) => $expect:expr) => {
                test!(#[$cfg] ($($ts),*) => $expect);
                test!($(#[$cfgs])* ($($ts),*) => $expect);
            };
        }

        test!(#[repr(C)] #[repr(transparent)] #[repr(packed)] () => false);
        test!(#[repr(C)] #[repr(transparent)] #[repr(packed)] (u8) => false);
        test!(#[repr(C)] #[repr(transparent)] #[repr(packed)] (u8, ()) => false);
        test!(#[repr(C)] #[repr(packed)] (u8, u8) => false);

        test!(#[repr(C)] (u8, AU64) => true);
        // Rust won't let you put `#[repr(packed)]` on a type which contains a
        // `#[repr(align(n > 1))]` type (`AU64`), so we have to use `u64` here.
        // It's not ideal, but it definitely has align > 1 on /some/ of our CI
        // targets, and this isn't a particularly complex macro we're testing
        // anyway.
        test!(#[repr(packed)] (u8, u64) => false);
    }

    #[test]
    fn test_union_has_padding() {
        // Test that, for each provided repr, `union_has_padding!` reports the
        // expected value.
        macro_rules! test {
            (#[$cfg:meta] {$($fs:ident: $ts:ty),*} => $expect:expr) => {{
                #[$cfg]
                #[allow(unused)] // fields are never read
                union Test{ $($fs: $ts),* }
                assert_eq!(union_has_padding!(Test, $($ts),*), $expect);
            }};
            (#[$cfg:meta] $(#[$cfgs:meta])* {$($fs:ident: $ts:ty),*} => $expect:expr) => {
                test!(#[$cfg] {$($fs: $ts),*} => $expect);
                test!($(#[$cfgs])* {$($fs: $ts),*} => $expect);
            };
        }

        test!(#[repr(C)] #[repr(packed)] {a: u8} => false);
        test!(#[repr(C)] #[repr(packed)] {a: u8, b: u8} => false);

        // Rust won't let you put `#[repr(packed)]` on a type which contains a
        // `#[repr(align(n > 1))]` type (`AU64`), so we have to use `u64` here.
        // It's not ideal, but it definitely has align > 1 on /some/ of our CI
        // targets, and this isn't a particularly complex macro we're testing
        // anyway.
        test!(#[repr(C)] #[repr(packed)] {a: u8, b: u64} => true);
    }
}