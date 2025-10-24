vulkano_shaders::shader! {
                ty: "compute",
                src: r#"
#version 450
#extension GL_ARB_gpu_shader_int64 : enable

const uint P = 2147483647u; // 2^31 - 1

// 单指令模乘法 - 利用64位整数
uint mul_mod(uint a, uint b) {
  uint64_t p = uint64_t(a) * uint64_t(b);
    uint r = uint((p & 2147483647u) + (p >> 31));
    return min(r, r - 2147483647u);  // GLSL的min是硬件加速的
}

uint mod_exp_p_minus_2(uint base) {
    // P-2 = 0x7FFFFFFD = 1111111111111111111111111111101 (二进制)
    // 手动展开所有31次迭代
    
    uint result = 1;
    uint b = base;
    
    // 第0位: 1
    result = mul_mod(result, b);  // result *= base^1
    b = mul_mod(b, b);            // b = base^2
    
    // 第1位: 0
    b = mul_mod(b, b);            // b = base^4
    
    // 第2-30位: 都是1 (共29位)
    result = mul_mod(result, b);  // 第2位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第3位  
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第4位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第5位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第6位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第7位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第8位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第9位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第10位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第11位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第12位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第13位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第14位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第15位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第16位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第17位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第18位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第19位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第20位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第21位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第22位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第23位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第24位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第25位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第26位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第27位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第28位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第29位
    b = mul_mod(b, b);
    result = mul_mod(result, b);  // 第30位
    
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
        io_buffer.data[i] = mod_exp_p_minus_2(x);
    }
}"# 
}
