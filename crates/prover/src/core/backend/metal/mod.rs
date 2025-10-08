pub mod circle;
use std::{ fmt::Debug, mem };
use std::ffi::c_void;
use metal::{ CompileOptions, Device,   };
use serde::{ Deserialize, Serialize };
use crate::core::backend::{ cpu::bit_reverse as cpu_bit_reverse, ColumnOps };
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct MetalBackend;
impl MetalBackend {
    pub fn device_and_queue() -> (Device, metal::CommandQueue) {
        let device = Device::system_default().expect("No device found");
        let queue = device.new_command_queue();
        (device, queue)
    }
    pub fn load_library(device: &Device, data: &[u8]) -> metal::Library {
        device.new_library_with_data(data).expect("Failed to compile Metal shader")
    }
    pub fn load_library_with_path(device: &Device, path: &str) -> metal::Library {
        device
            .new_library_with_source(path, &CompileOptions::new())
            .expect("Failed to compile Metal shader")
    }

    pub fn create_pipeline(
        device: &Device,
        library: &metal::Library,
        name: &str
    ) -> metal::ComputePipelineState {
        let kernel = library
            .get_function(name, None)
            .expect(&format!("Failed to find kernel {}", name));
        device
            .new_compute_pipeline_state_with_function(&kernel)
            .expect(&format!("Failed to create pipeline for kernel {}", name))
    }
}

impl<T: Debug + Clone + Default> ColumnOps<T> for MetalBackend {
    type Column = Vec<T>;

    fn bit_reverse_column(column: &mut Self::Column) {
        let n = column.len();
        assert!(n.is_power_of_two());
        let log_n = n.ilog2();
        if n < 1 << 12 {
            cpu_bit_reverse(column);
            return;
        }
        let buffer_size = std::mem::size_of::<T>() * n;
        let (device, queue) = Self::device_and_queue();
        let library = Self::load_library(&device, include_bytes!("shaders/bit_reverse.metallib"));
        // let library = Self::load_library_with_path(
        //     &device,
        //     include_str!("shaders/bit_reverse.metal")
        // );
        let pipeline = Self::create_pipeline(&device, &library, "bit_reverse_single");
        // 创建 input buffer
        let buffer = device.new_buffer_with_data(
            column.as_ptr() as *const _,
            buffer_size as u64,
            metal::MTLResourceOptions::StorageModeShared
        );

        // 创建 log_n buffer
        let log_n_buffer = device.new_buffer_with_data(
            &log_n as *const u32 as *const c_void,
            mem::size_of::<u32>() as u64,
            metal::MTLResourceOptions::StorageModeShared
        );

        let command_buffer = queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&pipeline);
        encoder.set_buffer(0, Some(&buffer), 0);
        encoder.set_buffer(1, Some(&log_n_buffer), 0);

        let thread_group_size = pipeline.thread_execution_width();
        let thread_group_count =
            ((n as u64) + (thread_group_size as u64) - 1) / (thread_group_size as u64);

        
        encoder.dispatch_thread_groups(
            metal::MTLSize {
                width: thread_group_count as u64,
                height: 1,
                depth: 1,
            },
            metal::MTLSize {
                width: thread_group_size as u64,
                height: 1,
                depth: 1,
            }
        );
        encoder.end_encoding();

        // 提交并等待
        command_buffer.commit();
        command_buffer.wait_until_completed();
        // 从 GPU 读回结果
        unsafe {
            let output_ptr = buffer.contents() as *const T;
            std::ptr::copy_nonoverlapping(output_ptr, column.as_mut_ptr(), n as usize);
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::core::backend::metal::MetalBackend;
    use crate::core::backend::ColumnOps;
    use crate::core::fields::m31::BaseField;

    use crate::core::backend::cpu::bit_reverse as cpu_bit_reverse;

    #[test]
    fn bit_reverse_large_column_works() {
        const LOG_SIZE: u32 = 22;
        let column = (0..1 << LOG_SIZE).map(BaseField::from).collect_vec();
        let mut expected = column.clone();
        cpu_bit_reverse(&mut expected);

        let mut res = column.clone();
        MetalBackend::bit_reverse_column(&mut res);

        assert_eq!(res, expected);
    }
}
