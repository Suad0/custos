use core::{cell::RefMut, marker::PhantomData};
use std::collections::HashMap;

use alloc::rc::Rc;

use crate::{
    bump_count, AddGraph, Alloc, BufFlag, Buffer, CacheAble, Device, GraphReturn, Ident, Node,
};

/// This trait makes a device's [`Cache`] accessible and is implemented for all compute devices.
pub trait CacheReturn: GraphReturn {
    type CT;
    /// Returns a device specific [`Cache`].
    fn cache(&self) -> RefMut<Cache<Self>>
    where
        Self: RawConv;
}

pub trait RawConv: Device + CacheReturn {
    fn construct<T, const N: usize>(ptr: &Self::Ptr<T, N>, len: usize, node: Node) -> Self::CT;
    fn destruct<T, const N: usize>(ct: &Self::CT) -> (Self::Ptr<T, N>, Node);
}

#[derive(Debug)]
pub struct Cache<D: RawConv> {
    pub nodes: HashMap<Ident, Rc<D::CT>>,
    _p: PhantomData<D>,
}

impl<D: RawConv> Default for Cache<D> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            _p: PhantomData,
        }
    }
}

impl<D> CacheAble<D> for Cache<D>
where
    D: RawConv,
{
    fn retrieve<T, const N: usize >(device: &D, len: usize, add_node: impl AddGraph) -> Buffer<T, D, N>
    where
        for<'b> D: Alloc<'b, T, N>,
    {
        Cache::get(device, len, add_node)
    }
}

impl<D: RawConv> Cache<D> {
    /// Adds a new cache entry to the cache.
    /// The next get call will return this entry if the [Ident] is correct.
    /// # Example
    /// ```
    /// use custos::prelude::*;
    /// use custos::Ident;
    ///
    /// let device = CPU::new();
    /// let cache: Buffer = device
    ///     .cache()
    ///     .add_node(&device, Ident { idx: 0, len: 7 }, ());
    ///
    /// let ptr = device
    ///     .cache()
    ///     .nodes
    ///     .get(&Ident { idx: 0, len: 7 })
    ///     .unwrap()
    ///     .clone();
    ///
    /// assert_eq!(cache.host_ptr(), ptr.ptr as *mut f32);
    /// ```
    pub fn add_node<'a, T, const N: usize>(
        &mut self,
        device: &'a D,
        node: Ident,
        _add_node: impl AddGraph,
    ) -> Buffer<'a, T, D, N>
    where
        D: Alloc<'a, T, N> + RawConv,
    {
        let ptr = device.alloc(node.len);

        #[cfg(feature = "opt-cache")]
        let graph_node = device.graph().add(node.len, _add_node);

        #[cfg(not(feature = "opt-cache"))]
        let graph_node = Node::default();

        let raw_ptr = D::construct(&ptr, node.len, graph_node);
        self.nodes.insert(node, Rc::new(raw_ptr));

        bump_count();

        Buffer {
            ptr,
            len: node.len,
            device: Some(device),
            flag: BufFlag::Cache,
            node: graph_node,
        }
    }

    /// Retrieves cached pointers and constructs a [`Buffer`] with the pointers and the given `len`gth.
    /// If a cached pointer doesn't exist, a new `Buffer` will be added to the cache and returned.
    ///
    /// # Example
    /// ```
    /// use custos::prelude::*;
    ///
    /// let device = CPU::new();
    ///     
    /// let cache_entry: Buffer = Cache::get(&device, 10, ());
    /// let new_cache_entry: Buffer = Cache::get(&device, 10, ());
    ///
    /// assert_ne!(cache_entry.ptrs(), new_cache_entry.ptrs());
    ///
    /// set_count(0);
    ///
    /// let first_entry: Buffer = Cache::get(&device, 10, ());
    /// assert_eq!(cache_entry.ptrs(), first_entry.ptrs());
    /// ```
    #[cfg(not(feature = "realloc"))]
    pub fn get<'a, T, const N: usize>(
        device: &'a D,
        len: usize,
        add_node: impl AddGraph,
    ) -> Buffer<'a, T, D, N>
    where
        D: Alloc<'a, T, N> + RawConv,
    {
        let node = Ident::new(len);

        let mut cache = device.cache();
        let ptr_option = cache.nodes.get(&node);

        match ptr_option {
            Some(ptr) => {
                bump_count();

                let (ptr, node) = D::destruct::<T, N>(ptr);

                Buffer {
                    ptr,
                    len,
                    device: Some(device),
                    flag: BufFlag::Cache,
                    node,
                }
            }
            None => cache.add_node(device, node, add_node),
        }
    }

    /// If the 'realloc' feature is enabled, this functions always returns a new [`Buffer`] with the size of `len`gth.
    #[cfg(feature = "realloc")]
    pub fn get<'a, T, const N: usize>(
        device: &'a D,
        len: usize,
        _: impl AddGraph,
    ) -> Buffer<T, D, N>
    where
        D: Alloc<'a, T, N>,
    {
        Buffer::new(device, len)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "realloc"))]
    use crate::{set_count, Cache};
    use crate::{Buffer, CacheReturn, Ident, CPU};

    #[test]
    fn test_add_node() {
        let device = CPU::new();
        let cache: Buffer = device
            .cache()
            .add_node(&device, Ident { idx: 0, len: 7 }, ());

        let ptr = device
            .cache()
            .nodes
            .get(&Ident { idx: 0, len: 7 })
            .unwrap()
            .clone();

        assert_eq!(cache.host_ptr(), ptr.ptr as *mut f32);
    }

    #[cfg(not(feature = "realloc"))]
    #[test]
    fn test_get() {
        // for: cargo test -- --test-threads=1
        set_count(0);
        let device = CPU::new();

        let cache_entry: Buffer = Cache::get(&device, 10, ());
        let new_cache_entry: Buffer = Cache::get(&device, 10, ());

        assert_ne!(cache_entry.ptrs(), new_cache_entry.ptrs());

        set_count(0);

        let first_entry: Buffer = Cache::get(&device, 10, ());
        assert_eq!(cache_entry.ptrs(), first_entry.ptrs());
    }
}
