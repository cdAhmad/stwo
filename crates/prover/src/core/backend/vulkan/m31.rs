use crate::core::fields::m31::M31;
use bytemuck::{ Pod, Zeroable };
pub const ELEMENT_SIZE: usize = 4;


pub type PackedBaseField = PackedM31;
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
#[repr(transparent)]
pub struct PackedM31([u32; ELEMENT_SIZE]);

 

impl PackedM31 {
    pub fn from_m31_array(values: [M31; ELEMENT_SIZE]) -> PackedM31 {
        PackedM31(values.map(|M31(x)| x))
    }
    // pub fn from_array(values: [u32; ELEMENT_SIZE]) -> PackedM31 {
    //     PackedM31(values)
    // }

    pub fn to_m31_array(self) -> [M31; ELEMENT_SIZE] {
        self.0.map(M31)
    }
    // pub fn set(&mut self, index: usize, values: u32) {
    //     self.0[index] = values;
    // }
    pub fn set_m31(&mut self, index: usize, values: M31) {
        self.0[index] = values.0;
    }
    pub fn zero() -> Self {
        Self([0; ELEMENT_SIZE])
    }

    // pub fn is_zero(&self) -> bool {
    //     self.0.iter().all(|&x| x == 0)
    // }
    // pub fn one() -> Self {
    //     Self([1; ELEMENT_SIZE])
    // }

    pub fn set_m31_array(&mut self, as_slice: &[M31]) {
        (0..as_slice.len()).for_each(|i| {
            self.0[i] = as_slice[i].0;
        });
    }
}

