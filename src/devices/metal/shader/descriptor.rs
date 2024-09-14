use metal::*;

pub struct BufferDescriptor;

impl BufferDescriptor {
    pub fn new(len: usize, options: MTLResourceOptions) -> MTLBuffer {
        let device = Device::system_default().expect("No Metal device found");
        device.new_buffer(len as u64, options)
    }
}
