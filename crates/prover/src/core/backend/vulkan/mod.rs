use serde::{ Deserialize, Serialize };
use vulkano_util::context::VulkanoContext;
use vulkano::{
    buffer::{ Buffer, BufferCreateInfo, BufferUsage },
    command_buffer::{
        allocator::StandardCommandBufferAllocator,
        AutoCommandBufferBuilder,
        CommandBufferUsage,
    },
    descriptor_set::{ allocator::StandardDescriptorSetAllocator, WriteDescriptorSet },
    memory::allocator::{ AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator },
    pipeline::{
        compute::ComputePipelineCreateInfo,
        layout::PipelineDescriptorSetLayoutCreateInfo,
        ComputePipeline,
        Pipeline,
        PipelineBindPoint,
        PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
    sync::GpuFuture,
};
mod bit_reverse;
use vulkano::descriptor_set::{ DescriptorSet };
use std::{ fmt::Debug, sync::Arc };
use crate::core::{
    backend::{ cpu::bit_reverse as cpu_bit_reverse, ColumnOps },
    fields::m31::{ BaseField },
};
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct VulkanBackend;
impl VulkanBackend {
    fn vulkano_context() -> VulkanoContext {
        VulkanoContext::new(Default::default())
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
        // Initialize Vulkan context
        let vulkano_context = Self::vulkano_context();
        let queue = vulkano_context.graphics_queue();
        let device = vulkano_context.device();
        // 加载着色器

        let shader = bit_reverse::load(device.clone()).expect("Failed to create shader module");
        let entry_point = shader.entry_point("main").unwrap();
        // 创建计算管线
        let compute_pipeline = {
            let stage = PipelineShaderStageCreateInfo {
                ..PipelineShaderStageCreateInfo::new(entry_point)
            };
            let layout = PipelineLayout::new(
                device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
                    .into_pipeline_layout_create_info(device.clone())
                    .unwrap()
            ).unwrap();
            ComputePipeline::new(
                device.clone(),
                None,
                ComputePipelineCreateInfo::stage_layout(stage, layout)
            ).unwrap()
        };

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        // Single in-place buffer
        let buffer = Buffer::from_iter(
            memory_allocator.clone(),
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

        let ds_allocator = Arc::new(
            StandardDescriptorSetAllocator::new(device.clone(), Default::default())
        );
        // 创建描述符集

        let descriptor_set = DescriptorSet::new(
            ds_allocator,
            compute_pipeline.layout().set_layouts()[0].clone(),
            [WriteDescriptorSet::buffer(0, buffer.clone())],
            []
        ).expect("Failed to create descriptor set");

        let cb_allocator = Arc::new(
            StandardCommandBufferAllocator::new(device.clone(), Default::default())
        );
        // 计算工作组数量
        let workgroup_count = ((n as u32) + 253) / 254;

        // 创建并执行命令缓冲区
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            cb_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit
        ).expect("Failed to create command buffer builder");

        command_buffer_builder
            .bind_pipeline_compute(compute_pipeline.clone())
            .expect("Failed to bind compute pipeline")
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                compute_pipeline.layout().clone(),
                0,
                descriptor_set
            )
            .expect("Failed to bind descriptor set");
        unsafe {
            let _ = command_buffer_builder
                .push_constants(compute_pipeline.layout().clone(), 0, [log_n as u32])
                .expect("Failed to push constants");
            command_buffer_builder.dispatch([workgroup_count, 1, 1]).expect("Failed to dispatch");
        }

        let command_buffer = command_buffer_builder
            .build()
            .expect("Failed to build command buffer");

        // 执行计算
        let future = vulkano::sync
            ::now(device.clone())
            .then_execute(queue.clone(), command_buffer)
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
