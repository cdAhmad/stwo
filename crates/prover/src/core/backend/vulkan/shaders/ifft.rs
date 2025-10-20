vulkano_shaders::shader! {
    ty: "compute",
    src: r#"
#version 450

layout(local_size_x = 256) in;

// --- Buffer Bindings ---
layout(set = 0, binding = 0) buffer ValuesBuffer {
    uint values[];
} values_buf; // 必须命名

layout(set = 0, binding = 1) buffer TwiddlesBuffer {
    uint all_twiddles[];
} twiddles_buf; // 必须命名

// --- Push Constants ---
const int MAX_LAYERS = 32;
layout(push_constant) uniform PushConstants {
    uint log_size;
    uint total_layers;
    uint total_size;
    uint inv_n;
    uint twiddles_offset[MAX_LAYERS];
} pc;

// --- 有限域参数 ---
const uint MODULUS = 2147483647u; // 2^31 - 1
const uint INV2    = 1073741824u; // (MODULUS + 1) / 2

// --- 有限域运算 ---
uint fe_add(uint a, uint b) {
    uint res = a + b;
    return res >= MODULUS ? res - MODULUS : res;
}

uint fe_sub(uint a, uint b) {
    return a >= b ? a - b : a + MODULUS - b;
}

// --- 安全模乘（无 uint64_t）---
uint fe_mul(uint a, uint b) {
    uint al = a & 0xFFFFu, ah = a >> 16;
    uint bl = b & 0xFFFFu, bh = b >> 16;

    uint t0 = al * bl;
    uint t1 = al * bh;
    uint t2 = ah * bl;
    uint t3 = ah * bh;

    uint mid = t1 + t2;
    uint carry = (mid < t1) ? 0x10000u : 0u;

    uint low = t0 + ((mid & 0xFFFFu) << 16);
    uint high = t3 + (mid >> 16) + carry + (low >> 31);
    low &= 0x7FFFFFFFu;

    uint x = high + low;
    return (x >= MODULUS) ? x - MODULUS : x;
}

// --- 逆蝴蝶操作 ---
void butterfly_inv(inout uint a, inout uint b, uint t) {
    uint sum = fe_add(a, b);
    uint diff = fe_sub(a, b);
    a = fe_mul(sum, INV2);
    b = fe_mul(diff, t);
}

// --- 主计算 ---
void main() {
    uint gid = gl_GlobalInvocationID.x;
    if (gid >= pc.total_size / 2u) return;

    // 逐层处理
    for (uint layer = 0; layer < pc.total_layers; layer++) {
        uint step = 1u << layer;
        uint idx0 = (gid / step) * (step * 2u) + (gid % step);
        uint idx1 = idx0 + step;
        if (idx1 >= pc.total_size) continue;

        uint t;
        if (layer == 0) {
            uint pair_idx = gid >> 1;
            uint base_t = twiddles_buf.all_twiddles[pair_idx];
            t = (gid & 1u) == 0u ? base_t : (MODULUS - base_t);
        } else {
            uint local_idx = gid / step;
            t = twiddles_buf.all_twiddles[pc.twiddles_offset[layer] + local_idx];
        }

        uint a = values_buf.values[idx0];
        uint b = values_buf.values[idx1];
        butterfly_inv(a, b, t);
        values_buf.values[idx0] = a;
        values_buf.values[idx1] = b;
    }

    // 归一化
    if (gid < pc.total_size) {
        values_buf.values[gid] = fe_mul(values_buf.values[gid], pc.inv_n);
    }
}
    "#
}
