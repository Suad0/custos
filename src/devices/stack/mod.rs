mod impl_buffer;
mod stack_array;
mod stack_device;

pub use stack_device::*;

#[cfg(test)]
mod tests {
    use crate::{Alloc, Buffer, Device, MainMemory, CPU};
    use core::ops::Add;

    use super::stack_device::Stack;

    pub trait AddBuf<T, D: Device, const N: usize = 0>: Device {
        fn add(&self, lhs: &Buffer<T, D, N>, rhs: &Buffer<T, D, N>) -> Buffer<T, Self, N>;
    }

    /*// Without stack support
    impl<T, D> AddBuf<T, D> for CPU
    where
        T: Add<Output = T> + Clone,
        D: CPUCL,
    {
        fn add(&self, lhs: &Buffer<T, D>, rhs: &Buffer<T, D>) -> Buffer<T, Self> {
            let len = core::cmp::min(lhs.len, rhs.len);

            let mut out = self.retrieve(len, (lhs, rhs));
            for i in 0..len {
                out[i] = lhs[i].clone() + rhs[i].clone();
            }
            out
        }
    }*/

    impl<T, D> AddBuf<T, D> for CPU
    where
        D: MainMemory,
        T: Add<Output = T> + Clone,
    {
        fn add(&self, lhs: &Buffer<T, D>, rhs: &Buffer<T, D>) -> Buffer<T, Self> {
            let len = core::cmp::min(lhs.len, rhs.len);

            let mut out = self.retrieve(len, (lhs, rhs));
            for i in 0..len {
                out[i] = lhs[i].clone() + rhs[i].clone();
            }
            out
        }
    }

    impl<const N: usize, T, D> AddBuf<T, D, N> for Stack
    where
        for<'a> Stack: Alloc<'a, T, N>,
        D: MainMemory,
        T: Add<Output = T> + Clone,
    {
        fn add(&self, lhs: &Buffer<T, D, N>, rhs: &Buffer<T, D, N>) -> Buffer<T, Self, N> {
            let len = core::cmp::min(lhs.len, rhs.len);

            let mut out = self.retrieve(len, (lhs, rhs));
            for i in 0..len {
                out[i] = lhs[i].clone() + rhs[i].clone();
            }
            out
        }
    }

    #[test]
    fn test_stack() {
        let buf = Buffer::<f32, Stack, 100>::from((Stack, [1f32; 100]));

        let out = Stack.add(&buf, &buf);
        assert_eq!(out.ptr.array, [2.; 100]);

        let cpu = CPU::new();

        // implement Buffer::<f32, _, 100> for cpu?
        //let buf = Buffer::<f32>::new(&cpu, 100);
        let buf = Buffer::from((&cpu, [1f32; 100]));
        let out = cpu.add(&buf, &buf);
        assert_eq!(out.as_slice(), &[2.; 100]);
    }
}
