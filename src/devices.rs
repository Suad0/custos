//! This module defines all available compute devices

mod generic_blas;
pub use generic_blas::*;

#[cfg(feature = "cpu")]
pub mod cpu;

#[cfg(feature = "cuda")]
pub mod cuda;

#[cfg(feature = "opencl")]
pub mod opencl;

#[cfg(feature = "stack")]
pub mod stack;

#[cfg(feature = "wgpu")]
pub mod wgpu;

#[cfg(feature = "network")]
pub mod network;

mod stack_array;
pub use stack_array::*;

mod cdatatype;
pub use cdatatype::*;

#[cfg(all(any(feature = "cpu", feature = "stack"), feature = "macro"))]
mod cpu_stack_ops;

use crate::{Alloc, Buffer, HasId, OnDropBuffer, PtrType, Shape};

pub trait Device: OnDropBuffer + Sized {
    type Data<T, S: Shape>: HasId + PtrType;

    type Error;

    #[inline]
    fn new() -> Result<Self, Self::Error> {
        todo!()
    }

    /// Creates a new [`Buffer`] using `A`.
    ///
    /// # Example
    #[cfg_attr(feature = "cpu", doc = "```")]
    #[cfg_attr(not(feature = "cpu"), doc = "```ignore")]
    /// use custos::{CPU, Device};
    ///
    /// let device = CPU::new();
    /// let buf = device.buffer([5, 4, 3]);
    ///
    /// assert_eq!(buf.read(), [5, 4, 3]);
    /// ```
    fn buffer<'a, T, S: Shape, A>(&'a self, arr: A) -> Buffer<'a, T, Self, S>
    where
        Buffer<'a, T, Self, S>: From<(&'a Self, A)>,
    {
        Buffer::from((self, arr))
    }
}

#[macro_export]
macro_rules! impl_buffer_hook_traits {
    ($device:ident) => {
        impl<T, D: Device, S: Shape, Mods: OnNewBuffer<T, D, S>> OnNewBuffer<T, D, S>
            for $device<Mods>
        {
            #[inline]
            fn on_new_buffer(&self, device: &D, new_buf: &Buffer<T, D, S>) {
                self.modules.on_new_buffer(device, new_buf)
            }
        }

        impl<Mods: OnDropBuffer> OnDropBuffer for $device<Mods> {
            #[inline]
            fn on_drop_buffer<'a, T, D: Device, S: Shape>(
                &self,
                device: &'a D,
                buf: &Buffer<T, D, S>,
            ) {
                self.modules.on_drop_buffer(device, buf)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_retriever {
    ($device:ident) => {
        impl<Mods: Retrieve<Self>> Retriever for $device<Mods> {
            #[inline]
            fn retrieve<T: 'static, S: Shape, const NUM_PARENTS: usize>(
                &self,
                len: usize,
                parents: impl crate::Parents<NUM_PARENTS>,
            ) -> Buffer<T, Self, S> {
                let data = self
                    .modules
                    .retrieve::<T, S, NUM_PARENTS>(self, len, parents);
                let buf = Buffer {
                    data,
                    device: Some(self),
                };
                self.modules.on_retrieve_finish(&buf);
                buf
            }
        }
    };
}
