
use crate::core::{
    backend::vulkan::{ column::VulkanColumn, gpu_context::PIPELINE_IFFT, shaders, VulkanBackend },
    fields::m31::M31,
};

pub unsafe fn ifft(values: &mut VulkanColumn, twiddle_dbl: &[u32], log_n_elements: usize) {
    let size = values.data.len();
    let u32_slice = unsafe {
        std::slice::from_raw_parts_mut(values.data.as_mut_ptr() as *mut u32, size)
    };
    let context = VulkanBackend::gpu_context();
    let pipeline = context.pipeline(PIPELINE_IFFT);

    let buffer = context.buffer_in_out(u32_slice);
    let twiddle_buffer = context.buffer_in_out(twiddle_dbl);
    let inv_n = M31(size as u32).inverse().0;
    let mut twwidles_offset = [0u32; 32];
    let mut offset = 0;
    for layer in 0..log_n_elements {
        twwidles_offset[layer] = offset;
        // 每层需要 size >> (layer+1) 个因子
        let num = size >> (layer + 1);
        offset += num as u32;
    }
    let push_constants = shaders::ifft::PushConstants {
        log_size: log_n_elements as u32,
        total_layers: log_n_elements as u32,
        total_size: size as u32,
        inv_n: inv_n,
        twiddles_offset: twwidles_offset,
    };

    let descriptor_set = context.descriptor_set(&pipeline, &[buffer.clone(), twiddle_buffer]);

    let group_counts = context.group_counts(size / 2);

    let command_buffer = context.command_buffer_constants(
        &pipeline,
        descriptor_set,
        group_counts,
        push_constants
    );
    context.execution_wait(command_buffer);
    let mapped = buffer.read().expect("Failed to read buffer");
    unsafe {
        std::ptr::copy_nonoverlapping(mapped.as_ptr(), values.data.as_mut_ptr() as *mut u32, size);
    }
}
#[cfg(test)]
mod test {
    use itertools::Itertools;
    use rand::{ rngs::SmallRng, Rng, SeedableRng };

    use crate::core::{
        backend::vulkan::{ column::VulkanColumn, fft::{ ifft::ifft, MIN_FFT_LOG_SIZE }, VulkanBackend },
        poly::circle::{ CanonicCoset, PolyOps,   },
    };

    #[test]
    fn test_ifft() {
        for log_size in 4..MIN_FFT_LOG_SIZE + 2 {
            let domain = CanonicCoset::new(log_size).circle_domain();
            let mut rng = SmallRng::seed_from_u64(0);
            let values = (0..domain.size()).map(|_| rng.gen()).collect_vec();
             let values2 =values.clone();
               assert_eq!(values,values2);
            let twiddle_tree = VulkanBackend::precompute_twiddles(domain.half_coset);
            let mut v = VulkanColumn { data: values };

            unsafe {
                ifft(&mut v, &twiddle_tree.itwiddles, log_size as usize);
            }
           
           
        }
    }

    
}
