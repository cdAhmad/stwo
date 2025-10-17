use itertools::Itertools;

pub unsafe fn ifft(values: *mut u32, twiddle_dbl: &[&[u32]], log_n_elements: usize) {
    println!("ifft log_n_elements {}", log_n_elements);
    println!("ifft values ptr {:?}", values);
    let twiddle_dbl_len = twiddle_dbl
        .iter()
        .map(|f| f.len())
        .collect_vec();
    println!("ifft twiddle_dbl {:?}   {:?}", twiddle_dbl.len(), twiddle_dbl_len);
    println!("ifft twiddle_dbl {:?}", twiddle_dbl);

    


}

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use rand::{ rngs::SmallRng, Rng, SeedableRng };

    use crate::core::{
        backend::{
            cpu::CpuCircleEvaluation,
            vulkan::{ column::VulkanColumn, fft::{ ifft::ifft, MIN_FFT_LOG_SIZE }, VulkanBackend },
        },
        fields::m31::BaseField,
        poly::{
            circle::{ CanonicCoset, CircleDomain, PolyOps },
            utils::domain_line_twiddles_from_tree,
        },
    };

    #[test]
    fn test_ifft() {
        for log_size in 1..MIN_FFT_LOG_SIZE + 2 {
            let domain = CanonicCoset::new(log_size).circle_domain();
            let mut rng = SmallRng::seed_from_u64(0);
            let values = (0..domain.size()).map(|_| rng.gen()).collect_vec();
            let twiddle_tree = VulkanBackend::precompute_twiddles(domain.half_coset);
            let twiddles = domain_line_twiddles_from_tree(domain, &twiddle_tree.itwiddles);
            let mut res = VulkanColumn::from_iter(values.iter().copied().collect_vec());
            unsafe {
                let v = res.data.as_mut_ptr() as *mut u32;
                ifft(v, &twiddles, log_size as usize);
            }
            ground_truth_ifft(domain, &values);
            // assert_eq!(res.to_cpu(), ground_truth_ifft(domain, &values));
        }
    }

    fn ground_truth_ifft(domain: CircleDomain, values: &[BaseField]) -> Vec<BaseField> {
        let eval = CpuCircleEvaluation::new(domain, values.to_vec());
        let mut res = eval.interpolate().coeffs;
        let denorm = BaseField::from(domain.size());
        res.iter_mut().for_each(|v| {
            *v *= denorm;
        });
        res
    }
}
