// bit_reverse.metal
#include <metal_stdlib>
using namespace metal;

kernel void bit_reverse_u32(
    const device uint* input [[buffer(0)]],
    device uint* output [[buffer(1)]],
    constant uint& log_n [[buffer(2)]],
    uint gid [[thread_position_in_grid]]
) {
    uint n = 1u << log_n;
    if (gid >= n) return;

    // Compute bit-reversed index of gid
    uint rev = 0;
    uint temp = gid;
    for (uint i = 0; i < log_n; i++) {
        rev = (rev << 1) | (temp & 1u);
        temp >>= 1;
    }

    output[gid] = input[rev];
}