use vulkano::{ descriptor_set::WriteDescriptorSet };

use crate::core::{
    backend::{
        Col,
        ColumnOps,
        CpuBackend,
        vulkan::{
            VulkanBackend,
            gpu_context::{ PIPELINE_BLAKE2_S_COMMIT_LAYER },
            shaders::blake2s_commit_layer,
        },
    },
    fields::m31::BaseField,
    vcs::{ blake2_hash::Blake2sHash, blake2_merkle::Blake2sMerkleHasher, ops::MerkleOps },
};

impl ColumnOps<Blake2sHash> for VulkanBackend {
    type Column = Vec<Blake2sHash>;

    fn bit_reverse_column(_column: &mut Self::Column) {
        unimplemented!()
    }
}
impl MerkleOps<Blake2sMerkleHasher> for VulkanBackend {
    fn commit_on_layer(
        log_size: u32,
        prev_layer: Option<&Vec<Blake2sHash>>,
        columns: &[&Col<Self, BaseField>]
    ) -> Vec<Blake2sHash> {
        // println!(
        //     "Vulkan commit_on_layer log_size {} {}  columns len {}",
        //     log_size,
        //     prev_layer.is_none(),
        //     columns.len()
        // );
        if log_size < 4 {
            return <CpuBackend as MerkleOps<Blake2sMerkleHasher>>::commit_on_layer(
                log_size,
                prev_layer,
                &columns
                    .iter()
                    .map(|f| &f.data)
                    .collect::<Vec<_>>()
            );
        }
        let layer_size = 1usize << log_size;
        let num_columns = columns.len();
        // === 1. prev_layer: Vec<u8> ===
        let prev_bytes: Vec<u8> = if let Some(prev) = prev_layer {
            prev.iter()
                .flat_map(|h| h.0)
                .collect() // h.0 is [u8; 32]
        } else {
            vec![]
        };

        // === 2. columns: Vec<u32> ===
        let mut col_u32s = Vec::with_capacity(num_columns * layer_size);
        for col in columns {
            for &field in &col.data {
                col_u32s.push(field.0); // BaseField { 0: u32 }
            }
        }
        let context = VulkanBackend::gpu_context();

        let col_buffer = context.buffer_in(&col_u32s);

        let output_buffer = context.buffer_out::<u8>((layer_size as u64) * 32);
        let prev_buffer = context.buffer_in(&prev_bytes);
        let pipeline = context.pipeline(PIPELINE_BLAKE2_S_COMMIT_LAYER);

        let descriptor_set = context.descriptor_set_writes(
            &pipeline,
            &[
                WriteDescriptorSet::buffer(0, col_buffer.clone()),
                WriteDescriptorSet::buffer(1, output_buffer.clone()),
                WriteDescriptorSet::buffer(2, prev_buffer.clone()),
            ]
        );
        let group_counts = context.group_counts(layer_size);
        // 3. Execute compute shader
        let command_buffer = context.command_buffer_constants(
            &pipeline,
            descriptor_set,
            group_counts,
            blake2s_commit_layer::Params {
                log_size,
                num_columns: num_columns as u32,
            }
        );
        context.execution_wait(command_buffer);
        // 4. Read output back to CPU
        let mapped = output_buffer.read().expect("Failed to read buffer");
        let a: Vec<Blake2sHash> = mapped
            .chunks(32)
            .map(|chunk| Blake2sHash(chunk.try_into().unwrap()))
            .collect();
        a
    }
}

#[cfg(test)]
mod test {
    use blake2::{ Blake2s256, Digest };
    use itertools::Itertools;

    use crate::core::{
        backend::{ CpuBackend, vulkan::{ VulkanBackend, column::VulkanColumn } },
        fields::m31::BaseField,
        vcs::{
            blake2_hash::{ Blake2sHash },
            blake2_merkle::Blake2sMerkleHasher,
            ops::{ MerkleHasher, MerkleOps },
        },
    };
    #[test]
    fn test_blake2s_commit_on_layer() {
        let layer_size = 1;
        let prev_layer = (0..1 << (layer_size + 1))
            .map(|i| Blake2sHash([i as u8; 32]))
            .collect_vec();

        let size = (1 << layer_size) * (1 << layer_size);
        // 步骤 1: 生成所有 BaseField 数据
        let all_data: Vec<BaseField> = (0..size).map(|i| BaseField::from(i as u32)).collect();

        // 步骤 2: 分块并创建 VulkanColumn 向量
        let columns: Vec<VulkanColumn> = all_data
            .chunks(1 << layer_size)
            .map(|chunk| VulkanColumn {
                data: chunk.to_vec(), // 复制一份
            })
            .collect();
        println!("Vulkan commit_on_layer...  {:?}", prev_layer);
        println!("Vulkan commit_on_layer...  {:?}", columns.iter().collect_vec().as_slice());
        let result = VulkanBackend::commit_on_layer(
            layer_size,
            std::option::Option::Some(&prev_layer),
            &columns.iter().collect_vec()
        );

        let c2 = all_data
            .chunks(1 << layer_size)
            .map(|chunk| chunk.to_vec())
            .collect_vec();
        let cpu_result = <CpuBackend as MerkleOps<Blake2sMerkleHasher>>::commit_on_layer(
            layer_size,
            std::option::Option::Some(&prev_layer),
            c2.iter().collect_vec().as_slice()
        );
        assert_eq!(result, cpu_result);
    }

    #[test]
    fn blake2s_hash() {
        let layer_size = 1;
        let prev_layer = (0..1 << (layer_size + 1))
            .map(|i| Blake2sHash([i as u8; 32]))
            .collect_vec();
        println!("prev layer {:?}", prev_layer);
        let a = Blake2sMerkleHasher::hash_node(
            Some((prev_layer[0], prev_layer[1])),
            &[BaseField::from(0u32), BaseField::from(2u32)]
        );

        println!("blake2s hash {:?}", a);
    }

    #[test]
    fn blake2s_hash2() {
        let layer_size = 1;
        let prev_layer = (0..1 << (layer_size + 1))
            .map(|_| Blake2sHash([0 as u8; 32]))
            .collect_vec();
        println!("prev layer {:?}", prev_layer);
        let b = Blake2s256::new();
        // b.update(prev_layer[0].as_ref());
        // b.update(prev_layer[1].as_ref());

        // b.update(&0u32.to_le_bytes());
        // b.update(&2u32.to_le_bytes());
        let c = Blake2sHash(b.finalize().into());

        println!("blake2s hash {:?}", c);
    }
}
