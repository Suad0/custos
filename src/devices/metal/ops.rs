use metal::*;
use crate::context::MetalContext;

pub fn add_arrays(
    ctx: &MetalContext,
    buffer_a: &Buffer,
    buffer_b: &Buffer,
    result_buffer: &Buffer,
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

#[cfg(test)]
mod tests {
    use super::*;
    use metal::*;
    use std::rc::Rc;

    #[test]
    fn test_add_arrays() {
        // Initialize Metal context
        let context = Rc::new(MetalContext::new());

        // Input data
        let size = 4;
        let data_a: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let data_b: Vec<f32> = vec![5.0, 6.0, 7.0, 8.0];
        let mut result: Vec<f32> = vec![0.0; size];

        // Create buffers
        let buffer_a = context.device.new_buffer_with_data(
            &data_a as *const _ as *const std::ffi::c_void,
            (size * std::mem::size_of::<f32>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );
        let buffer_b = context.device.new_buffer_with_data(
            &data_b as *const _ as *const std::ffi::c_void,
            (size * std::mem::size_of::<f32>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );
        let result_buffer = context.device.new_buffer(
            (size * std::mem::size_of::<f32>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );

        add_arrays(
            &context,
            &buffer_a,
            &buffer_b,
            &result_buffer,
            size,
        );

        let result_data = result_buffer.contents();
        unsafe {
            std::ptr::copy_nonoverlapping(
                result_data as *const f32,
                result.as_mut_ptr(),
                size
            );
        }

        let expected: Vec<f32> = vec![6.0, 8.0, 10.0, 12.0];

        assert_eq!(result, expected);
    }
}
