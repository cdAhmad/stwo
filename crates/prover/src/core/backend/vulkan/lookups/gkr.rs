
use crate::core::backend::vulkan::VulkanBackend;
use crate::core::fields::qm31::SecureField;
use crate::core::lookups::gkr_prover::{
    GkrMultivariatePolyOracle,
    GkrOps,
    Layer,
};
use crate::core::lookups::mle::Mle;
use crate::core::lookups::utils::UnivariatePoly;

impl GkrOps for VulkanBackend {
    fn gen_eq_evals(_y: &[SecureField], _v: SecureField) -> Mle<Self, SecureField> {
        unimplemented!("gen_eq_evals is not implemented for BaseField")
    }

    fn next_layer(_layer: &Layer<Self>) -> Layer<Self> {
        unimplemented!("next_layer is not implemented for BaseField")
    }

    fn sum_as_poly_in_first_variable(
        _h: &GkrMultivariatePolyOracle<'_, Self>,
        _claim: SecureField
    ) -> UnivariatePoly<SecureField> {
        unimplemented!("sum_as_poly_in_first_variable is not implemented for BaseField")
    }
}
