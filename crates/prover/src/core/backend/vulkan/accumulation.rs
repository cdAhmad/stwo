// use num_traits::One;
// use vulkano::buffer::{ Buffer, BufferCreateInfo, BufferUsage };
// use vulkano::memory::allocator::{ AllocationCreateInfo, MemoryTypeFilter };

// use crate::core::air::accumulation::AccumulationOps;
// use crate::core::backend::vulkan::{ shaders, VulkanBackend };
// use crate::core::fields::qm31::SecureField;
// use crate::core::fields::secure_column::SecureColumnByCoords;

// impl AccumulationOps for VulkanBackend {
//     fn accumulate(column: &mut SecureColumnByCoords<Self>, other: &SecureColumnByCoords<Self>) {
//         // Initialize Vulkan context
//         let (device, queue, pipeline, memory_allocator) = Self::device_queue_pipeline_allocator(
//             |d| { shaders::accumulate::load(d).expect("Failed to create shader module") }
//         );

//         // Single in-place buffer
//         let buffer = Buffer::from_iter(
//             memory_allocator.clone(),
//             BufferCreateInfo {
//                 usage: BufferUsage::STORAGE_BUFFER |
//                 BufferUsage::TRANSFER_SRC |
//                 BufferUsage::TRANSFER_DST,
//                 ..Default::default()
//             },
//             AllocationCreateInfo {
//                 memory_type_filter: MemoryTypeFilter::PREFER_HOST |
//                 MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
//                 ..Default::default()
//             },
//             column.iter().cloned()
//         ).expect("Failed to create buffer");
//         todo!()
//     }

//     fn generate_secure_powers(felt: SecureField, n_powers: usize) -> Vec<SecureField> {
//         (0..n_powers)
//             .scan(SecureField::one(), |acc, _| {
//                 let res = *acc;
//                 *acc *= felt;
//                 Some(res)
//             })
//             .collect()
//     }
// }

// #[cfg(test)]
// mod tests {
//     use num_traits::One;

//     use crate::core::air::accumulation::AccumulationOps;
//     use crate::core::backend::vulkan::VulkanBackend;
//     use crate::core::backend::CpuBackend;
//     use crate::core::fields::qm31::SecureField;
//     use crate::core::fields::FieldExpOps;
//     use crate::qm31;
//     #[test]
//     fn generate_secure_powers_works() {
//         let felt = qm31!(1, 2, 3, 4);
//         let n_powers: usize = 10;

//         let powers = <VulkanBackend as AccumulationOps>::generate_secure_powers(felt, n_powers);

//         assert_eq!(powers.len(), n_powers);
//         assert_eq!(powers[0], SecureField::one());
//         assert_eq!(powers[1], felt);
//         assert_eq!(powers[7], felt.pow(7));
//     }

//     #[test]
//     fn generate_empty_secure_powers_works() {
//         let felt = qm31!(1, 2, 3, 4);
//         let max_log_size = 0;

//         let powers = <CpuBackend as AccumulationOps>::generate_secure_powers(felt, max_log_size);

//         assert_eq!(powers, vec![]);
//     }
// }
