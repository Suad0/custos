use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{HasId, HostPtr, Id, Lazy, PtrType, ShallowCopy, WrappedData};

#[derive(Debug, Default)]
pub struct LazyWrapper<Data, T> {
    data: Option<Data>,
    id: Option<Id>,
    _pd: PhantomData<T>,
}

impl<Mods: WrappedData> WrappedData for Lazy<Mods> {
    type Wrap<T, Base: HasId + PtrType> = LazyWrapper<Mods::Wrap<T, Base>, T>;
    // type Wrap<T, Base: HasId + PtrType> = Mods::Wrap<T, Base>;

    #[inline]
    fn wrap_in_base<T, Base: HasId + PtrType>(&self, base: Base) -> Self::Wrap<T, Base> {
        todo!()
        // self.modules.wrap_in_base(base)
    }
}

impl<Data: HasId, T> HasId for LazyWrapper<Data, T> {
    #[inline]
    fn id(&self) -> crate::Id {
        self.id.unwrap()
    }
}

impl<Data: PtrType, T> PtrType for LazyWrapper<Data, T> {
    #[inline]
    fn size(&self) -> usize {
        self.data.as_ref().unwrap().size()
    }

    #[inline]
    fn flag(&self) -> crate::flag::AllocFlag {
        self.data.as_ref().unwrap().flag()
    }

    #[inline]
    unsafe fn set_flag(&mut self, flag: crate::flag::AllocFlag) {
        self.data.as_mut().unwrap().set_flag(flag)
    }
}

impl<Data: Deref<Target = [T]>, T> Deref for LazyWrapper<Data, T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.data.as_ref().unwrap()
    }
}

impl<Data: DerefMut<Target = [T]>, T> DerefMut for LazyWrapper<Data, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data.as_mut().unwrap()
    }
}

impl<T, Data: HostPtr<T>> HostPtr<T> for LazyWrapper<Data, T> {
    #[inline]
    fn ptr(&self) -> *const T {
        self.data.as_ref().unwrap().ptr()
    }

    #[inline]
    fn ptr_mut(&mut self) -> *mut T {
        self.data.as_mut().unwrap().ptr_mut()
    }
}

impl<Data: ShallowCopy, T> ShallowCopy for LazyWrapper<Data, T> {
    #[inline]
    unsafe fn shallow(&self) -> Self {
        LazyWrapper {
            id: self.id,
            data: self.data.as_ref().map(|data| data.shallow()),
            _pd: PhantomData,
        }
    }
}
