use crate::core::backend::vulkan::VulkanBackend;
use crate::core::fields::m31::BaseField;
use crate::core::fields::qm31::SecureField;
use crate::core::lookups::mle::{ Mle, MleOps };

impl MleOps<BaseField> for VulkanBackend {
    fn fix_first_variable(
        _mle: Mle<Self, BaseField>,
        _assignment: SecureField
    ) -> Mle<Self, SecureField> {
        unimplemented!("fix_first_variable is not implemented for BaseField")
    }
}

impl MleOps<SecureField> for VulkanBackend {
    fn fix_first_variable(
        _mle: Mle<Self, SecureField>,
        _assignment: SecureField
    ) -> Mle<Self, SecureField> {
        unimplemented!("fix_first_variable is not implemented for BaseField")
    }
}

// impl MultivariatePolyOracle for Mle<VulkanBackend, SecureField> {
//     fn n_variables(&self) -> usize {
//         self.n_variables()
//     }

//     fn sum_as_poly_in_first_variable(&self, _claim: SecureField) -> UnivariatePoly<SecureField> {
//         unimplemented!("fix_first_variable is not implemented for BaseField")
//     }

//     fn fix_first_variable(self, challenge: SecureField) -> Self {
//         self.fix_first_variable(challenge)
//     }
// }
