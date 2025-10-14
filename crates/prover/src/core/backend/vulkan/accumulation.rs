 

use crate::core::air::accumulation::AccumulationOps;
use crate::core::backend::simd::SimdBackend;
use crate::core::backend::vulkan::gpu_context::{ PIPELINE_ACCUMULATE };
use crate::core::backend::vulkan::{ VulkanBackend };
use crate::core::fields::qm31::SecureField;
use crate::core::fields::secure_column::SecureColumnByCoords;

impl AccumulationOps for VulkanBackend {
    fn accumulate(column: &mut SecureColumnByCoords<Self>, other: &SecureColumnByCoords<Self>) {
        // Initialize Vulkan context
        let context = Self::gpu_context();

        let buffer = context.buffer_in_out(&column.to_vec());
        let other_buffer = context.buffer_in_out(&other.to_vec());

        let pipeline = context.pipeline(PIPELINE_ACCUMULATE);
        let descriptor_set = context.descriptor_set(
            &pipeline,
            &[buffer.clone(), other_buffer.clone()]
        );
        let group_counts = context.group_counts(column.packed_len());    

        let command_buffer=context.command_buffer(&pipeline, descriptor_set, group_counts, &[]);

        context.execution_wait(command_buffer);
        // Read result back to CPU
        let mapped = buffer.read().expect("Failed to read buffer");
        column.copy_from_vec(bytemuck::cast_slice(&*mapped));
    }

    fn generate_secure_powers(felt: SecureField, n_powers: usize) -> Vec<SecureField> {
        SimdBackend::generate_secure_powers(felt, n_powers)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::{
            air::accumulation::AccumulationOps,
            backend::vulkan::{ VulkanBackend },
            fields::secure_column::SecureColumnByCoords,
        },
        qm31,
    };
    #[test]
    fn test_accumulate() {
        let mut column = SecureColumnByCoords::<VulkanBackend>::zeros(1 << 23);
        column.set(19, qm31!(1, 2, 3, 4));
        let mut other = SecureColumnByCoords::<VulkanBackend>::zeros(1 << 23);
        other.set(19, qm31!(4, 3, 2, 1));
        VulkanBackend::accumulate(&mut column, &other);
        assert_eq!(column.at(19), qm31!(5, 5, 5, 5));
    }
}
