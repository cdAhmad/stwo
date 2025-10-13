use crate::core::{
    backend::{ vulkan::VulkanBackend, ColumnOps },
    fields::m31::BaseField,
    vcs::{ blake2_hash::Blake2sHash, blake2_merkle::Blake2sMerkleHasher, ops::{MerkleHasher, MerkleOps} },
};
use itertools::Itertools;
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
        columns: &[&Vec<BaseField>]
    ) -> Vec<Blake2sHash> {
        (0..1 << log_size)
            .map(|i| {
                Blake2sMerkleHasher::hash_node(
                    prev_layer.map(|prev_layer| (prev_layer[2 * i], prev_layer[2 * i + 1])),
                    &columns
                        .iter()
                        .map(|column| column[i])
                        .collect_vec()
                )
            })
            .collect()
    }
}
