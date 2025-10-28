use itertools::Itertools;

use crate::core::{
    backend::{
        cpu::circle::slow_precompute_twiddles,
        vulkan::{
            fft::{ self, rfft, MIN_FFT_LOG_SIZE },
            gpu_context::PIPELINE_BATCH_INVERSE,
            VulkanBackend,
        },
        Col,
        Column,
        CpuBackend,
    },
    circle::{ CirclePoint, Coset },
    fields::{ m31::{ BaseField, M31 }, qm31::SecureField },
    poly::{
        circle::{ CanonicCoset, CircleDomain, CircleEvaluation, CirclePoly, PolyOps },
        twiddles::TwiddleTree,
        utils::fold,
        BitReversedOrder,
    },
};
impl PolyOps for VulkanBackend {
    type Twiddles = Vec<u32>;

    fn eval_at_point(poly: &CirclePoly<Self>, point: CirclePoint<SecureField>) -> SecureField {
        if poly.log_size() == 0 {
            return poly.coeffs.data[0].into();
        }

        let mut mappings = vec![point.y];
        let mut x = point.x;
        for _ in 1..poly.log_size() {
            mappings.push(x);
            x = CirclePoint::double_x(x);
        }
        mappings.reverse();

        fold(&poly.coeffs.data, &mappings)
    }

    fn extend(poly: &CirclePoly<Self>, log_size: u32) -> CirclePoly<Self> {
        // TODO(shahars): Get rid of extends.
        poly.evaluate(CanonicCoset::new(log_size).circle_domain()).interpolate()
    }

    fn evaluate(
        poly: &CirclePoly<Self>,
        domain: CircleDomain,
        twiddles: &TwiddleTree<Self>
    ) -> CircleEvaluation<Self, BaseField, BitReversedOrder> {
        let log_size = domain.log_size();
        let fft_log_size = poly.log_size();
        assert!(log_size >= fft_log_size, "Can only evaluate on larger domains");

        if fft_log_size < MIN_FFT_LOG_SIZE {
            let cpu_poly: CirclePoly<CpuBackend> = CirclePoly::new(poly.coeffs.data.to_cpu());
            let cpu_eval: CircleEvaluation<
                CpuBackend,
                crate::core::fields::m31::M31,
                BitReversedOrder
            > = cpu_poly.evaluate(domain);
            return CircleEvaluation::new(
                cpu_eval.domain,
                Col::<VulkanBackend, BaseField>::from_iter(cpu_eval.values)
            );
        }
        let mut values = poly.coeffs.clone();
        unsafe {
            rfft::fft(&mut values, &twiddles.twiddles, fft_log_size);
        }
        CircleEvaluation::new(domain, values)
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

    fn interpolate(
        eval: CircleEvaluation<Self, BaseField, BitReversedOrder>,
        twiddles: &TwiddleTree<Self>
    ) -> CirclePoly<Self> {
        let log_size = eval.values.data.len().ilog2();
        if log_size < MIN_FFT_LOG_SIZE {
            let cpu_poly = eval.to_cpu().interpolate();
            return CirclePoly::new(cpu_poly.coeffs.into_iter().collect());
        }
        let mut values = eval.values;
        unsafe {
            fft::ifft::ifft(&mut values, &twiddles.itwiddles, log_size);
        }
        CirclePoly::new(values)
    }
}

impl VulkanBackend {
    pub fn first_itwiddle_buffer(itwiddles: &[u32]) -> Vec<u32> {
        let twiddles_size = itwiddles.len();
        let first_twiddle_buffer = itwiddles[..twiddles_size / 2]
            .iter()
            .array_chunks()
            .flat_map(|[&x, &y]| [y, (-M31(y)).0, (-M31(x)).0, x])
            .collect_vec();
        first_twiddle_buffer
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
        backend::{ vulkan::{ fft::MIN_FFT_LOG_SIZE, VulkanBackend }, Column, CpuBackend },
        fields::m31::{ BaseField, M31 },
        poly::{ circle::{ CanonicCoset, CircleEvaluation, PolyOps }, BitReversedOrder },
    };

    #[test]
    fn test_interpolate_and_eval() {
        for log_size in 5..16 {
            let domain = CanonicCoset::new(log_size).circle_domain();
            let evaluation = CircleEvaluation::<VulkanBackend, BaseField, BitReversedOrder>::new(
                domain,
                (0..1 << log_size).map(BaseField::from).collect()
            );
            let a = evaluation.interpolate();

            let a2 = a.evaluate(domain);

            let evaluation2 = CircleEvaluation::<CpuBackend, BaseField, BitReversedOrder>::new(
                domain,
                (0..1 << log_size).map(BaseField::from).collect()
            );

            let b = evaluation2.interpolate();
            let b2 = b.evaluate(domain);
            // assert_eq!(a.coeffs.data.to_vec(), b.coeffs.to_vec());
            assert_ne!(a2.values.to_cpu().to_vec(), b2.values.to_vec());
        }
    }

    #[test]
    fn test_mul() {
        let inv = BaseField::from_u32_unchecked(67108864);
        println!("inv {}", inv);
        let mut a = M31(256);
        a *= inv;
        println!("a {}", a);
    }
    #[test]
    fn test_optimized_precompute_twiddles() {
        for log_size in MIN_FFT_LOG_SIZE + 1..18 {
            let coset = CanonicCoset::new(log_size).half_coset();
            let twiddles = VulkanBackend::precompute_twiddles(coset);
            let expected_twiddles = CpuBackend::precompute_twiddles(coset);
            println!("{} :{}", log_size, twiddles.twiddles.len());
            assert_eq!(
                twiddles.itwiddles,
                expected_twiddles.itwiddles
                    .iter()
                    .map(|x| x.0)
                    .collect::<Vec<u32>>()
            );
        }
    }
}
