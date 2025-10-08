// use crate::core::{backend::metal::MetalBackend, circle::CirclePoint, fields::m31::BaseField, poly::circle::CirclePoly};

// impl PolyOps for MetalBackend { 
//     type Twiddles = Vec<BaseField>;

//     fn eval_at_point(poly: &CirclePoly<Self>, point: CirclePoint<SecureField>) -> SecureField {
//         if poly.log_size() == 0 {
//             return poly.coeffs[0].into();
//         }

//         let mut mappings = vec![point.y];
//         let mut x = point.x;
//         for _ in 1..poly.log_size() {
//             mappings.push(x);
//             x = CirclePoint::double_x(x);
//         }
//         mappings.reverse();

//         fold(&poly.coeffs, &mappings)
//     }

// }
