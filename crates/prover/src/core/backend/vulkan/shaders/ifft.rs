vulkano_shaders::shader! {
    ty: "compute",
    src: r#"
#version 450
#extension GL_ARB_gpu_shader_int64 : enable
layout(local_size_x = 256) in;

// --- Buffer Bindings ---
layout(set = 0, binding = 0) buffer ValuesBuffer {
    uint values[];
} values_buf; // 必须命名

layout(set = 0, binding = 1) buffer TwiddlesBuffer {
    uint all_twiddles[];
} twiddles_buf; // 必须命名

// --- Push Constants ---
layout(push_constant) uniform PushConstants {
    uint log_n;           // 当前处理的层 (0 ~ total_layers-1)
    uint total_size;      // 总长度 N
    uint twiddles_offset; // 当前层 twiddle 因子的偏移
} pc;

// --- 有限域参数 ---
const uint MODULUS = 2147483647u; // 2^31 - 1

// --- 有限域运算 ---
uint fe_add(uint a, uint b) {
    uint res = a + b;
    return res >= MODULUS ? res - MODULUS : res;
}

uint fe_sub(uint a, uint b) {
    uint res = a + (MODULUS - b);
    return res >= MODULUS ? res - MODULUS : res;
}

// --- 安全模乘（无 uint64_t）---
uint fe_mul(uint a, uint b) {
    uint64_t product = uint64_t(a) * uint64_t(b);
    uint64_t step1 = (product >> 31) + product + 1;
    uint64_t step2 = step1 >> 31;
    uint64_t step3 = step2 + product;
    uint64_t result = step3 & uint64_t(MODULUS); // & P
    return uint(result);
}

// --- 逆蝴蝶操作 ---
void butterfly_inv(inout uint a, inout uint b, uint t) {
    uint sum = fe_add(a, b);
    uint diff = fe_sub(a, b);
    a=sum;
    b = fe_mul(diff, t);
}

void main() {
    uint gid = gl_GlobalInvocationID.x;
    uint step = 1u << pc.log_n;
 
    // 计算蝴蝶对索引
    uint idx0 = (gid / step) * (step * 2u) + (gid % step);
    uint idx1 = idx0 + step;

    // 只有 idx1 有效时才执行蝴蝶操作
    if (idx1 < pc.total_size) {
        uint local_idx = gid / step;
        uint t = twiddles_buf.all_twiddles[pc.twiddles_offset + local_idx];

        uint a = values_buf.values[idx0];
        uint b = values_buf.values[idx1];
        butterfly_inv(a, b, t);
        values_buf.values[idx0] = a;
        values_buf.values[idx1] = b;
}

    
}
    "#
}
