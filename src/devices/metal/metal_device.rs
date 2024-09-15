use metal::*;
use crate::context::MetalContext;

pub struct MetalBuffer {
    pub buffer: Buffer,
}

impl MetalBuffer {
    pub fn new(context: &MetalContext, data: &[f32]) -> MetalBuffer {
        let size = (data.len() * std::mem::size_of::<f32>()) as u64;
        let buffer = context.device.new_buffer_with_data(data.as_ptr() as *const _, size, MTLResourceOptions::CPUCacheModeDefaultCache);
        MetalBuffer { buffer }
    }
}
