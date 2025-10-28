
use crate::core::{
    backend::vulkan::{
        gpu_context::{ PIPELINE_FRI_FOLD_CIRCLE_INTO_LINE, PIPELINE_FRI_FOLD_LINE },
        shaders,
        VulkanBackend,
    },
    fields::{ m31::{ M31  }, qm31::SecureField, secure_column::SecureColumnByCoords },
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
        dst: &mut LineEvaluation<Self>,
        src: &SecureEvaluation<Self, BitReversedOrder>,
        alpha: SecureField,
        twiddles: &TwiddleTree<Self>
    ) {
        let n: usize = src.len();

        assert!(n >= 2, "Evaluation too small");
        let alpha_sq = alpha * alpha;
        let alpha_sq = alpha_sq.to_m31_array().map(|M31(x)| x);

        let f=VulkanBackend::first_itwiddle_buffer(&twiddles.itwiddles);
        let values = src.values.to_uvec4_vec();
        let dst_values = dst.values.to_uvec4_vec();
        let context = VulkanBackend::gpu_context();
        let pipeline = context.pipeline(PIPELINE_FRI_FOLD_CIRCLE_INTO_LINE);
        let buffer_values = context.buffer_in_out(&values);
        let buffer_dst_values = context.buffer_in_out(&dst_values);
        let buffer_twiddles = context.buffer_in_out(&f);
        let alpha_array = alpha.to_m31_array().map(|M31(x)| x);
        let push_constants = shaders::fri_fold_circle_into_line::PushConstants {
            alpha: alpha_array,
            alpha_sq,
            total_pairs: (n / 2) as u32,
        };
        let descriptor_set = context.descriptor_set(
            &pipeline,
            &[buffer_values.clone(), buffer_dst_values.clone(), buffer_twiddles]
        );
        let group_counts = context.group_counts(n / 2);
        let command_buffer = context.command_buffer_constants(
            &pipeline,
            descriptor_set,
            group_counts,
            push_constants
        );
        context.execution_wait(command_buffer);
        let result = buffer_dst_values.read().expect("Failed to read buffer");
        for (i, chunk) in result.array_chunks().enumerate() {
            let [a, b, c, d] = chunk;
            unsafe {
                dst.values.columns[0].data.get_unchecked_mut(i).0 = *a;
                dst.values.columns[1].data.get_unchecked_mut(i).0 = *b;
                dst.values.columns[2].data.get_unchecked_mut(i).0 = *c;
                dst.values.columns[3].data.get_unchecked_mut(i).0 = *d;
            }
        }
    }

    fn decompose(
        _eval: &SecureEvaluation<Self, BitReversedOrder>
    ) -> (SecureEvaluation<Self, BitReversedOrder>, SecureField) {
        unimplemented!("Decompose is not implemented for VulkanBackend");
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use rand::{ rngs::SmallRng, Rng, SeedableRng };

    use crate::{
        core::{
            backend::{ vulkan::VulkanBackend, CpuBackend },
            fields::{ qm31::SecureField, secure_column::SecureColumnByCoords },
            fri::FriOps,
            poly::{
                circle::{ CanonicCoset, PolyOps, SecureEvaluation },
                line::{ LineDomain, LineEvaluation },
            },
            utils::bit_reverse_index,
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
    #[test]
    fn twiddkles() {
        const LOG_SIZE: u32 = 7;
        let circle_domain = CanonicCoset::new(LOG_SIZE).circle_domain();
        let line_domain = LineDomain::new(circle_domain.half_coset);
        let tw = VulkanBackend::precompute_twiddles(line_domain.coset());
        let tww = VulkanBackend::first_itwiddle_buffer(&tw.itwiddles);

        let b = (0..1 << (LOG_SIZE - 1))
            .enumerate()
            .map(|(index, f)| {
                let t = bit_reverse_index(f << 1, circle_domain.log_size());
                let point = circle_domain.at(t);
                let itwid = point.y.inverse();
                let it2_index = tww.iter().find_position(|x| **x == itwid.0);
                println!(
                    "{index}:  t_i:{} {:?}   {},  ",
                    t,
                    itwid,
                    it2_index.unwrap_or((99, &0)).0
                );
                t
            })
            .collect_vec();
        println!("twiddles2 {} {:?}", b.len(), b);
    }
    #[test]
    fn test_fold_circle_into_line() {
        const LOG_SIZE: u32 = 7;
        let values: Vec<SecureField> = (0..1 << LOG_SIZE)
            .map(|i| qm31!(4 * i, 4 * i + 1, 4 * i + 2, 4 * i + 3))
            .collect();
        let alpha = qm31!(1, 3, 5, 7);
        let circle_domain = CanonicCoset::new(LOG_SIZE).circle_domain();
        let line_domain = LineDomain::new(circle_domain.half_coset);
        let mut cpu_fold = LineEvaluation::new(
            line_domain,
            SecureColumnByCoords::zeros(1 << (LOG_SIZE - 1))
        );
        CpuBackend::fold_circle_into_line(
            &mut cpu_fold,
            &SecureEvaluation::new(circle_domain, values.iter().copied().collect()),
            alpha,
            &CpuBackend::precompute_twiddles(line_domain.coset())
        );

        let mut simd_fold = LineEvaluation::new(
            line_domain,
            SecureColumnByCoords::zeros(1 << (LOG_SIZE - 1))
        );
        VulkanBackend::fold_circle_into_line(
            &mut simd_fold,
            &SecureEvaluation::new(circle_domain, values.iter().copied().collect()),
            alpha,
            &VulkanBackend::precompute_twiddles(line_domain.coset())
        );

        assert_eq!(cpu_fold.values.to_vec(), simd_fold.values.to_vec());
    }
}
