pub unsafe fn ifft(values: *mut u32, twiddle_dbl: &[&[u32]], log_n_elements: usize) {
    
   println!("ifft {}", log_n_elements);
   println!("ifft {:?}",  values);
   println!("ifft {:?}",  twiddle_dbl);



}


#[cfg(test)]
mod test{
    use std::mem::transmute;

    use itertools::Itertools;
    use rand::{rngs::SmallRng, Rng, SeedableRng};

    use crate::core::{backend::{cpu::CpuCircleEvaluation, simd::{column::BaseColumn, fft::{ifft::get_itwiddle_dbls, transpose_vecs}, m31::PackedBaseField}, vulkan::fft::{ifft::ifft, CACHED_FFT_LOG_SIZE}, Column}, fields::m31::BaseField, poly::circle::{CanonicCoset, CircleDomain}};


     #[test]
    fn test_ifft() {
        for log_size in CACHED_FFT_LOG_SIZE + 1..CACHED_FFT_LOG_SIZE + 3 {
            let domain = CanonicCoset::new(log_size).circle_domain();
            let mut rng = SmallRng::seed_from_u64(0);
            let values = (0..domain.size()).map(|_| rng.gen()).collect_vec();
            let twiddle_dbls = get_itwiddle_dbls(domain.half_coset);

            let mut res = values.iter().copied().collect::<BaseColumn>();
            unsafe {
                ifft(
                    transmute::<*mut PackedBaseField, *mut u32>(res.data.as_mut_ptr()),
                    &twiddle_dbls.iter().map(|x| x.as_slice()).collect_vec(),
                    log_size as usize,
                );
                transpose_vecs(
                    transmute::<*mut PackedBaseField, *mut u32>(res.data.as_mut_ptr()),
                    log_size as usize - 4,
                );
            }

            assert_eq!(res.to_cpu(), ground_truth_ifft(domain, &values));
        }
    }

    fn ground_truth_ifft(domain: CircleDomain, values: &[BaseField]) -> Vec<BaseField> {
        let eval = CpuCircleEvaluation::new(domain, values.to_vec());
        let mut res = eval.interpolate().coeffs;
        let denorm = BaseField::from(domain.size());
        res.iter_mut().for_each(|v| *v *= denorm);
        res
    }

}