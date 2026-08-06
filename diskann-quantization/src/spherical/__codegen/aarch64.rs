/*
 * Copyright (c) Microsoft Corporation.
 * Licensed under the MIT license.
 */

//! Explicitly instantiate the AArch64 Neon spherical inner-product path for
//! 4-bit data against 4-bit data.

use diskann_wide::arch::aarch64::Neon;

use crate::{
    alloc::{AllocatorError, GlobalAllocator},
    spherical::{
        iface::{AsData, DistanceComputer, Reify},
        vectors,
    },
};

/// Register the Neon implementation of spherical inner product for
/// `USlice<'_, 4> × USlice<'_, 4>`.
#[inline(never)]
pub fn fourbit_neon_ip_data_data(
    arch: Neon,
    shift: &[f32],
    dim: usize,
) -> Result<DistanceComputer, AllocatorError> {
    let reify = Reify::<_, _, AsData<4>, AsData<4>>::new(
        vectors::CompensatedIP::new(shift, dim),
        dim,
        arch,
    );

    DistanceComputer::new(reify, GlobalAllocator)
}