/*
 * Copyright (c) Microsoft Corporation.
 * Licensed under the MIT license.
 */
use super::common::{USizeConvertTo, bytes};
use crate::{
    constant::Const,
    traits::{ArrayType, SIMDVector, InterleavedLoadStore},
};

pub(crate) fn test_deinterleaved_load<T, const N: usize, const M: usize, V>(arch: V::Arch)
where
    T: Default + std::marker::Copy + std::cmp::PartialEq + bytemuck::Pod + std::fmt::Debug,
    [T; N]: bytemuck::Pod,
    usize: USizeConvertTo<T>,
    Const<N>: ArrayType<T, Type = [T; N]>,
    Const<M>: ArrayType<Const<N>, Type = [Const<N>; M]>,
    V: SIMDVector<Scalar = T, ConstLanes = Const<N>> + InterleavedLoadStore<M>,
{
    let mut references = [[T::default(); N]; M];
    for (stream, reference) in references.iter_mut().enumerate() {
        reference.iter_mut().enumerate().for_each(|(i, d)| {
            let value = stream * N + i;
            *d = value.test_convert();
        });
    }
    
    let mut interleaved_references = [[T::default(); N]; M];
    for (stream, reference) in interleaved_references.iter_mut().enumerate() {
        reference.iter_mut().enumerate().for_each(|(i, d)| {
            let value = i*M + stream;
            *d = value.test_convert();
        });
    }

    let elsize: usize = std::mem::size_of::<T>();
    let mut input = vec![0u8; elsize * 2 * N * M];

    for i in 0..=N * M * elsize {
        input.fill(0);
        for (j, reference) in references.into_iter().enumerate() {
            let lower_range: usize = i + j * elsize * N;
            let upper_range: usize = lower_range + elsize * N;
            input[lower_range..upper_range].copy_from_slice(bytes(&reference));
        }

        let v = unsafe { V::load_deinterleaved(arch, input.as_ptr().add(i).cast::<T>()) };
        assert_eq!(v.len(), interleaved_references.len());
        for (stream, vector) in v.into_iter().enumerate() {
            let arr = vector.to_array();
            assert_eq!(arr, interleaved_references[stream]);
        }
    }
}

pub(crate) fn test_interleaved_store<T, const N: usize, const M: usize, V>(arch: V::Arch)
where
    T: Default + std::marker::Copy + std::cmp::PartialEq + bytemuck::Pod + std::fmt::Debug,
    [T; N]: bytemuck::Pod,
    usize: USizeConvertTo<T>,
    Const<N>: ArrayType<T, Type = [T; N]>,
    Const<M>: ArrayType<Const<N>, Type = [Const<N>; M]>,
    V: SIMDVector<Scalar = T, ConstLanes = Const<N>> + InterleavedLoadStore<M>,
{
    let mut references = [[T::default(); N]; M];
    for (stream, reference) in references.iter_mut().enumerate() {
        reference.iter_mut().enumerate().for_each(|(i, d)| {
            let value = stream * N + i;
            *d = value.test_convert();
        });
    }
    
    let mut interleaved_references = [[T::default(); N]; M];

    for (chunk, reference) in interleaved_references.iter_mut().enumerate() {
        reference.iter_mut().enumerate().for_each(|(i, d)| {
            let p = chunk * N + i;

            let source_stream = p % M;
            let source_lane = p / M;

            let value = source_stream * N + source_lane;
            *d = value.test_convert();
        });
}
    let v = references.map(|reference| V::from_array(arch, reference));
    let elsize: usize = std::mem::size_of::<T>();
    let mut output = vec![0u8; elsize * 2 * N * M];

    for i in 0..=N * M * elsize {
        output.fill(0);
        unsafe { V::store_interleaved(v, output.as_mut_ptr().add(i).cast::<T>()) };
        check(&output, bytes(&interleaved_references), i, &FullStore(i));
    }
}

struct FullStore(usize);
impl std::fmt::Display for FullStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "full SIMD store at byte offset {}", self.0)
    }
}

fn check(output: &[u8], target: &[u8], offset: usize, message: &dyn std::fmt::Display) {
    let iszero = |x: &u8| *x == 0;

    assert!(
        output[..offset].iter().all(iszero),
        "prefix of {:?} up to {} is not zero -- {}",
        output,
        offset,
        message
    );

    assert_eq!(
        &output[offset..offset + target.len()],
        target,
        "output window from {} not equal to target -- {}",
        offset,
        message
    );

    assert!(
        output[offset + target.len()..].iter().all(iszero),
        "suffix of {:?} starting from {} is not zero -- {}",
        output,
        offset + target.len(),
        message
    );
}