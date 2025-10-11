use bytemuck::cast_slice;
use itertools::{ Itertools };

use crate::core::{
    backend::{
        vulkan::{ m31::{ PackedBaseField, PackedM31, ELEMENT_SIZE }, VulkanBackend },
        Column,
        CpuBackend,
    },
    fields::{ m31::{ BaseField }, qm31::SecureField, secure_column::SecureColumnByCoords },
};

#[derive(Clone, Debug)]
pub struct BaseColumn {
    pub data: Vec<PackedBaseField>,
    /// The number of [`BaseField`]s in the vector.
    pub length: usize,
}

impl BaseColumn {
    pub fn as_slice(&self) -> &[BaseField] {
        &cast_slice(&self.data)[..self.length]
    }
}

impl Column<BaseField> for BaseColumn {
    fn zeros(len: usize) -> Self {
        Self {
            data: vec![PackedM31::zero(); len.div_ceil(ELEMENT_SIZE)],
            length: len,
        }
    }
    #[allow(clippy::uninit_vec)]
    unsafe fn uninitialized(len: usize) -> Self {
        let mut data = Vec::with_capacity(len.div_ceil(ELEMENT_SIZE));
        data.set_len(len.div_ceil(ELEMENT_SIZE));
        Self { data, length: len }
    }

    fn to_cpu(&self) -> Vec<BaseField> {
        self.as_slice().to_vec()
    }

    fn len(&self) -> usize {
        self.length
    }

    fn at(&self, index: usize) -> BaseField {
        self.data[index / ELEMENT_SIZE].to_m31_array()[index % ELEMENT_SIZE]
    }

    fn set(&mut self, index: usize, value: BaseField) {
        self.data[index / ELEMENT_SIZE].set_m31(index % ELEMENT_SIZE, value);
    }

    fn is_empty(&self) -> bool {
        self.length == 0
    }
}

impl FromIterator<BaseField> for BaseColumn {
    fn from_iter<T: IntoIterator<Item = BaseField>>(iter: T) -> Self {
        let mut chunks = iter.into_iter().array_chunks();
        let mut data = (&mut chunks).map(PackedBaseField::from_m31_array).collect_vec();
        let mut length = data.len() * ELEMENT_SIZE;

        if let Some(remainder) = chunks.into_remainder() {
            if !remainder.is_empty() {
                length += remainder.len();
                let mut last: PackedM31 = PackedBaseField::zero();
                last.set_m31_array(remainder.as_slice());
                data.push(last);
            }
        }
        Self {
            data,
            length: length,
        }
    }
}

// use crate::core::{ backend::vulkan::VulkanBackend, fields::secure_column::SecureColumnByCoords };

impl SecureColumnByCoords<VulkanBackend> {
    pub const fn packed_len(&self) -> usize {
        self.columns[0].len()
    }
    
    pub unsafe fn packed_at(&self, vec_index: usize) -> [u32; 4] {
        [
            self.columns[0].get_unchecked(vec_index).0,
            self.columns[1].get_unchecked(vec_index).0,
            self.columns[2].get_unchecked(vec_index).0,
            self.columns[3].get_unchecked(vec_index).0,
        ]
    }

    pub fn to_uvec4(&self) -> Vec<[u32; 4]> {
        assert_eq!(self.columns.len(), 4);
        (0..self.packed_len()).map(|i| unsafe { self.packed_at(i) }).collect()
    }
    pub fn to_vec(&self) -> Vec<u32> {
        self.columns
            .iter()
            .flat_map(|c| c.iter().map(|f| f.0))
            .collect()
    }

    pub fn copy_from_slice(&mut self, slice: &[[u32; 4]]) {
        assert_eq!(self.columns.len(), 4);
        assert_eq!(self.packed_len(), slice.len());
        for i in 0..self.packed_len() {
            let [a, b, c, d] = slice[i];
            unsafe {
                self.columns[0].get_unchecked_mut(i).0 = a;
                self.columns[1].get_unchecked_mut(i).0 = b;
                self.columns[2].get_unchecked_mut(i).0 = c;
                self.columns[3].get_unchecked_mut(i).0 = d;
            }
        }
    }
    pub fn copy_from_vec(&mut self, slice: &[u32]) {
        assert_eq!(self.columns.len(), 4);
        //将 slice平铺到 self.columns中 不要transform
        let packed_len = self.packed_len();

        self.columns
            .iter_mut()
            .enumerate()
            .for_each(|(i, col)| {
                col.iter_mut()
                    .enumerate()
                    .for_each(|(j, f)| {
                        f.0 = slice[i * packed_len + j];
                    })
            });
    }
}
impl FromIterator<SecureField> for SecureColumnByCoords<VulkanBackend> {
    fn from_iter<I: IntoIterator<Item = SecureField>>(iter: I) -> Self {
        let cpu_col = SecureColumnByCoords::<CpuBackend>::from_iter(iter);
        let columns = cpu_col.columns.map(|col| col.into_iter().collect());
        SecureColumnByCoords { columns }
    }
}
