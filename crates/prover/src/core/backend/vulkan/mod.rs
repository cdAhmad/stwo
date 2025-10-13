use serde::{ Deserialize, Serialize };

mod accumulation;
mod shaders;
mod m31;
mod column;
use std::{ fmt::Debug, sync::Arc };

use crate::core::{ backend::{ vulkan::gpu_context::{ GpuContext, GPU_CONTEXT } } };

mod circle;
mod gpu_context;
mod blake2s;
mod grind;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct VulkanBackend;

impl VulkanBackend {
    pub fn gpu_context() -> &'static Arc<GpuContext> {
        GPU_CONTEXT.get_or_init(GpuContext::new)
    }
}
// impl Backend for VulkanBackend {}
// impl BackendForChannel<Blake2sMerkleChannel> for VulkanBackend {}
// #[cfg(not(target_arch = "wasm32"))]
// impl BackendForChannel<Poseidon252MerkleChannel> for VulkanBackend {}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use crate::core::backend::vulkan::VulkanBackend;
    use crate::core::backend::{ ColumnOps };
    use crate::core::fields::m31::BaseField;
    use crate::core::backend::cpu::bit_reverse as cpu_bit_reverse;

    #[test]
    fn bit_reverse_large_column_works() {
        const LOG_SIZE: u32 = 22;
        let column = (0..1 << LOG_SIZE).map(BaseField::from).collect_vec();
        let mut expected = column.clone();
        cpu_bit_reverse(&mut expected);
        let mut res = column.clone();
        <VulkanBackend as ColumnOps<BaseField>>::bit_reverse_column(&mut res);
        assert_eq!(res, expected);
    }
}
