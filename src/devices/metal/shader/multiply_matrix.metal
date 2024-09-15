kernel void matrix_multiply_optimized(const device Matrix& A [[buffer(0)]],
                                      const device Matrix& B [[buffer(1)]],
                                      device Matrix& C [[buffer(2)]],
                                      uint2 id [[thread_position_in_grid]],
                                      uint2 local_id [[thread_position_in_threadgroup]],
                                      threadgroup float tile_A[TILE_SIZE][TILE_SIZE],
                                      threadgroup float tile_B[TILE_SIZE][TILE_SIZE]) {

    float sum = 0.0;
    for (uint t = 0; t < (A.cols + TILE_SIZE - 1) / TILE_SIZE; ++t) {
        // Load tiles into shared memory
        if (t * TILE_SIZE + local_id.x < A.cols && id.y < C.rows)
            tile_A[local_id.y][local_id.x] = A.elements[id.y * A.cols + t * TILE_SIZE + local_id.x];
        else
            tile_A[local_id.y][local_id.x] = 0.0;

        if (t * TILE_SIZE + local_id.y < B.rows && id.x < C.cols)
            tile_B[local_id.y][local_id.x] = B.elements[(t * TILE_SIZE + local_id.y) * B.cols + id.x];
        else
            tile_B[local_id.y][local_id.x] = 0.0;

        threadgroup_barrier(mem_flags::mem_threadgroup);

        // Perform the dot product on the tile
        for (uint i = 0; i < TILE_SIZE; ++i) {
            sum += tile_A[local_id.y][i] * tile_B[i][local_id.x];
        }

        threadgroup_barrier(mem_flags::mem_threadgroup);
    }

    if (id.x < C.rows && id.y < C.cols) {
        C.elements[id.x * C.cols + id.y] = sum;
    }
}
