use crate::core::{backend::vulkan::VulkanBackend, channel::Blake2sChannel, proof_of_work::GrindOps};


impl GrindOps<Blake2sChannel> for VulkanBackend {
    fn grind(_: &Blake2sChannel, _: u32) -> u64 {
        unimplemented!()
    }
} 