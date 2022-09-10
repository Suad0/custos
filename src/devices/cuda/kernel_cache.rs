use super::api::{
    cufree, load_module_data,
    nvrtc::{create_program, nvrtcDestroyProgram},
    FnHandle,
};
use crate::{cache::CacheType, CUDA, Error, Node};
use std::{collections::HashMap, ffi::CString, ptr::null_mut};

#[derive(Debug)]
pub struct RawCUBuf {
    pub ptr: u64,
    pub node: Node,
}

impl CacheType for RawCUBuf {
    fn new<T>(ptr: (*mut T, *mut std::ffi::c_void, u64), _: usize, node: Node) -> Self {
        RawCUBuf { ptr: ptr.2, node }
    }

    fn destruct<T>(&self) -> ((*mut T, *mut std::ffi::c_void, u64), Node) {
        ((null_mut(), null_mut(), self.ptr), self.node)
    }
}

impl Drop for RawCUBuf {
    fn drop(&mut self) {
        unsafe { cufree(self.ptr).unwrap() }
    }
}

#[derive(Debug, Default)]
pub struct KernelCacheCU {
    pub kernels: HashMap<String, FnHandle>,
}

impl KernelCacheCU {
    pub fn kernel(
        &mut self,
        device: &CUDA,
        src: &str,
        fn_name: &str,
    ) -> Result<FnHandle, Error> {
        let kernel = self.kernels.get(src);

        if let Some(kernel) = kernel {
            return Ok(*kernel);
        }

        let mut x = create_program(src, "")?;

        x.compile(Some(vec![CString::new("--use_fast_math").unwrap()]))?;

        let module = load_module_data(x.ptx()?)?;
        let function = module.function(fn_name)?;

        device.modules.borrow_mut().push(module);

        self.kernels.insert(src.into(), function);
        unsafe { nvrtcDestroyProgram(&mut x.0).to_result()? };
        Ok(function)
    }
}

pub fn fn_cache(device: &CUDA, src: &str, fn_name: &str) -> crate::Result<FnHandle> {
    device
        .kernel_cache
        .borrow_mut()
        .kernel(device, src, fn_name)
}
