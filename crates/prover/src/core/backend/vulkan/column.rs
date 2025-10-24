use std::fmt::Debug;

use num_traits::Zero;

use crate::core::{
    backend::{
        vulkan::{ gpu_context::PIPELINE_BIT_REVERSE, VulkanBackend },
        Column,
        ColumnOps,
        CpuBackend,
    },
    fields::{
        cm31::CM31,
        m31::{ BaseField, M31 },
        qm31::SecureField,
        secure_column::SecureColumnByCoords,
    },
};
#[derive(Clone, Debug, PartialEq)]
pub struct VulkanColumn {
    pub data: Vec<BaseField>,
}

impl ColumnOps<BaseField> for VulkanBackend {
    type Column = VulkanColumn;
    fn bit_reverse_column(column: &mut Self::Column) {
        let n = column.len();
        assert!(n.is_power_of_two());
        let log_n = n.ilog2();

        let context = Self::gpu_context();

        let u32_slice = unsafe {
            std::slice::from_raw_parts_mut(column.data.as_mut_ptr() as *mut u32, n)
        };

        let buffer = context.buffer_in_out(u32_slice);
        let pipeline = context.pipeline(PIPELINE_BIT_REVERSE);

        let descriptor_set = context.descriptor_set(&pipeline, &[buffer.clone()]);
        let group_counts = context.group_counts(n);
        // 创建命令缓冲区
        let command_buffer = context.command_buffer(
            &pipeline,
            descriptor_set,
            group_counts,
            &[log_n as u32]
        );
        context.execution_wait(command_buffer);
        let mapped = buffer.read().expect("Failed to read buffer");
        unsafe {
            std::ptr::copy_nonoverlapping(mapped.as_ptr(), column.data.as_mut_ptr() as *mut u32, n);
        }
    }
}

impl Column<BaseField> for VulkanColumn {
    fn zeros(len: usize) -> Self {
        let data = vec![M31(0); len];
        Self { data }
    }

    unsafe fn uninitialized(len: usize) -> Self {
        let data = Vec::with_capacity(len);
        Self { data }
    }

    fn to_cpu(&self) -> Vec<BaseField> {
        self.data.clone()
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn at(&self, index: usize) -> BaseField {
        self.data[index]
    }

    fn set(&mut self, index: usize, value: BaseField) {
        self.data[index] = value;
    }
}

impl FromIterator<BaseField> for VulkanColumn {
    fn from_iter<T: IntoIterator<Item = BaseField>>(iter: T) -> Self {
        let data = iter.into_iter().collect();
        Self { data }
    }
}

impl IntoIterator for VulkanColumn {
    type Item = BaseField;
    type IntoIter = std::vec::IntoIter<BaseField>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct VulkanCM31Column {
    pub data: Vec<CM31>,
}

impl ColumnOps<CM31> for VulkanBackend {
    type Column = VulkanCM31Column;

    fn bit_reverse_column(column: &mut Self::Column) {
        let n = column.len();
        assert!(n.is_power_of_two());
        let log_n = n.ilog2();

        let context = Self::gpu_context();

        let u32_slice = unsafe {
            std::slice::from_raw_parts_mut(column.data.as_mut_ptr() as *mut u32, n)
        };

        let buffer = context.buffer_in_out(u32_slice);
        let pipeline = context.pipeline(PIPELINE_BIT_REVERSE);

        let descriptor_set = context.descriptor_set(&pipeline, &[buffer.clone()]);
        let group_counts = context.group_counts(n << 1);
        // 创建命令缓冲区
        let command_buffer = context.command_buffer(
            &pipeline,
            descriptor_set,
            group_counts,
            &[log_n as u32]
        );
        context.execution_wait(command_buffer);
        let mapped = buffer.read().expect("Failed to read buffer");
        unsafe {
            std::ptr::copy_nonoverlapping(mapped.as_ptr(), column.data.as_mut_ptr() as *mut u32, n);
        }
    }
}
impl Column<CM31> for VulkanCM31Column {
    fn zeros(len: usize) -> Self {
        let data = vec![CM31::zero(); len];
        Self { data }
    }
    #[allow(clippy::uninit_vec)]
    unsafe fn uninitialized(length: usize) -> Self {
        let data = Vec::with_capacity(length);

        Self { data }
    }

    fn to_cpu(&self) -> Vec<CM31> {
        self.data.clone()
    }
    fn at(&self, index: usize) -> CM31 {
        self.data[index]
    }

    fn set(&mut self, index: usize, value: CM31) {
        self.data[index] = value;
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

impl FromIterator<CM31> for VulkanCM31Column {
    fn from_iter<T: IntoIterator<Item = CM31>>(iter: T) -> Self {
        let data = iter.into_iter().collect();
        Self { data }
    }
}
impl IntoIterator for VulkanCM31Column {
    type Item = CM31;
    type IntoIter = std::vec::IntoIter<CM31>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct VulkanSecureColumn {
    pub data: Vec<SecureField>,
}

impl ColumnOps<SecureField> for VulkanBackend {
    type Column = VulkanSecureColumn;
    fn bit_reverse_column(column: &mut Self::Column) {
        let n = column.len();
        assert!(n.is_power_of_two());
        let log_n = n.ilog2();

        let context = Self::gpu_context();

        let u32_slice = unsafe {
            std::slice::from_raw_parts_mut(column.data.as_mut_ptr() as *mut u32, n)
        };

        let buffer = context.buffer_in_out(u32_slice);
        let pipeline = context.pipeline(PIPELINE_BIT_REVERSE);

        let descriptor_set = context.descriptor_set(&pipeline, &[buffer.clone()]);
        let group_counts = context.group_counts(n << 1);
        // 创建命令缓冲区
        let command_buffer = context.command_buffer(
            &pipeline,
            descriptor_set,
            group_counts,
            &[log_n as u32]
        );
        context.execution_wait(command_buffer);
        let mapped = buffer.read().expect("Failed to read buffer");
        unsafe {
            std::ptr::copy_nonoverlapping(mapped.as_ptr(), column.data.as_mut_ptr() as *mut u32, n);
        }
    }
}

impl Column<SecureField> for VulkanSecureColumn {
    fn zeros(len: usize) -> Self {
        let data = vec![SecureField::zero(); len];
        Self { data }
    }
    unsafe fn uninitialized(len: usize) -> Self {
        let data = Vec::with_capacity(len);
        Self { data }
    }
    fn to_cpu(&self) -> Vec<SecureField> {
        self.data.clone()
    }
    fn at(&self, index: usize) -> SecureField {
        self.data[index]
    }
    fn set(&mut self, index: usize, value: SecureField) {
        self.data[index] = value;
    }
    fn len(&self) -> usize {
        self.data.len()
    }
    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl FromIterator<SecureField> for VulkanSecureColumn {
    fn from_iter<T: IntoIterator<Item = SecureField>>(iter: T) -> Self {
        let data = iter.into_iter().collect();
        Self { data }
    }
}
impl IntoIterator for VulkanSecureColumn {
    type Item = SecureField;
    type IntoIter = std::vec::IntoIter<SecureField>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl SecureColumnByCoords<VulkanBackend> {
    pub const fn packed_len(&self) -> usize {
        self.columns[0].data.len()
    }

    pub unsafe fn packed_at(&self, vec_index: usize) -> [u32; 4] {
        [
            self.columns[0].data.get_unchecked(vec_index).0,
            self.columns[1].data.get_unchecked(vec_index).0,
            self.columns[2].data.get_unchecked(vec_index).0,
            self.columns[3].data.get_unchecked(vec_index).0,
        ]
    }

    pub fn to_uvec4(&self) -> Vec<[u32; 4]> {
        assert_eq!(self.columns.len(), 4);
        (0..self.packed_len()).map(|i| unsafe { self.packed_at(i) }).collect()
    }
    pub fn to_vec(&self) -> Vec<u32> {
        self.columns
            .iter()
            .flat_map(|c| c.data.iter().map(|f| f.0))
            .collect()
    }

    pub fn copy_from_slice(&mut self, slice: &[[u32; 4]]) {
        assert_eq!(self.columns.len(), 4);
        assert_eq!(self.packed_len(), slice.len());
        for i in 0..self.packed_len() {
            let [a, b, c, d] = slice[i];
            unsafe {
                self.columns[0].data.get_unchecked_mut(i).0 = a;
                self.columns[1].data.get_unchecked_mut(i).0 = b;
                self.columns[2].data.get_unchecked_mut(i).0 = c;
                self.columns[3].data.get_unchecked_mut(i).0 = d;
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
                col.data
                    .iter_mut()
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

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use crate::core::{ backend::{ cpu::bit_reverse as cpu_bit_reverse, vulkan::{column::VulkanColumn, VulkanBackend}, ColumnOps }, fields::m31::BaseField };

    #[test]
    fn bit_reverse_large_column_works() {
        const LOG_SIZE: u32 = 22;
        let column = (0..1 << LOG_SIZE).map(BaseField::from).collect_vec();
        let mut expected = column.clone();
        cpu_bit_reverse(&mut expected);

        let mut column = VulkanColumn { data: column.clone() };
        <VulkanBackend as ColumnOps<BaseField>>::bit_reverse_column(&mut column);

        assert_eq!(expected, column.data);
    }
}
