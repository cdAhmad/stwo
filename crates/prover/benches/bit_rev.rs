#![feature(iter_array_chunks)]

use criterion::{ criterion_group, criterion_main, BatchSize, Criterion };
use itertools::Itertools;
use stwo_prover::core::{ fields::m31::BaseField };

const LOG_SIZE: u32 = 26;
const SIZE: usize = 1 << LOG_SIZE;

pub fn cpu_bit_rev(c: &mut Criterion) {
    use stwo_prover::core::backend::cpu::bit_reverse;
    // TODO(andrew): Consider using same size for all.
    let data = (0..SIZE).map(BaseField::from).collect_vec();
    c.bench_function(format!("cpu bit_rev {}bit", LOG_SIZE).as_str(), |b| {
        b.iter_batched(
            || data.clone(),
            |mut data| bit_reverse(&mut data),
            BatchSize::LargeInput
        );
    });
}

pub fn simd_bit_rev(c: &mut Criterion) {
    use stwo_prover::core::backend::simd::bit_reverse::bit_reverse_m31;
    use stwo_prover::core::backend::simd::column::BaseColumn;
    let data = (0..SIZE).map(BaseField::from).collect::<BaseColumn>();
    c.bench_function(format!("simd bit_rev {}bit", LOG_SIZE).as_str(), |b| {
        b.iter_batched(
            || data.data.clone(),
            |mut data| bit_reverse_m31(&mut data),
            BatchSize::LargeInput
        );
    });
}

pub fn metal_bit_rev(c: &mut Criterion) {
    use stwo_prover::core::backend::ColumnOps;
    use stwo_prover::core::backend::metal::MetalBackend;
    use stwo_prover::core::fields::m31::BaseField;
    let data = (0..SIZE).map(BaseField::from).collect_vec();
    c.bench_function(format!("metal bit_rev {}bit", LOG_SIZE).as_str(), |b| {
        b.iter_batched(
            || data.clone(),
            |mut data| MetalBackend::bit_reverse_column(&mut data),
            BatchSize::LargeInput
        );
    });
}
pub fn vulkan_bit_rev(c: &mut Criterion) {
    use stwo_prover::core::backend::ColumnOps;
    use stwo_prover::core::backend::vulkan::VulkanBackend;
    use stwo_prover::core::fields::m31::BaseField;
    let data = (0..SIZE).map(BaseField::from).collect_vec();
    c.bench_function(format!("vulkan bit_rev {}bit", LOG_SIZE).as_str(), |b| {
        b.iter_batched(
            || data.clone(),
            |mut data| <VulkanBackend as ColumnOps<BaseField>>::bit_reverse_column(&mut data),
            BatchSize::LargeInput
        );
    });
}

criterion_group!(
    name = bit_rev;
    config = Criterion::default().sample_size(10);
    targets = simd_bit_rev, cpu_bit_rev,metal_bit_rev,vulkan_bit_rev);
criterion_main!(bit_rev);
