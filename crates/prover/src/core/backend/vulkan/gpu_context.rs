use std::{ collections::HashMap, sync::{ Arc, OnceLock } };

use vulkano::{
    buffer::{ Buffer, BufferCreateInfo, BufferUsage, Subbuffer },
    command_buffer::{
        allocator::StandardCommandBufferAllocator,
        AutoCommandBufferBuilder,
        CommandBufferUsage,
        PrimaryAutoCommandBuffer,
        PrimaryCommandBufferAbstract,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator,
        DescriptorSet,
        WriteDescriptorSet,
    },
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
    shader::ShaderModule,
    sync::GpuFuture,
    Validated,
    VulkanError,
};
use vulkano_util::context::VulkanoContext;

use crate::core::backend::vulkan::shaders;

pub const PIPELINE_ACCUMULATE: &str = "accumulate";
pub const PIPELINE_BIT_REVERSE: &str = "bit_reverse";
pub const PIPELINE_BATCH_INVERSE: &str = "batch_inverse";
pub struct GpuContext {
    device: Arc<vulkano::device::Device>,
    queue: Arc<vulkano::device::Queue>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_allocator: Arc<StandardDescriptorSetAllocator>,
    pipelines: HashMap<&'static str, Arc<ComputePipeline>>,
}
const DISPATCH_SIZE: usize = 256;
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
            PIPELINE_ACCUMULATE,
            Self::create_pipeline(&device, shaders::accumulate::load(device.clone()))
        );

        pipelines.insert(
            PIPELINE_BIT_REVERSE,
            Self::create_pipeline(&device, shaders::bit_reverse::load(device.clone()))
        );
        pipelines.insert(
            PIPELINE_BATCH_INVERSE,
            Self::create_pipeline(&device, shaders::batch_inverse::load(device.clone()))
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

    pub fn buffer_in_out(self: &Arc<Self>, data: &[u32]) -> Subbuffer<[u32]> {
        Buffer::from_iter(
            self.memory_allocator(),
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
            data.iter().copied()
        ).expect("Failed to create buffer")
    }
    
    pub fn descriptor_set(
        self: &Arc<Self>,
        pipeline: &Arc<ComputePipeline>,
        buffers: &[Subbuffer<[u32]>]
    ) -> Arc<DescriptorSet> {
        DescriptorSet::new(
            self.descriptor_allocator(),
            pipeline.layout().set_layouts()[0].clone(),
            buffers.iter().enumerate().map(|(index, buffer)| {
                WriteDescriptorSet::buffer(index as u32, buffer.clone())
            }),
            []
        ).expect("Failed to create descriptor set")
    }
    pub fn group_counts(self: &Arc<Self>, count: usize) -> [u32; 3] {
        let group_count = (count + DISPATCH_SIZE - 1) / DISPATCH_SIZE;
        [group_count as u32, 1, 1]
    }

    pub fn command_buffer(
        self: &Arc<Self>,
        pipeline: &Arc<ComputePipeline>,
        descriptor_set: Arc<DescriptorSet>,
        group_counts: [u32; 3],
        push_constants: &[u32]
    ) -> Arc<PrimaryAutoCommandBuffer> {
        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_allocator(),
            self.queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit
        ).expect("Failed to create command buffer builder");
        builder.bind_pipeline_compute(pipeline.clone()).expect("Failed to bind compute pipeline");
        builder
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                pipeline.layout().clone(),
                0,
                descriptor_set
            )
            .expect("Failed to bind descriptor set");
        push_constants
            .iter()
            .enumerate()
            .for_each(|(i, v)| {
                builder
                    .push_constants(pipeline.layout().clone(), i as u32, v.clone())
                    .expect("Failed to push constants {i}:{v}");
            });

        unsafe {
            builder.dispatch(group_counts).expect("Failed to dispatch");
        }

        builder.build().expect("Failed to build command buffer")
    }
    pub fn execution_wait(self: &Arc<Self>, command_buffer: Arc<PrimaryAutoCommandBuffer>) {
        let future = command_buffer
            .execute(self.queue())
            .expect("Failed to execute command buffer");
        future
            .then_signal_fence_and_flush()
            .expect("Failed to signal fence")
            .wait(None)
            .expect("Failed to wait for future");
    }
}

pub static GPU_CONTEXT: OnceLock<Arc<GpuContext>> = OnceLock::new();

#[cfg(test)]
mod test {
    use vulkano::{
        device::physical::PhysicalDeviceType,
        instance::{ Instance, InstanceCreateInfo, InstanceExtensions },
        VulkanLibrary,
    };

    #[test]
    fn test() {
        let library = VulkanLibrary::new().expect("Failed to create Vulkan library");
        let instance = Instance::new(library, InstanceCreateInfo {
            enabled_extensions: InstanceExtensions::empty(), // 根据需要添加
            ..Default::default()
        }).expect("Failed to create instance");

        let physical_device = instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate physical devices")
            .filter(
                |p|
                    p.properties().device_type == PhysicalDeviceType::DiscreteGpu ||
                    p.properties().device_type == PhysicalDeviceType::IntegratedGpu
            )
            .next()
            .expect("No suitable physical device found");

        // 1. 检查物理设备是否支持 shaderInt64
        let supported_features = physical_device.supported_features();
        println!("shaderInt64: {}", supported_features.shader_int64);
    }
}
