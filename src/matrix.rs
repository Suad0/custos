use crate::{Buffer, get_device, libs::{opencl::GenericOCL, cpu::TBlas}, get_gemm, Device};


#[derive(Debug, Clone, Copy)]
pub struct Matrix<T> {
    data: Buffer<T>,
    dims: (usize, usize)
}

impl <T: GenericOCL>Matrix<T> {
    pub fn new<D: Device<T>>(device: D, dims: (usize, usize)) -> Matrix<T> {

        Matrix {
            data: Buffer { ptr: device.alloc(dims.0*dims.1), len: dims.0*dims.1 },
            dims,
        }
    }
    pub fn data(&self) -> Buffer<T> {
        self.data
    }
}

impl <T: GenericOCL+TBlas>Matrix<T> {
    pub fn gemm(self, rhs: Matrix<T>) -> Matrix<T> {
        let device = get_gemm::<T>();
        device.gemm(self, rhs)
    }
}

impl <T>Matrix<T> {
    pub fn ptr(&self) -> *mut T {
        self.data.ptr
    }
    pub fn dims(&self) -> (usize, usize) {
        self.dims
    }
    pub fn size(&self) -> usize {
        self.dims.0 * self.dims.1
    }

    pub fn as_cpu_slice(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(self.data.ptr, self.data.len)
        }
    }

    pub fn as_cpu_slice_mut(&mut self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(self.data.ptr, self.data.len)
        }
    }
}

impl <T>From<(*mut T, (usize, usize))> for Matrix<T> {
    fn from(ptr_dims: (*mut T, (usize, usize))) -> Self {
        let dims = ptr_dims.1;
        Matrix {
            data: Buffer {ptr: ptr_dims.0, len: dims.0*dims.1},
            dims
        }
    }
}

impl <T: GenericOCL, const N: usize>From<((usize, usize), &[T; N])> for Matrix<T> {
    fn from(dims_slice: ((usize, usize), &[T; N])) -> Self {
        let device = get_device::<T>();
        
        let buffer = Buffer::from((device, dims_slice.1));
        Matrix {
            data: buffer,
            dims: dims_slice.0
        }        
    }
}

impl <T: GenericOCL, D: Device<T>, const N: usize>From<(D, (usize, usize), &[T; N])> for Matrix<T> {
    fn from(dims_slice: (D, (usize, usize), &[T; N])) -> Self {
        let buffer = Buffer::from((&dims_slice.0, dims_slice.2));
        Matrix {
            data: buffer,
            dims: dims_slice.1
        }        
    }
}



impl <T: GenericOCL>core::ops::Add for Matrix<T> {
    type Output = Matrix<T>;

    fn add(self, rhs: Self) -> Self::Output {
        let device = get_device::<T>();
        device.add(self, rhs)
    }
}


