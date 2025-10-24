use itertools::Itertools;

use crate::core::{
    backend::{ vulkan::VulkanBackend, ColumnOps },
    fields::m31::BaseField,
    vcs::{
        blake2_hash::Blake2sHash,
        blake2_merkle::Blake2sMerkleHasher,
        ops::{ MerkleHasher, MerkleOps },
    },
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
        prev_layer: Option<
            &crate::core::backend::Col<Self, <Blake2sMerkleHasher as MerkleHasher>::Hash>
        >,
        columns: &[&crate::core::backend::Col<Self, BaseField>]
    ) -> crate::core::backend::Col<Self, <Blake2sMerkleHasher as MerkleHasher>::Hash> {
       (0..(1 << log_size))
            .map(|i| {
                Blake2sMerkleHasher::hash_node(
                    prev_layer.map(|prev_layer| (prev_layer[2 * i], prev_layer[2 * i + 1])),
                    &columns.iter().map(|column| column.data[i]).collect_vec(),
                )
            })
            .collect()
    }
}
