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
        _log_size: u32,
        _prev_layer: Option<
            &crate::core::backend::Col<Self, <Blake2sMerkleHasher as MerkleHasher>::Hash>
        >,
        _columns: &[&crate::core::backend::Col<Self, BaseField>]
    ) -> crate::core::backend::Col<Self, <Blake2sMerkleHasher as MerkleHasher>::Hash> {
        todo!()
    }
}
