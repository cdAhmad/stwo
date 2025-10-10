// use crate::core::fields::m31::M31;

// pub const ELEMENT_SIZE: usize = 4;

// pub type Vulkan<T> = [T; ELEMENT_SIZE];

// pub type VulkanU32 = Vulkan<u32>;
// pub type VulkanF32 = Vulkan<f32>;
// pub type VulkanF64 = Vulkan<f64>;

// pub type PackedBaseField = PackedM31;

// #[derive(Copy, Clone, Debug)]
// #[repr(transparent)]
// pub struct PackedM31(Vulkan<u32>);

// impl PackedM31 {
//     pub fn from_array(values: [M31; ELEMENT_SIZE]) -> PackedM31 {
//         PackedM31(values.map(|x| x.0))
//     }

//     pub fn to_array(self) -> [M31; ELEMENT_SIZE] {
//         self.0.map(|x| M31(x))
//     }
// }
