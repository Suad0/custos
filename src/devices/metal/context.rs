use metal::*;
use objc::rc::Rc;

pub struct MetalContext {
    pub device: Rc<MTLDevice>,
    pub command_queue: MTLCommandQueue,
}

impl MetalContext {
    pub fn new() -> Self {
        let device = Device::system_default().expect("No Metal device found");
        let command_queue = device.new_command_queue();

        MetalContext {
            device: Rc::new(device),
            command_queue,
        }
    }
}
