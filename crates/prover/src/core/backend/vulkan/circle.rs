use crate::core::{
    backend::{
        cpu::circle::slow_precompute_twiddles,
        vulkan::{ fft, gpu_context::PIPELINE_BATCH_INVERSE, VulkanBackend },
    },
    circle::{ CirclePoint, Coset },
    fields::{ m31::BaseField, qm31::SecureField },
    poly::{
        circle::{ CircleDomain, CircleEvaluation, CirclePoly, PolyOps },
        twiddles::TwiddleTree,
        utils::domain_line_twiddles_from_tree,
        BitReversedOrder,
    },
};
impl PolyOps for VulkanBackend {
    type Twiddles = Vec<u32>;

    fn interpolate(
        eval: CircleEvaluation<Self, BaseField, BitReversedOrder>,
        twiddles: &TwiddleTree<Self>
    ) -> CirclePoly<Self> {
        let log_size = eval.values.data.len().ilog2();
        if log_size < 5 {
            let cpu_poly = eval.to_cpu().interpolate();
            return CirclePoly::new(cpu_poly.coeffs.into_iter().collect());
        }
        let mut values = eval.values;
        let twiddles = domain_line_twiddles_from_tree(eval.domain, &twiddles.itwiddles);
        unsafe {
            fft::ifft::ifft( values.data.as_mut_ptr() as *mut u32, &twiddles, log_size as usize);
        }

        todo!()
    }

    fn eval_at_point(_: &CirclePoly<Self>, _: CirclePoint<SecureField>) -> SecureField {
        // poly.eval_at_point(point);
        todo!()
    }

    fn extend(_: &CirclePoly<Self>, _: u32) -> CirclePoly<Self> {
        unimplemented!()
    }

    fn evaluate(
        _: &CirclePoly<Self>,
        _: CircleDomain,
        _: &TwiddleTree<Self>
    ) -> CircleEvaluation<Self, BaseField, BitReversedOrder> {
        unimplemented!()
    }

    fn precompute_twiddles(coset: Coset) -> TwiddleTree<Self> {
        const CHUNK_LOG_SIZE: usize = 12;
        const CHUNK_SIZE: usize = 1 << CHUNK_LOG_SIZE;

        let root_coset = coset;
        let twiddles = slow_precompute_twiddles(coset); // CPU: 正向根

        if CHUNK_SIZE > root_coset.size() {
            // 小域：纯 CPU
            let itwiddles: Vec<u32> = twiddles
                .iter()
                .map(|&t| t.inverse().0)
                .collect();
            let twiddles: Vec<u32> = twiddles
                .iter()
                .map(|&t| t.0)
                .collect();
            return TwiddleTree {
                root_coset,
                twiddles,
                itwiddles,
            };
        }
        let twiddles: Vec<u32> = twiddles
            .iter()
            .map(|&t| t.0)
            .collect();
        // GPU 加速批量求逆
        let itwiddles = gpu_batch_inverse(&twiddles);
        TwiddleTree {
            root_coset,
            twiddles,
            itwiddles,
        }
    }
}
fn gpu_batch_inverse(twiddles: &Vec<u32>) -> Vec<u32> {
    let context = VulkanBackend::gpu_context();
    let len = twiddles.len();
    // 1. 创建输入缓冲区
    let buffer = context.buffer_in_out(twiddles);
    let pipeline = context.pipeline(PIPELINE_BATCH_INVERSE);

    let descriptor_set = context.descriptor_set(&pipeline, &[buffer.clone()]);
    let group_counts = context.group_counts(len);
    // 创建命令缓冲区
    let command_buffer = context.command_buffer(&pipeline, descriptor_set, group_counts, &[]);
    context.execution_wait(command_buffer);
    let mapped = buffer.read().expect("Failed to read buffer");
    mapped.to_vec()
}

#[cfg(test)]
mod test {
    use crate::core::{
        backend::{ vulkan::VulkanBackend, CpuBackend },
        poly::circle::{ CanonicCoset, PolyOps },
    };
    #[test]
    fn test_optimized_precompute_twiddles() {
        let coset = CanonicCoset::new(14).half_coset();
        let twiddles = VulkanBackend::precompute_twiddles(coset);
        let expected_twiddles = CpuBackend::precompute_twiddles(coset);
        assert_eq!(
            twiddles.twiddles,
            expected_twiddles.twiddles
                .iter()
                .map(|x| x.0)
                .collect::<Vec<u32>>()
        );
        assert_eq!(
            twiddles.itwiddles,
            expected_twiddles.itwiddles
                .iter()
                .map(|x| x.0)
                .collect::<Vec<u32>>()
        );
    }
}
