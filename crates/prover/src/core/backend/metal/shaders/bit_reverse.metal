// bit_reverse.metal
#include <metal_stdlib>
using namespace metal;

kernel void bit_reverse(
    const device uint* input [[buffer(0)]],
    device uint* output [[buffer(1)]],
    constant uint& log_n [[buffer(2)]],
    uint gid [[thread_position_in_grid]]
) {
    uint n = 1u << log_n;
    if (gid >= n) return;

    // ✅ 使用硬件加速的位反转
    uint rev = __builtin_bitreverse32(gid) >> (32u - log_n);
    output[gid] = input[rev];
}