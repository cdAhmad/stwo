vulkano_shaders::shader! {
                ty: "compute",
                src: r#"
#version 450

const uint P = 2147483647u; // 2^31 - 1

// 安全计算 (a * b) % P，仅使用 u32
uint mul_mod(uint a, uint b) {
    uint res = 0;
    a %= P;
    b %= P;
    while (b > 0) {
        if ((b & 1)!=0) {
            res = (res + a) % P;
        }
        a = (a << 1) % P;  // a *= 2
        b >>= 1;
    }
    return res;
}

uint mod_exp(uint base, uint exp) {
    uint result = 1;
    while (exp > 0) {
        if ((exp & 1) != 0) {
            result = mul_mod(result, base);
        }
        base = mul_mod(base, base);
        exp >>= 1;
    }
    return result;
}

layout(std430, set = 0, binding = 0) buffer IoBuffer {
    uint data[];
} io_buffer;

layout(local_size_x = 256) in;

void main() {
    uint i = gl_GlobalInvocationID.x;
    if (i >= io_buffer.data.length()) return;

    uint x = io_buffer.data[i];
    if (x == 0) {
        io_buffer.data[i] = 0;
    } else {
        io_buffer.data[i] = mod_exp(x, P - 2);
    }
}"# 
}
