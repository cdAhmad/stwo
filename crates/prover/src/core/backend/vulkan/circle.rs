// use crate::core::{
//     backend::vulkan::VulkanBackend,
//     circle::{ CirclePoint, Coset },
//     fields::{ m31::BaseField, qm31::SecureField },
//     poly::{
//         circle::{ CircleDomain, CircleEvaluation, CirclePoly, PolyOps },
//         twiddles::TwiddleTree,
//         BitReversedOrder,
//     },
// };
// pub const MIN_FFT_LOG_SIZE: u32 = 5;
// impl PolyOps for VulkanBackend {
//     type Twiddles = Vec<u32>;

//     fn interpolate(
//         eval: CircleEvaluation<Self, BaseField, BitReversedOrder>,
//         itwiddles: &TwiddleTree<Self>
//     ) -> CirclePoly<Self> {
//          let log_size = eval.values.length.ilog2();
//         // if log_size < MIN_FFT_LOG_SIZE {
//         //     let cpu_poly = eval.to_cpu().interpolate();
//         //     return CirclePoly::new(cpu_poly.coeffs.into_iter().collect());
//         // }
//         todo!()
//     }

//     fn eval_at_point(poly: &CirclePoly<Self>, point: CirclePoint<SecureField>) -> SecureField {
//         todo!()
//     }

//     fn extend(poly: &CirclePoly<Self>, log_size: u32) -> CirclePoly<Self> {
//         todo!()
//     }

//     fn evaluate(
//         poly: &CirclePoly<Self>,
//         domain: CircleDomain,
//         twiddles: &TwiddleTree<Self>
//     ) -> CircleEvaluation<Self, BaseField, BitReversedOrder> {
//         todo!()
//     }

//     fn precompute_twiddles(coset: Coset) -> TwiddleTree<Self> {
//         todo!()
//     }
// }
