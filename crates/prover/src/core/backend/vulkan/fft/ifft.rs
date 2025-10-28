
use crate::core::{
    backend::vulkan::{
        column::VulkanColumn,
        gpu_context::{ PIPELINE_IFFT, PIPELINE_NORMALIZE },
        shaders,
        VulkanBackend,
    },
    fields::m31::M31,
};

pub unsafe fn ifft(values: &mut VulkanColumn, twiddle_dbl: &[u32], log_size: u32) {
    let size = values.data.len();
    let twiddles_size = twiddle_dbl.len();
    let u32_slice = unsafe {
        std::slice::from_raw_parts_mut(values.data.as_mut_ptr() as *mut u32, size)
    };
    let context = VulkanBackend::gpu_context();
    let pipeline = context.pipeline(PIPELINE_IFFT);

    let buffer = context.buffer_in_out(u32_slice);
    let mut twiddle_buffer = Vec::new();

    let first_twiddle_buffer = VulkanBackend::first_itwiddle_buffer(&twiddle_dbl);
    twiddle_buffer.extend(first_twiddle_buffer);
    //   let inv = BaseField::from_u32_unchecked(eval.domain.size() as u32).inverse();
    let inv_n = M31(size as u32).inverse().0;
    let mut twwidles_offset = [0u32; 32];
    let mut offset = twiddle_buffer.len() as u32;
    twiddle_buffer.extend(twiddle_dbl.iter());
    for layer in 0..log_size {
        let layer_idx = layer + 1;
        twwidles_offset[layer_idx as usize] = offset;
        // 每层需要 size >> (layer+1) 个因子
        let num = twiddles_size >> (layer + 1);
        offset += num as u32;
    }
    let twiddle_buffer = context.buffer_in_out(&twiddle_buffer);

    for log_n in 0..log_size {
        let push_constants = shaders::ifft::PushConstants {
            log_n: log_n,
            total_size: size as u32,
            twiddles_offset: twwidles_offset[log_n as usize],
        };

        let descriptor_set = context.descriptor_set(
            &pipeline,
            &[buffer.clone(), twiddle_buffer.clone()]
        );
        let group_counts = context.group_counts(size/2);
        let command_buffer = context.command_buffer_constants(
            &pipeline,
            descriptor_set,
            group_counts,
            push_constants
        );
        context.execution_wait(command_buffer);
    }
    //normalize
    let pipeline = context.pipeline(PIPELINE_NORMALIZE);
    let push_constants = shaders::normalize::PushConstants {
        total_size: size as u32,
        inv_n: inv_n,
    };

    let descriptor_set = context.descriptor_set(&pipeline, &[buffer.clone()]);
    let group_counts = context.group_counts(size);
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
        backend::vulkan::{
            column::VulkanColumn,
            fft::{ ifft::ifft, MIN_FFT_LOG_SIZE },
            VulkanBackend,
        },
        poly::circle::{ CanonicCoset, PolyOps },
    };

    #[test]
    fn test_ifft() {
        for log_size in 4..MIN_FFT_LOG_SIZE + 2 {
            let domain = CanonicCoset::new(log_size).circle_domain();
            let mut rng = SmallRng::seed_from_u64(0);
            let values = (0..domain.size()).map(|_| rng.gen()).collect_vec();
            let values2 = values.clone();
            assert_eq!(values, values2);
            let twiddle_tree = VulkanBackend::precompute_twiddles(domain.half_coset);
            let mut v = VulkanColumn { data: values };

            unsafe {
                ifft(&mut v, &twiddle_tree.itwiddles, log_size);
            }
        }
    }
}
