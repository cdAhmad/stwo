// use bytemuck::cast_slice;
// use itertools::Chunk;

// use crate::core::{
//     backend::{ vulkan::m31::{ PackedBaseField, ELEMENT_SIZE }, Column },
//     fields::m31::{ BaseField, M31 },
// };

// #[derive(Clone, Debug)]
// pub struct BaseColumn {
//     pub data: Vec<PackedBaseField>,
//     /// The number of [`BaseField`]s in the vector.
//     pub length: usize,
// }

// impl BaseColumn {
//     pub fn as_slice(&self) -> &[BaseField] {
//         &cast_slice(&self.data)[..self.length]
//     }
// }

// impl Column<BaseField> for BaseColumn {
//     fn zeros(len: usize) -> Self {
//         Self {
//             data: vec![[0; ELEMENT_SIZE]; len.div_ceil(ELEMENT_SIZE)],
//             length: len,
//         }
//     }
//     #[allow(clippy::uninit_vec)]
//     unsafe fn uninitialized(len: usize) -> Self {
//         let mut data = Vec::with_capacity(len.div_ceil(ELEMENT_SIZE));
//         data.set_len(len.div_ceil(ELEMENT_SIZE));
//         Self { data, length: len }
//     }

//     fn to_cpu(&self) -> Vec<BaseField> {
//         self.as_slice().to_vec()
//     }

//     fn len(&self) -> usize {
//         self.length
//     }

//     fn at(&self, index: usize) -> BaseField {
//         M31(self.data[index / ELEMENT_SIZE][index % ELEMENT_SIZE])
//     }

//     fn set(&mut self, index: usize, value: BaseField) {
//         self.data[index / ELEMENT_SIZE][index % ELEMENT_SIZE] = value.0;
//     }

//     fn is_empty(&self) -> bool {
//         self.length == 0
//     }
// }

// impl FromIterator<BaseField> for BaseColumn {
//     fn from_iter<T: IntoIterator<Item = BaseField>>(iter: T) -> Self {
//         let mut chunks = iter.into_iter().array_chunks();
//         let mut data = (&mut chunks).map(PackedBaseField::from_array).collect_vec();
//         let mut length = data.len() * ELEMENT_SIZE;

//         let mut data = Vec::new();
//         let mut temp = [0u32; ELEMENT_SIZE];
//         let mut count = 0;
//         for value in iter {
//             temp[count % ELEMENT_SIZE] = value.0;
//             count += 1;
//             if count % ELEMENT_SIZE == 0 {
//                 data.push(temp);
//                 temp = [0u32; ELEMENT_SIZE];
//             }
//         }
//         if count % ELEMENT_SIZE != 0 {
//             data.push(temp);
//         }
//         Self {
//             data,
//             length: count,
//         }
//     }
// }

// use crate::core::{ backend::vulkan::VulkanBackend, fields::secure_column::SecureColumnByCoords };

// impl SecureColumnByCoords<VulkanBackend> {
//     pub fn to_uvec4(&self) {
//         let a: Vec<crate::core::fields::m31::M31>= self.columns[0];
//     }
// }
