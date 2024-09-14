use metal::*;

pub struct MetalCommand {
    pub command_buffer: CommandBuffer,
}

impl MetalCommand {
    pub fn new(context: &MetalContext) -> MetalCommand {
        let command_buffer = context.command_queue.new_command_buffer();
        MetalCommand { command_buffer }
    }

    pub fn commit(&self) {
        self.command_buffer.commit();
        self.command_buffer.wait_until_completed();
    }
}
