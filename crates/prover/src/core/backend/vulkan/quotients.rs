use crate::core::{
    backend::vulkan::VulkanBackend,
    fields::{ m31::BaseField, qm31::SecureField },
    pcs::quotients::{ ColumnSampleBatch, QuotientOps },
    poly::{ circle::{ CircleDomain, CircleEvaluation, SecureEvaluation }, BitReversedOrder },
};

impl QuotientOps for VulkanBackend {
    fn accumulate_quotients(
        _domain: CircleDomain,
        _columns: &[&CircleEvaluation<Self, BaseField, BitReversedOrder>],
        _random_coeff: SecureField,
        _sample_batches: &[ColumnSampleBatch],
        _log_blowup_factor: u32
    ) -> SecureEvaluation<Self, BitReversedOrder> {
         todo!()
        
    }
}
