use crate::{
    flag::AllocFlag,
    module_comb::{Alloc, Module, Retrieve, Setup, OnDropBuffer},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Base;

impl<D> Module<D> for Base {
    type Module = Base;

    #[inline]
    fn new() -> Self::Module {
        Base
    }
}

impl<D> Setup<D> for Base {}
impl OnDropBuffer for Base {}

impl<D> Retrieve<D> for Base {
    #[inline]
    fn retrieve<T, S: crate::Shape>(&self, device: &D, len: usize) -> <D>::Data<T, S>
    where
        D: Alloc,
    {
        device.alloc(len, AllocFlag::None)
    }
}
