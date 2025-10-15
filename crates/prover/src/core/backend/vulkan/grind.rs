use crate::core::{
    backend::{ simd::SimdBackend, vulkan::VulkanBackend },
    channel::Blake2sChannel,
    proof_of_work::GrindOps,
};

impl GrindOps<Blake2sChannel> for VulkanBackend {
    fn grind(channel: &Blake2sChannel, pow_bits: u32) -> u64 {
        SimdBackend::grind(channel, pow_bits)
    }
}
