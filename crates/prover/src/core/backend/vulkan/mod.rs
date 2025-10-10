use serde::{ Deserialize, Serialize };
use vulkano::{
    buffer::{ Buffer, BufferCreateInfo, BufferUsage },
    command_buffer::{ AutoCommandBufferBuilder, CommandBufferUsage },
    descriptor_set::{ WriteDescriptorSet },
    memory::allocator::{ AllocationCreateInfo, MemoryTypeFilter },
    pipeline::{ Pipeline, PipelineBindPoint },
    sync::GpuFuture,
};
mod accumulation;
mod shaders;
mod m31;
mod column;
use vulkano::descriptor_set::{ DescriptorSet };
use std::{ fmt::Debug, sync::Arc };
use crate::core::{
    backend::{
        cpu::bit_reverse as cpu_bit_reverse,
        vulkan::gpu_context::{ GpuContext, GPU_CONTEXT, PIPELINE_BIT_REVERSE },
        ColumnOps,
    },
    fields::m31::BaseField,
};
mod circle;
mod gpu_context;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct VulkanBackend;
impl VulkanBackend {
    pub fn gpu_context() -> &'static Arc<GpuContext> {
        GPU_CONTEXT.get_or_init(GpuContext::new)
    }
}

impl ColumnOps<BaseField> for VulkanBackend {
    type Column = Vec<BaseField>;
    fn bit_reverse_column(column: &mut Self::Column) {
        let n = column.len();
        assert!(n.is_power_of_two());
        let log_n = n.ilog2();
        if n < 1 << 12 {
            cpu_bit_reverse(column);
            return;
        }
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
            column.iter().cloned()
        ).expect("Failed to create buffer");
        let pipeline = context.pipeline(PIPELINE_BIT_REVERSE);
        // 创建描述符集
        let descriptor_set = DescriptorSet::new(
            context.descriptor_allocator(),
            pipeline.layout().set_layouts()[0].clone(),
            [WriteDescriptorSet::buffer(0, buffer.clone())],
            []
        ).expect("Failed to create descriptor set");

        // 计算工作组数量
        let workgroup_count = ((n as u32) + 255) / 256;

        // 创建并执行命令缓冲区
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            context.command_allocator(),
            context.queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit
        ).expect("Failed to create command buffer builder");

        command_buffer_builder
            .bind_pipeline_compute(pipeline.clone())
            .expect("Failed to bind compute pipeline")
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                pipeline.clone().layout().clone(),
                0,
                descriptor_set
            )
            .expect("Failed to bind descriptor set");
        unsafe {
            let _ = command_buffer_builder
                .push_constants(pipeline.clone().layout().clone(), 0, [log_n as u32])
                .expect("Failed to push constants");
            command_buffer_builder.dispatch([workgroup_count, 1, 1]).expect("Failed to dispatch");
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
        column.copy_from_slice(bytemuck::cast_slice(&*mapped));
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::core::backend::vulkan::VulkanBackend;
    use crate::core::backend::ColumnOps;
    use crate::core::fields::m31::BaseField;

    use crate::core::backend::cpu::bit_reverse as cpu_bit_reverse;

    #[test]
    fn bit_reverse_large_column_works() {
        const LOG_SIZE: u32 = 22;
        let column = (0..1 << LOG_SIZE).map(BaseField::from).collect_vec();
        let mut expected = column.clone();
        cpu_bit_reverse(&mut expected);
        let mut res = column.clone();
        VulkanBackend::bit_reverse_column(&mut res);
        assert_eq!(res, expected);
    }
}
