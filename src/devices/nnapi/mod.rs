mod nnapi_device;
pub use nnapi_device::*;

pub use nnapi::*;

use crate::{flag::AllocFlag, PtrConv, PtrType, Shape, HasId};

/*pub fn log(priority: ndk_sys::android_LogPriority, msg: &str) {
    let tag = std::ffi::CString::new("MyApp").unwrap();
    let msg = std::ffi::CString::new(msg).unwrap();
    unsafe { ndk_sys::__android_log_print(priority.0 as i32, tag.as_ptr(), msg.as_ptr()) };
}*/

#[derive(Debug, Clone)]
/// Denotes an index with a data type to a node in a nnapi model
pub struct NnapiPtr {
    /// The data type of the node
    pub dtype: Operand,
    /// The index of the node
    pub idx: u32,
    flag: AllocFlag,
}

impl Default for NnapiPtr {
    fn default() -> Self {
        Self {
            dtype: Operand::activation(),
            idx: u32::MAX,
            flag: AllocFlag::Wrapper,
        }
    }
}

impl HasId for NnapiPtr {
    #[inline]
    fn id(&self) -> crate::Id {
        crate::Id {
            id: self.idx as u64,
            len: self.dtype.len
        }
    }
}

impl PtrConv for NnapiDevice {
    unsafe fn convert<T, IS: Shape, Conv, OS: Shape>(
        ptr: &Self::Data<T, IS>,
        flag: crate::flag::AllocFlag,
    ) -> Self::Data<Conv, OS> {
        NnapiPtr {
            dtype: ptr.dtype.clone(),
            idx: ptr.idx,
            flag,
        }
    }
}

impl PtrType for NnapiPtr {
    #[inline]
    fn size(&self) -> usize {
        self.dtype.len
    }

    #[inline]
    fn flag(&self) -> crate::flag::AllocFlag {
        self.flag
    }
}
