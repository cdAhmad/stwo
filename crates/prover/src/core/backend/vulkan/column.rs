use crate::core::{
    backend::{
        vulkan::{ gpu_context::PIPELINE_BIT_REVERSE, VulkanBackend },
        Column,
        ColumnOps,
        CpuBackend,
    },
    fields::{ m31::{ BaseField, M31 }, qm31::SecureField, secure_column::SecureColumnByCoords },
};

impl ColumnOps<BaseField> for VulkanBackend {
    type Column = Vec<BaseField>;
    fn bit_reverse_column(column: &mut Self::Column) {
        let n = column.len();
        assert!(n.is_power_of_two());
        let log_n = n.ilog2();

        let context = Self::gpu_context();

        let u32_slice = unsafe {
            std::slice::from_raw_parts_mut(column.as_mut_ptr() as *mut u32, n)
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
            std::ptr::copy_nonoverlapping(mapped.as_ptr(), column.as_mut_ptr() as *mut u32, n);
        }
    }
}

#[derive(Clone, Debug)]
pub struct BaseColumn {
    pub data: Vec<BaseField>,
}

// impl BaseColumn {
//     pub fn as_slice(&self) -> &[BaseField] {
//         &self.data
//     }
// }

impl Column<BaseField> for BaseColumn {
    fn zeros(len: usize) -> Self {
        Self {
            data: vec![M31(0); len],
        }
    }
    #[allow(clippy::uninit_vec)]
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

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl FromIterator<BaseField> for BaseColumn {
    fn from_iter<T: IntoIterator<Item = BaseField>>(iter: T) -> Self {
        let data = iter.into_iter().collect();
        Self {
            data,
        }
    }
}

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
