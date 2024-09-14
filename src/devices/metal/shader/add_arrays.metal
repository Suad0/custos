// shader/add_arrays.metal
#include <metal_stdlib>
using namespace metal;

kernel void add_arrays(
    device const float* buffer_a [[ buffer(0) ]],
    device const float* buffer_b [[ buffer(1) ]],
    device float* result_buffer [[ buffer(2) ]],
    uint id [[ thread_position_in_grid ]])
{
    result_buffer[id] = buffer_a[id] + buffer_b[id];
}
