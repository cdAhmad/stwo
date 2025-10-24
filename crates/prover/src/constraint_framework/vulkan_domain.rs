use std::ops::Mul;

use num_traits::Zero;

use super::logup::LogupAtRow;
use super::{ EvalAtRow, INTERACTION_TRACE_IDX };
use crate::core::backend::vulkan::VulkanBackend;
use crate::core::fields::m31::BaseField;
use crate::core::fields::qm31::SecureField;
use crate::core::fields::secure_column::SECURE_EXTENSION_DEGREE;
use crate::core::lookups::utils::Fraction;
use crate::core::pcs::TreeVec;
use crate::core::poly::circle::CircleEvaluation;
use crate::core::poly::BitReversedOrder;
use crate::core::utils::offset_bit_reversed_circle_domain_index;

/// Evaluates constraints at an evaluation domain points.
pub struct VulkanDomainEvaluator<'a> {
    pub trace_eval: &'a TreeVec<
        Vec<&'a CircleEvaluation<VulkanBackend, BaseField, BitReversedOrder>>
    >,
    pub column_index_per_interaction: Vec<usize>,
    /// The row index of the simd-vector row to evaluate the constraints at.
    pub row: usize,
    pub random_coeff_powers: &'a [SecureField],
    pub row_res: SecureField,
    pub constraint_index: usize,
    pub domain_log_size: u32,
    pub eval_domain_log_size: u32,
    pub logup: LogupAtRow<Self>,
}
impl<'a> VulkanDomainEvaluator<'a> {
    #[allow(dead_code)]
    pub fn new(
        trace_eval: &'a TreeVec<Vec<&CircleEvaluation<VulkanBackend, BaseField, BitReversedOrder>>>,
        row: usize,
        random_coeff_powers: &'a [SecureField],
        domain_log_size: u32,
        eval_log_size: u32,
        log_size: u32,
        claimed_sum: SecureField
    ) -> Self {
        Self {
            trace_eval,
            column_index_per_interaction: vec![0; trace_eval.len()],
            row,
            random_coeff_powers,
            row_res: SecureField::zero(),
            constraint_index: 0,
            domain_log_size,
            eval_domain_log_size: eval_log_size,
            logup: LogupAtRow::new(INTERACTION_TRACE_IDX, claimed_sum, log_size),
        }
    }
}
impl EvalAtRow for VulkanDomainEvaluator<'_> {
    type F = BaseField;
    type EF = SecureField;

    // TODO(Ohad): Add debug boundary checks.
    fn next_interaction_mask<const N: usize>(
        &mut self,
        interaction: usize,
        offsets: [isize; N]
    ) -> [Self::F; N] {
        let col_index = self.column_index_per_interaction[interaction];
        self.column_index_per_interaction[interaction] += 1;
        offsets.map(|off| {
            // If the offset is 0, we can just return the value directly from this row.
            if off == 0 {
                unsafe {
                    let col = &self.trace_eval
                        .get_unchecked(interaction)
                        .get_unchecked(col_index).values;
                    return col.data[self.row];
                }
            }
            // Otherwise, we need to look up the value at the offset.
            // Since the domain is bit-reversed circle domain ordered, we need to look up the value
            // at the bit-reversed natural order index at an offset.
            let row = offset_bit_reversed_circle_domain_index(
                self.row,
                self.domain_log_size,
                self.eval_domain_log_size,
                off
            );
            self.trace_eval[interaction][col_index].to_cpu()[row]
        })
    }
    fn add_constraint<G>(&mut self, constraint: G)
        where Self::EF: Mul<G, Output = Self::EF> + From<G>
    {
        self.row_res += self.random_coeff_powers[self.constraint_index] * constraint;
        self.constraint_index += 1;
    }

    fn combine_ef(values: [Self::F; SECURE_EXTENSION_DEGREE]) -> Self::EF {
        SecureField::from_m31_array(values)
    }

    super::logup_proxy!();
}
