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

#[cfg(test)]
mod text {
    use itertools::Itertools;

    use crate::core::{
        backend::{ simd::{ column::BaseColumn, SimdBackend }, Column, CpuBackend },
        circle::SECURE_FIELD_CIRCLE_GEN,
        fields::m31::BaseField,
        pcs::quotients::{ ColumnSampleBatch, QuotientOps },
        poly::{ circle::{ CanonicCoset, CircleEvaluation }, BitReversedOrder },
    };
    use crate::qm31;
    #[test]
    fn test_quotient_ops() {
        const LOG_SIZE: u32 = 8;
        const LOG_BLOWUP_FACTOR: u32 = 1;
        let small_domain = CanonicCoset::new(LOG_SIZE).circle_domain();
        let domain = CanonicCoset::new(LOG_SIZE + LOG_BLOWUP_FACTOR).circle_domain();
        let e0: BaseColumn = (0..small_domain.size()).map(BaseField::from).collect();
        let e1: BaseColumn = (0..small_domain.size()).map(|i| BaseField::from(2 * i)).collect();
        let polys = [
            CircleEvaluation::<SimdBackend, BaseField, BitReversedOrder>
                ::new(small_domain, e0)
                .interpolate(),
            CircleEvaluation::<SimdBackend, BaseField, BitReversedOrder>
                ::new(small_domain, e1)
                .interpolate(),
        ];
        let columns = [polys[0].evaluate(domain), polys[1].evaluate(domain)];
        let random_coeff = qm31!(1, 2, 3, 4);
        let a = polys[0].eval_at_point(SECURE_FIELD_CIRCLE_GEN);
        let b = polys[1].eval_at_point(SECURE_FIELD_CIRCLE_GEN);
        let samples = vec![ColumnSampleBatch {
            point: SECURE_FIELD_CIRCLE_GEN,
            columns_and_values: vec![(0, a), (1, b)],
        }];
        let cpu_columns = columns
            .iter()
            .map(|c| CircleEvaluation::new(c.domain, c.values.to_cpu()))
            .collect_vec();
        let cpu_result = CpuBackend::accumulate_quotients(
            domain,
            &cpu_columns.iter().collect_vec(),
            random_coeff,
            &samples,
            LOG_BLOWUP_FACTOR
        ).values.to_vec();

        let res = SimdBackend::accumulate_quotients(
            domain,
            &columns.iter().collect_vec(),
            random_coeff,
            &samples,
            LOG_BLOWUP_FACTOR
        ).values.to_vec();

        assert_eq!(res, cpu_result);
    }
}
