use metal::*;

pub struct MetalOperation {
    // Operation-specific information
}

impl MetalOperation {
    pub fn dispatch(encoder: &ComputeCommandEncoder, thread_groups: MTLSize, threads_per_group: MTLSize) {
        encoder.dispatch_thread_groups(thread_groups, threads_per_group);
    }
}
