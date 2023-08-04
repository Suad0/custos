mod ops;

use core::convert::Infallible;

use super::Device;
use crate::{
    cpu::CPUPtr,
    flag::AllocFlag,
    module_comb::{
        Alloc, Base, Buffer, Cached, CachedModule, HasId, HasModules, Module, OnDropBuffer,
        OnNewBuffer, Retrieve, Retriever, Setup, MainMemory, TapeActions,
    },
    Shape,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CPU<Mods = Base> {
    pub modules: Mods,
}

// maybe
impl<Mods> CPU<Mods> {
    pub fn default() -> CPU<CachedModule<Base, CPU<Cached<Base>>>> {
        CPU::<Cached<Base>>::new()
    }
}

impl<Mods: OnDropBuffer> Device for CPU<Mods> {
    type Error = Infallible;

    fn new() -> Result<Self, Self::Error> {
        todo!()
        // Ok(CPU::new())
    }
}

impl<Mods: OnDropBuffer> MainMemory for CPU<Mods> {
    #[inline]
    fn as_ptr<T, S: Shape>(ptr: &Self::Data<T, S>) -> *const T {
        ptr.ptr
    }

    #[inline]
    fn as_ptr_mut<T, S: Shape>(ptr: &mut Self::Data<T, S>) -> *mut T {
        ptr.ptr
    }
}

impl<Mods: OnDropBuffer> OnDropBuffer for CPU<Mods> {
    #[inline]
    fn on_drop_buffer<'a, T, D: Device, S: Shape>(&self, device: &'a D, buf: &Buffer<T, D, S>) {
        self.modules.on_drop_buffer(device, buf)
    }
}

impl<Mods> HasModules<Mods> for CPU<Mods> {
    #[inline]
    fn modules(&self) -> &Mods {
        &self.modules
    }
}

impl<SimpleMods> CPU<SimpleMods> {
    #[inline]
    pub fn new<NewMods>() -> CPU<NewMods>
    where
        SimpleMods: Module<CPU<SimpleMods>, Module = NewMods>,
        NewMods: Setup<CPU<NewMods>>,
    {
        let mut cpu = CPU {
            modules: SimpleMods::new(),
        };
        NewMods::setup(&mut cpu);
        cpu
    }
}

impl<Mods> Alloc for CPU<Mods> {
    type Data<T, S: Shape> = CPUPtr<T>;

    fn alloc<T, S: Shape>(&self, mut len: usize, flag: AllocFlag) -> Self::Data<T, S> {
        assert!(len > 0, "invalid buffer len: 0");

        if S::LEN > len {
            len = S::LEN
        }

        CPUPtr::new_initialized(len, flag)
    }

    fn alloc_from_slice<T, S>(&self, data: &[T]) -> Self::Data<T, S>
    where
        S: Shape,
        T: Clone,
    {
        assert!(!data.is_empty(), "invalid buffer len: 0");
        assert!(S::LEN <= data.len(), "invalid buffer len: {}", data.len());

        let cpu_ptr = unsafe { CPUPtr::new(data.len(), AllocFlag::None) };
        let slice = unsafe { std::slice::from_raw_parts_mut(cpu_ptr.ptr, data.len()) };
        slice.clone_from_slice(data);

        cpu_ptr
    }

    fn alloc_from_vec<T, S: Shape>(&self, mut vec: Vec<T>) -> Self::Data<T, S>
    where
        T: Clone,
    {
        assert!(!vec.is_empty(), "invalid buffer len: 0");

        let ptr = vec.as_mut_ptr();
        let len = vec.len();
        core::mem::forget(vec);

        unsafe { CPUPtr::from_ptr(ptr, len, AllocFlag::None) }
    }
}

impl<T, S: Shape, Mods: OnNewBuffer<T, Self, S> + OnDropBuffer> OnNewBuffer<T, Self, S>
    for CPU<Mods>
{
    #[inline]
    fn on_new_buffer(&self, device: &Self, new_buf: &Buffer<T, Self, S>) {
        self.modules.on_new_buffer(device, new_buf)
    }
}

impl<Mods: Retrieve<Self>> Retriever for CPU<Mods> {
    #[inline]
    fn retrieve<T: 'static, S: Shape>(&self, len: usize) -> Buffer<T, Self, S> {
        let data = self.modules.retrieve::<T, S>(self, len);
        let buf = Buffer {
            data,
            device: Some(self),
            // id: LocationId::new()
        };
        self.modules.on_retrieve_finish(&buf);
        buf
    }
}

impl<Mods: TapeActions> TapeActions for CPU<Mods> {
    #[inline]
    fn tape(&self) -> Option<core::cell::Ref<crate::module_comb::Tape>> {
        self.modules.tape()
    }

    #[inline]
    fn tape_mut(&self) -> Option<core::cell::RefMut<crate::module_comb::Tape>> {
        self.modules.tape_mut()
    }
}
