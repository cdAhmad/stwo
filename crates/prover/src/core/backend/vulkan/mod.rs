use serde::{ Deserialize, Serialize };

mod accumulation;
mod shaders;
mod column;
mod quotients;
use std::{ fmt::Debug, sync::Arc };

use crate::core::{backend::{vulkan::gpu_context::{ GpuContext, GPU_CONTEXT }, Backend, BackendForChannel}, vcs::blake2_merkle::Blake2sMerkleChannel};

mod circle;
mod gpu_context;
mod blake2s;
mod grind;
mod lookups;
mod fri;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct VulkanBackend;

impl VulkanBackend {
    pub fn gpu_context() -> &'static Arc<GpuContext> {
        GPU_CONTEXT.get_or_init(GpuContext::new)
    }
}
impl Backend for VulkanBackend {}
impl BackendForChannel<Blake2sMerkleChannel> for VulkanBackend {}
// #[cfg(not(target_arch = "wasm32"))]
// impl BackendForChannel<Poseidon252MerkleChannel> for VulkanBackend {}
 