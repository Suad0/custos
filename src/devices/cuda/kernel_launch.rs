use crate::{number::Number, Buffer, CUDA};
use std::ffi::c_void;

use super::{
    api::{cuOccupancyMaxPotentialBlockSize, culaunch_kernel},
    fn_cache,
};

/// Converts `Self` to a (cuda) *mut c_void.
/// This enables taking `Buffer` and a number `T` as an argument to an CUDA kernel.
/// # Example
/// ```
/// use custos::{CUDA, Buffer, cuda::AsCudaCvoidPtr};
///
/// fn args(args: &[&dyn AsCudaCvoidPtr]) {
///     // ...
/// }
///
/// fn main() -> custos::Result<()> {
///     let device = CUDA::new(0)?;
///
///     let buf = Buffer::<f32, _>::new(&device, 10);
///     let num = 4;
///     args(&[&num, &buf]);
///     Ok(())
/// }
/// ```
pub trait AsCudaCvoidPtr {
    /// Converts `Self` to a (cuda) *mut c_void.
    /// # Example
    /// ```
    /// use custos::{CUDA, Buffer, cuda::AsCudaCvoidPtr};
    ///
    /// fn main() -> custos::Result<()> {
    ///     let device = CUDA::new(0)?;
    ///     let buf = Buffer::<f32, _>::new(&device, 10);
    ///     
    ///     let _ptr = buf.as_cvoid_ptr();
    ///     Ok(())
    /// }
    ///
    fn as_cvoid_ptr(&self) -> *mut c_void;
}

impl<'a, T> AsCudaCvoidPtr for &Buffer<'a, T, CUDA> {
    fn as_cvoid_ptr(&self) -> *mut c_void {
        &self.ptr.ptr as *const u64 as *mut c_void
    }
}

impl<'a, T> AsCudaCvoidPtr for Buffer<'a, T, CUDA> {
    fn as_cvoid_ptr(&self) -> *mut c_void {
        &self.ptr.ptr as *const u64 as *mut c_void
    }
}

impl<T: Number> AsCudaCvoidPtr for T {
    fn as_cvoid_ptr(&self) -> *mut c_void {
        self as *const T as *mut c_void
    }
}

/// uses calculated occupancy as launch configuration to launch a CUDA kernel
/// # Safety
/// All kernel arguments must be set.
pub fn launch_kernel1d(
    len: usize,
    device: &CUDA,
    src: &str,
    fn_name: &str,
    params: &[&dyn AsCudaCvoidPtr],
) -> crate::Result<()> {
    let params = params
        .iter()
        .map(|param| param.as_cvoid_ptr())
        .collect::<Vec<_>>();

    let func = fn_cache(device, src, fn_name)?;

    let mut min_grid_size = 0;
    let mut block_size = 0;

    unsafe {
        cuOccupancyMaxPotentialBlockSize(
            &mut min_grid_size,
            &mut block_size,
            func.0,
            0,
            0,
            len as i32,
        )
        .to_result()?
    };
    let grid_size = (len as i32 + block_size - 1) / block_size;

    culaunch_kernel(
        &func,
        [grid_size as u32, 1, 1],
        [block_size as u32, 1, 1],
        device.stream(),
        &params,
    )?;
    Ok(())
}