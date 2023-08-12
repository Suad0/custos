use core::{cell::RefCell, marker::PhantomData};

use crate::{
    module_comb::{
        Alloc, Buffer, Cache, Device, Module, OnDropBuffer, OnNewBuffer, Parents, PtrConv,
        Retrieve, Setup, TapeActions,
    },
    Shape,
};

// creator struct
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Cached<Mods> {
    pd: PhantomData<Mods>,
}

/*impl<Mods, D> Retrieve<D> for Cached<Mods> {
    fn retrieve<T, S: Shape>(&self, device: &D, len: usize) -> <D>::Data<T, S>
    where
        D: Alloc
    {
        todo!()
    }
}*/

impl<Mods: Module<D>, D: Alloc> Module<D> for Cached<Mods> {
    type Module = CachedModule<Mods::Module, D>;

    fn new() -> Self::Module {
        CachedModule {
            modules: Mods::new(),
            cache: RefCell::new(Cache {
                nodes: Default::default(),
            }),
        }
    }
}

pub struct CachedModule<Mods, D: Alloc> {
    pub modules: Mods,
    cache: RefCell<Cache<D>>,
}

impl<Mods: Setup<NewDev>, D: Alloc, NewDev> Setup<NewDev> for CachedModule<Mods, D> {
    #[inline]
    fn setup(device: &mut NewDev) {
        Mods::setup(device)
    }
}

impl<T, D: Device, S: Shape, Mods: OnNewBuffer<T, D, S>, SD: Alloc> OnNewBuffer<T, D, S>
    for CachedModule<Mods, SD>
{
    fn on_new_buffer(&self, device: &D, new_buf: &Buffer<T, D, S>) {
        self.modules.on_new_buffer(device, new_buf)
    }
}

impl<Mods: OnDropBuffer, SD: Alloc> OnDropBuffer for CachedModule<Mods, SD> {
    #[inline]
    fn on_drop_buffer<'a, T, D: Device, S: Shape>(&self, device: &'a D, buf: &Buffer<T, D, S>) {
        self.modules.on_drop_buffer(device, buf)
    }
}

// TODO: a more general OnDropBuffer => "Module"
impl<Mods: Retrieve<D>, D: Alloc + PtrConv<SimpleDevice>, SimpleDevice: Alloc + PtrConv<D>>
    Retrieve<D> for CachedModule<Mods, SimpleDevice>
{
    #[inline]
    fn retrieve<T, S: Shape, const NUM_PARENTS: usize>(
        &self,
        device: &D,
        len: usize,
        _parents: impl Parents<NUM_PARENTS>,
    ) -> D::Data<T, S> {
        self.cache.borrow_mut().get(device, len, || ())
    }

    #[inline]
    fn on_retrieve_finish<T, S: Shape>(&self, retrieved_buf: &Buffer<T, D, S>)
    where
        T: 'static,
        D: Device,
    {
        self.modules.on_retrieve_finish(retrieved_buf)
    }
}

impl<Mods: TapeActions, SD: Alloc> TapeActions for CachedModule<Mods, SD> {
    #[inline]
    fn tape(&self) -> Option<core::cell::Ref<super::Tape>> {
        self.modules.tape()
    }

    #[inline]
    fn tape_mut(&self) -> Option<core::cell::RefMut<super::Tape>> {
        self.modules.tape_mut()
    }
}

#[macro_export]
macro_rules! debug_assert_tracked {
    () => {
        #[cfg(debug_assertions)]
        {
            let location = core::panic::Location::caller();
            assert_ne!(
                (file!(), line!(), column!()),
                (location.file(), location.line(), location.column()),
                "Function and operation must be annotated with `#[track_caller]`."
            );
        }
    };
}

/// This macro is nothing but a mechanism to ensure that the specific operation is annotated with `#[track_caller]`.
/// If the operation is not annotated with `#[track_caller]`, then the macro will cause a panic (in debug mode).
///
/// This macro turns the device, length and optionally type information into the following line of code:
/// ## From:
/// ```ignore
/// retrieve!(device, 10, f32)
/// ```
/// ## To:
/// ```ignore
/// custos::debug_assert_tracked!();
/// device.retrieve::<f32, ()>(10)
/// ```
///
/// If you ensure that the operation is annotated with `#[track_caller]`, then you can just write the following:
/// ```ignore
/// device.retrieve::<f32, ()>(10)
/// ```
///
/// # Example
/// Operation is not annotated with `#[track_caller]` and therefore will panic:
/// ```should_panic
/// use custos::{retrieve, module_comb::{CPU, Retriever, Buffer, Retrieve, Cached, Base}};
///
/// fn add_bufs<Mods: Retrieve<CPU<Mods>>>(device: &CPU<Mods>) -> Buffer<f32, CPU<Mods>, ()> {
///     retrieve!(device, 10, f32)
/// }
///
/// let device = CPU::<Cached<Base>>::new();
/// add_bufs(&device);
/// ```
/// Operation is annotated with `#[track_caller]`:
/// ```
/// use custos::{Dim1, retrieve, module_comb::{CPU, Retriever, Buffer, Retrieve, Cached, Base}};
///
/// #[track_caller]
/// fn add_bufs<Mods: Retrieve<CPU<Mods>>>(device: &CPU<Mods>) -> Buffer<f32, CPU<Mods>, Dim1<30>> {
///     retrieve!(device, 10, f32, Dim1<30>); // you can also specify the shape
///     retrieve!(device, 10) // or infer the type and shape from the output type
/// }
///
/// let device = CPU::<Cached<Base>>::new();
/// add_bufs(&device);
/// ```
#[macro_export]
macro_rules! retrieve {
    ($device:ident, $len:expr, $parents:expr) => {{
        $crate::debug_assert_tracked!();
        $device.retrieve($len, $parents)
    }}; /*($device:ident, $len:expr, $dtype:ty, ) => {{
            $crate::debug_assert_tracked!();
            $device.retrieve::<$dtype, ()>($len)
        }};
        ($device:ident, $len:expr, $dtype:ty, $shape:ty) => {{
            $crate::debug_assert_tracked!();
            $device.retrieve::<$dtype, $shape>($len)
        }};*/
}

#[cfg(test)]
mod tests {
    use core::{panic::Location, ptr::addr_of};

    // crate::modules
    use crate::module_comb::{location, Base, Buffer, Retrieve, Retriever, CPU};

    use super::Cached;

    // forgot to add track_caller
    #[cfg(debug_assertions)]
    fn add_bufs<Mods: Retrieve<CPU<Mods>>>(device: &CPU<Mods>) -> Buffer<f32, CPU<Mods>, ()> {
        retrieve!(device, 10, ())
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn test_forgot_track_caller_runtime_detection() {
        let device = CPU::<Cached<Base>>::new();

        let _out = add_bufs(&device);
        let _out = add_bufs(&device);
    }

    #[track_caller]
    fn add_bufs_tracked<Mods: Retrieve<CPU<Mods>>>(
        device: &CPU<Mods>,
    ) -> Buffer<f32, CPU<Mods>, ()> {
        retrieve!(device, 10, ())
    }

    #[test]
    fn test_added_track_caller() {
        let device = CPU::<Cached<Base>>::new();

        let _out = add_bufs_tracked(&device);
        let _out = add_bufs_tracked(&device);
    }

    #[test]
    fn test_location_ref_unique() {
        let ptr = location();
        let ptr1 = location();
        // bad
        assert_ne!(addr_of!(ptr), addr_of!(ptr1));
    }

    #[track_caller]
    fn location_tracked() -> &'static Location<'static> {
        Location::caller()
    }

    #[test]
    fn test_location_file_ptr_unique() {
        let ptr = location();
        let ptr1 = location();
        // good
        assert_eq!(ptr.file().as_ptr(), ptr1.file().as_ptr());
    }

    #[test]
    fn test_location_file_tracked_ptr_unique() {
        let ptr = location_tracked();
        let ptr1 = location_tracked();
        // good
        assert_eq!(ptr.file().as_ptr(), ptr1.file().as_ptr());
    }

    #[test]
    fn test_location_with_different_file_location_ptr_unique() {
        let ptr = location_tracked();
        let ptr1 = location();
        // good
        assert_ne!(ptr.file().as_ptr(), ptr1.file().as_ptr());
    }
}
