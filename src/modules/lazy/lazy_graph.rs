use super::ty::{Graphable, Type};
use crate::{
    bounds_to_range, Buffer, Device, DeviceError, Id, NoHasher, OpArgs, Parents, PtrConv, Shape,
    UniqueId,
};
use core::{
    alloc::Layout,
    any::Any,
    hash::BuildHasherDefault,
    mem::{align_of, size_of, transmute},
    ops::RangeBounds,
};
use std::collections::HashMap;

pub type ForwardFn = *mut dyn Fn(&'static ()) -> crate::Result<()>;
pub type TypedForwardFn<'a, T, D, S> =
    *mut (dyn Fn(&mut Buffer<T, D, S>) -> crate::Result<()> + 'a);

pub type ForwardFn2 = *mut dyn Fn(&'static (), *mut ()) -> crate::Result<()>;

#[derive(Default)]
pub struct LazyGraph {
    pub operations: Vec<(Type, ForwardFn)>,
    pub operations2: Vec<(Type, ForwardFn2)>,
    pub ids_to_check: Vec<Vec<UniqueId>>,
    pub ops: Vec<fn(*mut (), *mut ()) -> crate::Result<()>>,
    pub args: Vec<*mut ()>,
    pub arg_dealloc_info: Vec<(usize, usize)>,
}

impl Drop for LazyGraph {
    fn drop(&mut self) {
        for (_, operation) in self.operations.iter_mut() {
            unsafe { drop(Box::from_raw(*operation)) }
        }

        for (arg_ptr, (align, size)) in self.args.iter().zip(&self.arg_dealloc_info) {
            let layout = Layout::from_size_align(*size, *align).unwrap();
            unsafe { std::alloc::dealloc(*arg_ptr as *mut u8, layout) }
        }
    }
}

pub fn execute_with_type<T: 'static, D: Device + 'static>(
    operation: &mut ForwardFn,
    buf: &mut dyn Any,
) -> crate::Result<()> {
    let operation = unsafe { transmute::<_, &mut TypedForwardFn<T, D, ()>>(operation) };

    let buf: &mut Buffer<T, D, ()> =
        unsafe { &mut *(buf as *mut dyn Any as *mut Buffer<T, D, ()>) };
    unsafe { (**operation)(buf) }
}

pub fn execute_operation<D: Device + 'static>(
    ty: Type,
    operation: &mut ForwardFn,
    buf: &mut dyn Any,
) -> crate::Result<()> {
    match ty {
        Type::F32 => execute_with_type::<f32, D>(operation, buf),
        Type::I32 => execute_with_type::<i32, D>(operation, buf),
    }
}

pub type TypedForwardFn2<'a, T, D, S, Args> =
    *mut (dyn Fn(&mut Buffer<T, D, S>, &Args) -> crate::Result<()> + 'a);

impl LazyGraph {
    // TODO: could use a broader range of Args! (limited to Parents<N>)
    pub fn add_operation_op_args<T, D, S, Args: Parents<N>, const N: usize>(
        &mut self,
        args: Args,
        op: fn(&mut Buffer<T, D, S>, &Args) -> crate::Result<()>,
    ) where
        T: Graphable,
        D: PtrConv,
        S: Shape,
    {
        self.arg_dealloc_info
            .push((align_of::<Args>(), size_of::<Args>()));

        let args = Box::leak(Box::new(args));

        // store ids and test if buffers are still in cache
        self.ids_to_check
            .push(args.ids().into_iter().map(|id| *id).collect());

        self.args.push(args as *mut Args as *mut _);
        unsafe { self.ops.push(transmute(op)) }
    }

    pub unsafe fn call_lazily_op_args<D: Device + 'static>(
        &mut self,
        out_buf_order: &[Id],
        outs_unordered: &mut HashMap<UniqueId, Box<dyn Any>, BuildHasherDefault<NoHasher>>,
    ) -> crate::Result<()> {
        for (((args, op), ids_to_check), out_id) in self
            .args
            .iter()
            .zip(&self.ops)
            .zip(&self.ids_to_check)
            .zip(out_buf_order)
        {
            for id_to_check in ids_to_check.iter() {
                outs_unordered
                    .get(id_to_check)
                    .ok_or(DeviceError::InvalidLazyOutBuf)?;
            }

            let out = &mut **outs_unordered
                .get_mut(&out_id)
                .ok_or(DeviceError::InvalidLazyOutBuf)? as *mut _ as *mut ();

            op(out, *args)?;
        }
        Ok(())
    }

    pub fn add_operation<T, D, S>(
        &mut self,
        operation: impl Fn(&mut Buffer<T, D, S>) -> crate::Result<()>,
    ) where
        T: Graphable,
        D: PtrConv,
        S: Shape,
    {
        let operation = Box::leak(Box::new(operation));
        self.operations
            .push((T::TYPE, operation as TypedForwardFn<T, D, S> as *mut _))
    }

    /// # Safety
    /// The required 'static lifetime is ignored when adding operations. Hence, all captured variables must live long enough.
    pub unsafe fn call_lazily<D: Device + 'static>(
        &mut self,
        out_buf_order: &[Id],
        outs_unordered: &mut HashMap<UniqueId, Box<dyn Any>, BuildHasherDefault<NoHasher>>,
    ) -> crate::Result<()> {
        for ((ty, operation), buf_id) in self.operations.iter_mut().zip(out_buf_order) {
            let buf = &mut **outs_unordered
                .get_mut(buf_id)
                .ok_or(DeviceError::InvalidLazyOutBuf)?;

            execute_operation::<D>(*ty, operation, buf)?;
        }
        Ok(())
    }

    pub unsafe fn call_range<D: Device + 'static>(
        &mut self,
        bounds: impl RangeBounds<usize>,
        out_buf_order: &mut Vec<Id>,
        outs_unordered: &mut HashMap<UniqueId, Box<dyn Any>, BuildHasherDefault<NoHasher>>,
    ) -> crate::Result<()> {
        let range = bounds_to_range(bounds, out_buf_order.len());
        for ((ty, mut operation), buf_id) in self
            .operations
            .drain(range.clone())
            .zip(out_buf_order.drain(range))
        {
            let buf = &mut **outs_unordered
                .get_mut(&buf_id)
                .ok_or(DeviceError::InvalidLazyOutBuf)?;

            execute_operation::<D>(ty, &mut operation, buf)?;
            unsafe { drop(Box::from_raw(operation)) }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::{register_buf, Base, Buffer, Device, HasId, Retriever, CPU};
    use super::LazyGraph;
     
    #[test]
    #[should_panic]
    fn test_lazy_op_args_args_out_of_scope() {
        let device = CPU::<Base>::new();
        let mut graph = LazyGraph::default();
        let mut outs_unordered = HashMap::default();

        let out_id = {
            let lhs = device.buffer([1f32, 2., 3., 4., 5.]);
            let rhs = device.buffer([1f32, 2., 6., 4., 5.]);
            let out: Buffer = device.retrieve(lhs.len(), (&lhs, &rhs));
            unsafe { register_buf(&mut outs_unordered, &out) };
            // outs_unordered.insert(out.id(), )

            graph.add_operation_op_args::<f32, CPU, (), _, 2>((&lhs, &rhs), |_out, args| {
                let (lhs, rhs) = *args;
                assert_eq!(lhs.as_slice(), &[1f32, 2., 3., 4., 5.,]);
                assert_eq!(rhs.as_slice(), &[1f32, 2., 6., 4., 5.,]);
                Ok(())
            });

            out.id()
        };

        unsafe {
            graph
                .call_lazily_op_args::<CPU>(&[out_id], &mut outs_unordered)
                .unwrap()
        }
    }

    #[test]
    fn test_lazy_op_args() {
        let device = CPU::<Base>::new();
        let mut graph = LazyGraph::default();

        let lhs = device.buffer([1f32, 2., 3., 4., 5.]);
        let rhs = device.buffer([1f32, 2., 6., 4., 5.]);

        let mut outs_unordered = HashMap::default();

        let out: Buffer = device.retrieve(lhs.len(), (&lhs, &rhs));
        unsafe { register_buf(&mut outs_unordered, &lhs) };
        unsafe { register_buf(&mut outs_unordered, &rhs) };
        unsafe { register_buf(&mut outs_unordered, &out) };
        // outs_unordered.insert(out.id(), )

        graph.add_operation_op_args::<f32, CPU, (), _, 2>((&lhs, &rhs), |_out, args| {
            let (lhs, rhs) = *args;
            assert_eq!(lhs.as_slice(), &[1f32, 2., 3., 4., 5.,]);
            assert_eq!(rhs.as_slice(), &[1f32, 2., 6., 4., 5.,]);
            Ok(())
        });

        unsafe {
            graph
                .call_lazily_op_args::<CPU>(&[out.id()], &mut outs_unordered)
                .unwrap()
        }
    }
}
