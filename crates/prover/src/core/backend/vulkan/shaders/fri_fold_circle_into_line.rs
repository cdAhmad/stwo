vulkano_shaders::shader! {
    ty: "compute",
    src: r#"
#version 450
#extension GL_ARB_gpu_shader_int64 : enable
layout(local_size_x = 256) in;

// --- Buffer Bindings ---
layout(set = 0, binding = 0) buffer ValuesBuffer {
    uvec4 values[];
} values_buf; // 必须命名
 layout(set = 0, binding = 1) buffer DstBuffer {
    uvec4 values[];
} dst_buf; // 必须命名

layout(set = 0, binding = 2) buffer TwiddlesBuffer {
    uint all_twiddles[];
} twiddles_buf; // 必须命名



// --- Push Constants ---
layout(push_constant) uniform PushConstants {
    uvec4 alpha;
    uvec4 alpha_sq;
    uint total_pairs;   // N/2，即输出长度
} pc;

// --- 有限域参数 ---
const uint MODULUS =  2147483647u; // 2^31 - 1

// --- 有限域运算 ---
uint fe_add(uint a, uint b) {
    uint res = a + b;
    return res - (MODULUS * uint(res >= MODULUS));
}

// --- 有限域运算 ---
uvec2 fe_add_vec2(uvec2 a, uvec2 b) {
    return uvec2(fe_add(a.x, b.x), fe_add(a.y, b.y));
}

uvec4 fe_add_vec4(uvec4 a, uvec4 b) {
    return uvec4(fe_add(a.x, b.x),
                 fe_add(a.y, b.y),
                 fe_add(a.z, b.z),
                 fe_add(a.w, b.w));
}

uint fe_sub(uint a, uint b) {
     uint diff = a - b;
    // 如果 diff 为负，mask 为全1，否则为0
    uint mask = uint(int(diff) >> 31);
    return diff + (mask & MODULUS);
}

uvec2 fe_sub_vec2(uvec2 a, uvec2 b) {
    return uvec2(fe_sub(a.x, b.x), fe_sub(a.y, b.y));
}    
uvec4 fe_sub_vec4(uvec4 a, uvec4 b) {
return uvec4(fe_sub(a.x, b.x),
                 fe_sub(a.y, b.y),
                 fe_sub(a.z, b.z),
                 fe_sub(a.w, b.w));
    }   

  // --- 安全模乘 ---
uint fe_mul(uint a, uint b) {
  uint64_t p = uint64_t(a) * uint64_t(b);
    uint r = uint((p & 2147483647u) + (p >> 31));
    return min(r, r - 2147483647u);  // GLSL的min是硬件加速的
}

 
// --- 复数有限域乘 ---
uvec2 fe_mul_vec2(uvec2 a, uvec2 b) {
uint c=fe_sub(fe_mul(a.x, b.x) ,fe_mul(a.y, b.y));
  uint d=fe_add(fe_mul(a.x, b.y) , fe_mul(a.y, b.x));
   return  uvec2(c,d);
}

const uvec2 R=uvec2(2, 1);


uvec4 fe_mul_vec4(uvec4 a, uvec4 b) {
     uvec2 a0=uvec2(a[0],a[1]);
     uvec2 a1=uvec2(a[2],a[3]);
     uvec2 b0=uvec2(b[0],b[1]);
     uvec2 b1=uvec2(b[2],b[3]);
     uvec2 c0= fe_add_vec2(fe_mul_vec2(a0,b0),fe_mul_vec2(R,fe_mul_vec2(a1,b1)));
     uvec2 c1= fe_add_vec2(fe_mul_vec2(a0,b1),fe_mul_vec2(a1,b0));
     return uvec4(c0.x,c0.y,c1.x,c1.y);
}



uvec4 fe_mul_vec(uvec4 a, uint b) {
     return uvec4(fe_mul(a[0],b),
                  fe_mul(a[1],b),
                  fe_mul(a[2],b),
                  fe_mul(a[3],b));
}
 

// --- 逆蝴蝶操作并融合输出 ---
void main() {
    uint gid = gl_GlobalInvocationID.x;

    // ✅ 边界检查：确保 gid < total_pairs
    if (gid >= pc.total_pairs) {
        return;
    }

    uint idx0 = gid * 2;
    uint idx1 = gid * 2 + 1;

    uvec4 f_x     = values_buf.values[idx0];
    uvec4 f_neg_x = values_buf.values[idx1];

    // ✅ 获取 twiddle 因子 t
    uint t = twiddles_buf.all_twiddles[gid]; // 假设 all_twiddles.length >= total_pairs

    // ✅ 逆蝴蝶
    uvec4 a = fe_add_vec4(f_x, f_neg_x);
    uvec4 b = fe_mul_vec(fe_sub_vec4(f_x, f_neg_x),t);

    uvec4 f_prime = fe_add_vec4(a,fe_mul_vec4(b, pc.alpha));

   dst_buf.values[gid] =fe_add_vec4(fe_mul_vec4(dst_buf.values[gid],pc.alpha_sq),f_prime);
}
    "#
}
