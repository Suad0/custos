use metal::*;

pub struct MetalPipeline {
    pub pipeline_state: ComputePipelineState,
}

impl MetalPipeline {
    pub fn new(context: &MetalContext, shader: &MetalShader) -> MetalPipeline {
        let pipeline_state = context.device.new_compute_pipeline_state_with_function(&shader.function).unwrap();
        MetalPipeline { pipeline_state }
    }
}
