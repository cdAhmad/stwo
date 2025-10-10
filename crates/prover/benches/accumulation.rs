#![feature(iter_array_chunks)]

use criterion::{ criterion_group, criterion_main, BatchSize, Criterion };
use stwo_prover::core::{
    air::accumulation::AccumulationOps,
    backend::{ simd::SimdBackend, vulkan::VulkanBackend, CpuBackend },
    fields::{ qm31::SecureField, secure_column::SecureColumnByCoords },
};

const LOG_SIZE: u32 = 26;
const SIZE: usize = 1 << LOG_SIZE;

pub fn cpu_accumulation(c: &mut Criterion) {
    // TODO(andrew): Consider using same size for all.
    let mut column = SecureColumnByCoords::<stwo_prover::core::backend::cpu::CpuBackend>::zeros(
        SIZE
    );
    let mut other = SecureColumnByCoords::<stwo_prover::core::backend::cpu::CpuBackend>::zeros(
        SIZE
    );
    for index in 0..SIZE {
        let temp=(index as u32)% 2147483647;
        column.set(index, SecureField::from(temp));
        other.set(index, SecureField::from(temp));
    }
    c.bench_function(format!("cpu accumlate {}bit", LOG_SIZE).as_str(), |b| {
        b.iter_batched(
            || column.clone(),
            |mut data| CpuBackend::accumulate(&mut data, &other),
            BatchSize::LargeInput
        );
    });
}

pub fn simd_accumulation(c: &mut Criterion) {
    let mut column = SecureColumnByCoords::<SimdBackend>::zeros(SIZE);
    let mut other = SecureColumnByCoords::<SimdBackend>::zeros(SIZE);
    for index in 0..SIZE {
        column.set(index, SecureField::from(index as u32));
        other.set(index, SecureField::from(index as u32));
    }

    c.bench_function(format!("simd accumlate {}bit", LOG_SIZE).as_str(), |b| {
        b.iter_batched(
            || column.clone(),
            |mut data| SimdBackend::accumulate(&mut data, &other),
            BatchSize::LargeInput
        );
    });
}

pub fn vulkan_accumulation(c: &mut Criterion) {
    let mut column = SecureColumnByCoords::<VulkanBackend>::zeros(SIZE);
    let mut other = SecureColumnByCoords::<VulkanBackend>::zeros(SIZE);
    for index in 0..SIZE {
        column.set(index, SecureField::from(index as u32));
        other.set(index, SecureField::from(index as u32));
    }

    c.bench_function(format!("vulkan accumlate {}bit", LOG_SIZE).as_str(), |b| {
        b.iter_batched(
            || column.clone(),
            |mut data| VulkanBackend::accumulate(&mut data, &other),
            BatchSize::LargeInput
        );
    });
}

criterion_group!(
    name = accumulation;
    config = Criterion::default().sample_size(10);
    targets = cpu_accumulation,simd_accumulation,vulkan_accumulation
);
criterion_main!(accumulation);
