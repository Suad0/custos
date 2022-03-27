use crate::matrix::Matrix;

pub trait BaseOps<T> {
    fn add(&self, lhs: Matrix<T>, rhs: Matrix<T>) -> Matrix<T>;
    fn sub(&self, lhs: Matrix<T>, rhs: Matrix<T>) -> Matrix<T>;
    fn mul(&self, lhs: Matrix<T>, rhs: Matrix<T>) -> Matrix<T>;
}

pub trait Device<T> {
    fn alloc(&self, len: usize) -> *mut T;
    fn with_data(&self, data: &[T]) -> *mut T;
}

#[derive(Debug, Clone, Copy)]
pub struct Buffer<T> {
    pub ptr: *mut T,
    pub len: usize,
}

impl <T: Default+Copy>Buffer<T> {
    pub fn new<D: Device<T>>(device: &D, len: usize) -> Buffer<T> {
        Buffer {
            ptr: device.alloc(len),
            len,
        }
    }
}

impl <T: Clone, const N: usize>From<(&Box<dyn Device<T>>, &[T; N])> for Buffer<T> {
    fn from(device_slice: (&Box<dyn Device<T>>, &[T; N])) -> Self {
        Buffer {
            ptr: device_slice.0.with_data(device_slice.1),
            len: device_slice.1.len()
        }
    }
}


impl <T: Clone, D: Device<T>, const N: usize>From<(&D, [T; N])> for Buffer<T> {
    fn from(device_slice: (&D, [T; N])) -> Self {
        Buffer {
            ptr: device_slice.0.with_data(&device_slice.1),
            len: device_slice.1.len()
        }
        
    }
}

impl <T: Clone, D: Device<T>>From<(&D, &[T])> for Buffer<T> {
    fn from(device_slice: (&D, &[T])) -> Self {
        Buffer {
            ptr: device_slice.0.with_data(device_slice.1),
            len: device_slice.1.len()
        }
        
    }
}

impl <T: Clone, D: Device<T>>From<(&D, Vec<T>)> for Buffer<T> {
    fn from(device_slice: (&D, Vec<T>)) -> Self {
        Buffer {
            ptr: device_slice.0.with_data(&device_slice.1),
            len: device_slice.1.len()
        }
        
    }
}
