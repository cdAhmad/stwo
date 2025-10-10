use std::{ collections::HashMap, sync::{ Arc, OnceLock } };

use vulkano::{
    command_buffer::allocator::StandardCommandBufferAllocator,
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    memory::allocator::StandardMemoryAllocator,
    pipeline::{
        compute::ComputePipelineCreateInfo,
        layout::PipelineDescriptorSetLayoutCreateInfo,
        ComputePipeline,
        PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
    shader::ShaderModule,
    Validated,
    VulkanError,
};
use vulkano_util::context::VulkanoContext;

use crate::core::backend::vulkan::shaders;

pub struct GpuContext {
    device: Arc<vulkano::device::Device>,
    queue: Arc<vulkano::device::Queue>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_allocator: Arc<StandardDescriptorSetAllocator>,
    pipelines: HashMap<&'static str, Arc<ComputePipeline>>,
}

impl GpuContext {
    pub fn new() -> Arc<Self> {
        let vulkano_context = VulkanoContext::new(Default::default());
        let device = vulkano_context.device().clone();
        let queue = vulkano_context.graphics_queue().clone();
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let command_allocator = Arc::new(
            StandardCommandBufferAllocator::new(device.clone(), Default::default())
        );
        let descriptor_allocator = Arc::new(
            StandardDescriptorSetAllocator::new(device.clone(), Default::default())
        );

        let mut pipelines = HashMap::new();
        // 创建所有 pipeline
        pipelines.insert(
            "accumulate",
            Self::create_pipeline(&device, shaders::accumulate::load(device.clone()))
        );

        pipelines.insert(
            "bit_reverse",
            Self::create_pipeline(&device, shaders::bit_reverse::load(device.clone()))
        );
        let a = Self {
            device,
            queue,
            memory_allocator,
            command_allocator,
            descriptor_allocator,
            pipelines,
        };
        Arc::new(a)
    }

    fn create_pipeline(
        device: &Arc<vulkano::device::Device>,
        shader_module: Result<Arc<ShaderModule>, Validated<VulkanError>>
    ) -> Arc<ComputePipeline> {
        let entry_point = shader_module
            .expect("Failed to create shader module")
            .entry_point("main")
            .unwrap();
        // 创建计算管线
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
    }

    pub fn device(self: &Arc<Self>) -> Arc<vulkano::device::Device> {
        self.device.clone()
    }
    pub fn queue(self: &Arc<Self>) -> Arc<vulkano::device::Queue> {
        self.queue.clone()
    }
    pub fn memory_allocator(self: &Arc<Self>) -> Arc<StandardMemoryAllocator> {
        self.memory_allocator.clone()
    }
    pub fn command_allocator(self: &Arc<Self>) -> Arc<StandardCommandBufferAllocator> {
        self.command_allocator.clone()
    }
    pub fn descriptor_allocator(self: &Arc<Self>) -> Arc<StandardDescriptorSetAllocator> {
        self.descriptor_allocator.clone()
    }

    pub fn pipeline(self: &Arc<Self>, name: &'static str) -> Arc<ComputePipeline> {
        self.pipelines.get(name).cloned().expect("Failed to get pipeline").clone()
    }
}

pub static GPU_CONTEXT: OnceLock<Arc<GpuContext>> = OnceLock::new();
