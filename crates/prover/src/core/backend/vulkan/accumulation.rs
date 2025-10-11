use std::sync::Arc;

use num_traits::One;
use vulkano::buffer::{ Buffer, BufferCreateInfo, BufferUsage };
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{ AutoCommandBufferBuilder, CommandBufferUsage };
use vulkano::descriptor_set::{ DescriptorSet, WriteDescriptorSet };
use vulkano::memory::allocator::{ AllocationCreateInfo, MemoryTypeFilter };
use vulkano::pipeline::{ Pipeline, PipelineBindPoint };
use vulkano::sync::GpuFuture;

use crate::core::air::accumulation::AccumulationOps;
use crate::core::backend::vulkan::gpu_context::{ PIPELINE_ACCUMULATE };
use crate::core::backend::vulkan::{ VulkanBackend };
use crate::core::fields::qm31::SecureField;
use crate::core::fields::secure_column::SecureColumnByCoords;

impl AccumulationOps for VulkanBackend {
    fn accumulate(column: &mut SecureColumnByCoords<Self>, other: &SecureColumnByCoords<Self>) {
        // Initialize Vulkan context
        let context = Self::gpu_context();
        // Single in-place buffer
        let buffer = Buffer::from_iter(
            context.memory_allocator(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER |
                BufferUsage::TRANSFER_SRC |
                BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST |
                MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            column.to_vec()
        ).expect("Failed to create buffer");
        // other buffer
        let other_buffer = Buffer::from_iter(
            context.memory_allocator(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER |
                BufferUsage::TRANSFER_SRC |
                BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST |
                MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            other.to_vec()
        ).expect("Failed to create buffer");
        let pipeline = context.pipeline(PIPELINE_ACCUMULATE);
        // 创建描述符集
        let descriptor_set = DescriptorSet::new(
            context.descriptor_allocator(),
            pipeline.layout().set_layouts()[0].clone(),
            [
                WriteDescriptorSet::buffer(0, buffer.clone()),
                WriteDescriptorSet::buffer(1, other_buffer.clone()),
            ],
            []
        ).expect("Failed to create descriptor set");

        let cb_allocator = Arc::new(
            StandardCommandBufferAllocator::new(context.device(), Default::default())
        );
        // 计算工作组数量
        let workgroup_count = (column.columns.len() * column.packed_len() + 255) / 256;

        // 创建并执行命令缓冲区
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            cb_allocator,
            context.queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit
        ).expect("Failed to create command buffer builder");

        command_buffer_builder
            .bind_pipeline_compute(pipeline.clone())
            .expect("Failed to bind compute pipeline")
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                pipeline.layout().clone(),
                0,
                descriptor_set
            )
            .expect("Failed to bind descriptor set");

        unsafe {
            command_buffer_builder
                .dispatch([workgroup_count as u32, 1, 1])
                .expect("Failed to dispatch");
        }

        let command_buffer = command_buffer_builder
            .build()
            .expect("Failed to build command buffer");

        // 执行计算
        let future = vulkano::sync
            ::now(context.device())
            .then_execute(context.queue(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).expect("Failed to wait for fence");

        // Read result back to CPU
        let mapped = buffer.read().expect("Failed to read buffer");
        column.copy_from_vec(bytemuck::cast_slice(&*mapped));
    }

    fn generate_secure_powers(felt: SecureField, n_powers: usize) -> Vec<SecureField> {
        (0..n_powers)
            .scan(SecureField::one(), |acc, _| {
                let res = *acc;
                *acc *= felt;
                Some(res)
            })
            .collect()
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
