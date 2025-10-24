vulkano_shaders::shader! {
    ty: "compute",
    src: r#"
#version 450
#extension GL_ARB_gpu_shader_int64 : enable
layout(local_size_x = 256) in;

layout(set = 0, binding = 0) buffer ValuesBuffer {
    uint values[];
} values_buf;

layout(push_constant) uniform PushConstants {
    uint total_size;
    uint inv_n;
} pc;

const uint MODULUS = 2147483647u;

  // --- 安全模乘---
uint fe_mul(uint a, uint b) {
  uint64_t p = uint64_t(a) * uint64_t(b);
    uint r = uint((p & 2147483647u) + (p >> 31));
    return min(r, r - 2147483647u);  // GLSL的min是硬件加速的
}

void main() {
    uint gid = gl_GlobalInvocationID.x;
    if (gid >= pc.total_size) return;
    values_buf.values[gid] = fe_mul(values_buf.values[gid], pc.inv_n);
}

    "#
}
