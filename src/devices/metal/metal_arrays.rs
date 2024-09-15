use metal::*;
use std::ops::{Deref, DerefMut};
use std::ptr::null_mut;
use std::rc::Rc;
use crate::alloc_flag::AllocFlag;
use crate::context::MetalContext;
use metal::Buffer;

pub struct MetalArray<T> {
    pub len: usize,
    pub buffer: Buffer,
    pub context: Rc<MetalContext>,
    pub mapped_ptr: *mut T,
    pub flag: AllocFlag,
}

impl<T> MetalArray<T> {
    pub fn write_buf(&self, src_buf: &MetalArray<T>) {
        let command_buffer = self.context.command_queue.new_command_buffer();

        let blit_encoder = command_buffer.new_blit_command_encoder();
        blit_encoder.copy_from_buffer(
            &src_buf.buffer,
            0,
            &self.buffer,
            0,
            (self.len * std::mem::size_of::<T>()) as u64,
        );
        blit_encoder.end_encoding();

        command_buffer.commit();
        command_buffer.wait_until_completed();
    }

    pub fn new(
        context: Rc<MetalContext>,
        len: usize,
        usage: MTLResourceOptions,
        flag: AllocFlag,
    ) -> Self {
        let buffer_size = len * std::mem::size_of::<T>();

        // Create Metal buffer
        let buffer = context.device.new_buffer(buffer_size as u64, usage);

        // Check if memory is host-visible (shared)
        let mapped_ptr = if usage.contains(MTLResourceOptions::StorageModeShared) {
            buffer.contents() as *mut T
        } else {
            null_mut()
        };

        MetalArray {
            len,
            buffer, // Assign the `Buffer` directly
            context,
            mapped_ptr,
            flag,
        }
    }

    pub fn from_slice(
        context: Rc<MetalContext>,
        data: &[T],
        usage: MTLResourceOptions,
        flag: AllocFlag,
    ) -> Self where T: Clone {
        let mut array = MetalArray::<T>::new(context, data.len(), usage, flag);
        array.clone_from_slice(data);
        array
    }

    pub fn write_staged(&self, data: &[T])
        where T: Clone {
        if !self.mapped_ptr.is_null() {
            unsafe {
                std::ptr::copy_nonoverlapping(data.as_ptr(), self.mapped_ptr, self.len);
            }
        } else {
            // For device-local memory, we stage the data in a temporary buffer
            let staging_buffer = MetalArray::from_slice(
                self.context.clone(),
                data,
                MTLResourceOptions::StorageModeShared,
                AllocFlag::None,
            );
            self.write_buf(&staging_buffer);
        }
    }

    pub fn read_staged(&self) -> Vec<T>
        where T: Clone {
        let mut result = Vec::with_capacity(self.len);
        if !self.mapped_ptr.is_null() {
            unsafe {
                result.set_len(self.len);
                std::ptr::copy_nonoverlapping(self.mapped_ptr, result.as_mut_ptr(), self.len);
            }
        } else {
            // For device-local memory, use a staging buffer to copy data back to the host
            let staging_buffer = MetalArray::<T>::new(
                self.context.clone(),
                self.len,
                MTLResourceOptions::StorageModeShared,
                AllocFlag::None,
            );
            staging_buffer.write_buf(self);
            return staging_buffer.read_staged(); // Read from the staging buffer
        }
        result
    }
}

// Implement dereferencing so you can access data in the buffer like an array
impl<T> Deref for MetalArray<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        assert!(!self.mapped_ptr.is_null());
        unsafe { std::slice::from_raw_parts(self.mapped_ptr, self.len) }
    }
}

impl<T> DerefMut for MetalArray<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        assert!(!self.mapped_ptr.is_null());
        unsafe { std::slice::from_raw_parts_mut(self.mapped_ptr, self.len) }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use metal::MTLResourceOptions;
    use crate::alloc_flag::AllocFlag;
    use super::{MetalArray, MetalContext};

    #[test]
    fn test_metal_array_allocation() {
        let context = Rc::new(MetalContext::new());
        let arr1 = MetalArray::<f32>::new(
            context.clone(),
            10,
            MTLResourceOptions::StorageModeShared,
            AllocFlag::None,
        );
        assert_eq!(arr1.len(), 10);
    }

    #[test]
    fn test_metal_array_write_read() {
        let context = Rc::new(MetalContext::new());
        let mut arr1 = MetalArray::<f32>::new(
            context.clone(),
            5,
            MTLResourceOptions::StorageModeShared,
            AllocFlag::None,
        );

        arr1.write_staged(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let output = arr1.read_staged();
        assert_eq!(output, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    }
}
