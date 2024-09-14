use metal::*;

pub struct MetalDescriptor {
    // Define any necessary state for resource binding
}

impl MetalDescriptor {
    pub fn set_buffer(encoder: &ComputeCommandEncoder, buffer: &Buffer, index: usize) {
        encoder.set_buffer(index as u64, Some(buffer), 0);
    }
}
