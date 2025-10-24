use crate::core::{
    backend::vulkan::VulkanBackend,
    fields::qm31::SecureField,
    fri::{ fold_circle_into_line, FriOps },
    poly::{
        circle::SecureEvaluation,
        line::LineEvaluation,
        twiddles::TwiddleTree,
        BitReversedOrder,
    },
};

impl FriOps for VulkanBackend {
    fn fold_line(
        _eval: &LineEvaluation<Self>,
        _alpha: SecureField,
        _twiddles: &TwiddleTree<Self>
    ) -> LineEvaluation<Self> {
        todo!()
    }

    fn fold_circle_into_line(
        _dst: &mut LineEvaluation<Self>,
        _src: &SecureEvaluation<Self, BitReversedOrder>,
        _alpha: SecureField,
        _twiddles: &TwiddleTree<Self>
    ) {
        todo!()
    }

    fn decompose(
        _eval: &SecureEvaluation<Self, BitReversedOrder>
    ) -> (SecureEvaluation<Self, BitReversedOrder>, SecureField) {
        todo!()
    }
}
