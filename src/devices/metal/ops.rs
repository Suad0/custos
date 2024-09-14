use crate::metal::context::MetalContext;
use metal::*;

pub fn add_arrays(
    ctx: &MetalContext,
    buffer_a: &MTLBuffer,
    buffer_b: &MTLBuffer,
    result_buffer: &MTLBuffer,
    size: usize,
) {
    let source = include_str!("shader/add_arrays.metal");

    let library = ctx
        .device
        .new_library_with_source(source, &CompileOptions::new())
        .expect("Failed to compile Metal shader");

    let kernel = library
        .get_function("add_arrays", None)
        .expect("Failed to find function in Metal shader");

    let pipeline_state = ctx
        .device
        .new_compute_pipeline_state_with_function(&kernel)
        .expect("Failed to create compute pipeline state");

    let command_buffer = ctx.command_queue.new_command_buffer();
    let encoder = command_buffer.new_compute_command_encoder();

    encoder.set_compute_pipeline_state(&pipeline_state);
    encoder.set_buffer(0, Some(buffer_a), 0);
    encoder.set_buffer(1, Some(buffer_b), 0);
    encoder.set_buffer(2, Some(result_buffer), 0);

    let grid_size = MTLSize::new(size as u64, 1, 1);
    let threadgroup_size = MTLSize::new(pipeline_state.max_total_threads_per_threadgroup(), 1, 1);
    encoder.dispatch_threads(grid_size, threadgroup_size);

    encoder.end_encoding();
    command_buffer.commit();
    command_buffer.wait_until_completed();
}
