use crate::core::{
    backend::{  CpuBackend, vulkan::VulkanBackend },
    channel::Blake2sChannel,
    proof_of_work::GrindOps,
};

impl GrindOps<Blake2sChannel> for VulkanBackend {
    fn grind(channel: &Blake2sChannel, pow_bits: u32) -> u64 {
        CpuBackend::grind(channel, pow_bits)
    }
}
