use std::rc::Rc;
use metal::*;

pub struct MetalContext {
    pub device: Rc<Device>,  // Change here to Rc<Device>
    pub command_queue: CommandQueue,
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
