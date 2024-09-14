use metal::*;

pub struct Command {
    pub command_buffer: MTLCommandBuffer,
    pub encoder: MTLComputeCommandEncoder,
}

impl Command {
    pub fn new(command_queue: &MTLCommandQueue, pipeline_state: &MTLComputePipelineState) -> Self {
        let command_buffer = command_queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(pipeline_state);
        Command {
            command_buffer,
            encoder,
        }
    }

    pub fn dispatch(&mut self, size: usize, threads_per_group: usize) {
        let grid_size = MTLSize::new(size as u64, 1, 1);
        let threadgroup_size = MTLSize::new(threads_per_group as u64, 1, 1);
        self.encoder.dispatch_threads(grid_size, threadgroup_size);
    }

    pub fn end_encoding(&mut self) {
        self.encoder.end_encoding();
    }

    pub fn commit(&mut self) {
        self.command_buffer.commit();
        self.command_buffer.wait_until_completed();
    }
}
