use std::cell::RefCell;

use crate::number::Number;

#[cfg(feature="opencl")]
pub mod opencl;
#[cfg(feature="cuda")]
pub mod cuda;
pub mod cpu;

#[cfg(not(feature="opencl"))]
#[derive(Debug)]
pub struct CLDevice;

thread_local! {
    pub static COUNT: RefCell<usize> = RefCell::new(0);
}

/// Sets current cache identifier / index.
/// This function is usually called after an iteration in a loop -> [Count] or [range]
pub fn set_count(count: usize) {
    COUNT.with(|c| *c.borrow_mut() = count);
}

/// Returns current cache identifier / index
pub fn get_count() -> usize {
    COUNT.with(|c| *c.borrow())
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
/// A Node is used to identify a cached pointer.
pub struct Node {
    pub idx: usize,
    pub len: usize,
}

impl Node {
    pub fn new(len: usize) -> Node {
        crate::COUNT.with(|count| {
            let node = Node {
                idx: *count.borrow(),
                len,
            };
            *count.borrow_mut() += 1;
            node
        })
    }
}

pub trait GenericOCL: Number {
    fn as_ocl_type_str() -> &'static str;
}

#[cfg(any(not(target_os="macos"), not(feature="opencl")))]
impl GenericOCL for f64 {
    fn as_ocl_type_str() -> &'static str {
        "double"
    }
}

impl GenericOCL for f32 {
    fn as_ocl_type_str() -> &'static str {
        "float"
    }
}

impl GenericOCL for i32 {
    fn as_ocl_type_str() -> &'static str {
        "int"
    }
}

impl GenericOCL for u32 {
    fn as_ocl_type_str() -> &'static str {
        "uint"
    }
}

impl GenericOCL for i8 {
    fn as_ocl_type_str() -> &'static str {
        "char"
    }
}

impl GenericOCL for u8 {
    fn as_ocl_type_str() -> &'static str {
        "uchar"
    }
}

impl GenericOCL for i16 {
    fn as_ocl_type_str() -> &'static str {
        "short"
    }
}
impl GenericOCL for u16 {
    fn as_ocl_type_str() -> &'static str {
        "ushort"
    }
}

impl GenericOCL for i64 {
    fn as_ocl_type_str() -> &'static str {
        "long"
    }
}

impl GenericOCL for u64 {
    fn as_ocl_type_str() -> &'static str {
        "ulong"
    }
}

pub fn remove_value<T: Ord>(values: &mut Vec<T>, match_value: &T) -> Result<(), usize> {
    let idx = values.binary_search(match_value)?;
    values.swap_remove(idx);
    Ok(())
}