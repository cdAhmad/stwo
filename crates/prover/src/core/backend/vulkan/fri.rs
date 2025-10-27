use crate::core::{
    backend::vulkan::{gpu_context::PIPELINE_FRI_FOLD_LINE, shaders, VulkanBackend},
    fields::{m31::M31, qm31::SecureField, secure_column::SecureColumnByCoords},
    fri::FriOps,
    poly::{
        circle::SecureEvaluation,
        line::LineEvaluation,
        twiddles::TwiddleTree,
        BitReversedOrder,
    },
};

impl FriOps for VulkanBackend {
    fn fold_line(
        eval: &LineEvaluation<Self>,
        alpha: SecureField,
        twiddles: &TwiddleTree<Self>
    ) -> LineEvaluation<Self> {
        let n = eval.len();

        assert!(n >= 2, "Evaluation too small");

        let domain = eval.domain();
        let itwiddles = &twiddles.itwiddles[0..twiddles.itwiddles.len() >> 1];
        let alpha = alpha.to_m31_array().map(|M31(x)| x);
        let values = eval.values.to_uvec4_vec();
        let context = VulkanBackend::gpu_context();
        let pipeline = context.pipeline(PIPELINE_FRI_FOLD_LINE);
        let buffer_values = context.buffer_in_out(&values);
        let buffer_twiddles = context.buffer_in_out(itwiddles);
        let push_constants = shaders::fri_fold_line::PushConstants {
            alpha,
            total_pairs: (n / 2) as u32,
        };
        let descriptor_set = context.descriptor_set(
            &pipeline,
            &[buffer_values.clone(), buffer_twiddles]
        );
        let group_counts = context.group_counts(n >> 1);
        let command_buffer = context.command_buffer_constants(
            &pipeline,
            descriptor_set,
            group_counts,
            push_constants
        );
        context.execution_wait(command_buffer);
        let result = buffer_values.read().expect("Failed to read buffer");
        let mut folded_values = SecureColumnByCoords::<Self>::zeros(n >> 1);
        folded_values.copy_from_slice_vec4(&result, n >> 1);

        LineEvaluation::new(domain.double(), folded_values)
    }

    fn fold_circle_into_line(
        _dst: &mut LineEvaluation<Self>,
        _src: &SecureEvaluation<Self, BitReversedOrder>,
        _alpha: SecureField,
        _twiddles: &TwiddleTree<Self>
    ) {
        todo!()
    }

    fn decompose(
        _eval: &SecureEvaluation<Self, BitReversedOrder>
    ) -> (SecureEvaluation<Self, BitReversedOrder>, SecureField) {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use rand::{ rngs::SmallRng, Rng, SeedableRng };

    use crate::{
        core::{
            backend::{ vulkan::VulkanBackend, CpuBackend },
            fri::FriOps,
            poly::{ circle::{ CanonicCoset, PolyOps }, line::{ LineDomain, LineEvaluation } },
        },
        qm31,
    };

    #[test]
    fn test_fold_line() {
        const LOG_SIZE: u32 = 7;
        let mut rng = SmallRng::seed_from_u64(0);
        let values = (0..1 << LOG_SIZE).map(|_| rng.gen()).collect_vec();
        let alpha = qm31!(1, 3, 5, 7);
        let domain = LineDomain::new(CanonicCoset::new(LOG_SIZE + 1).half_coset());
        let cpu_fold = CpuBackend::fold_line(
            &LineEvaluation::new(domain, values.iter().copied().collect()),
            alpha,
            &CpuBackend::precompute_twiddles(domain.coset())
        );

        let avx_fold: LineEvaluation<VulkanBackend> = VulkanBackend::fold_line(
            &LineEvaluation::new(domain, values.iter().copied().collect()),
            alpha,
            &VulkanBackend::precompute_twiddles(domain.coset())
        );

        assert_eq!(
            cpu_fold.values.to_vec(),
            avx_fold.values
                .to_uvec4()
                .iter()
                .map(|f| qm31!(f[0], f[1], f[2], f[3]))
                .collect_vec()
        );
    }
}
