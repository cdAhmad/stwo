vulkano_shaders::shader! {
    ty: "compute",
    src: r#"
    #version 450

const uvec4 P_VEC = uvec4(2147483647u); // 2^31 - 1

uvec4 add_mod4(uvec4 a, uvec4 b) {
    uvec4 c = a + b;
    return min(c, c - P_VEC);
}

layout(std430, binding = 0)  buffer ColumnBlock {
    uvec4 data[];
} column;

layout(std430, binding = 1) readonly buffer OtherBlock {
    uvec4 data[];
} other;

layout(local_size_x = 256) in;

void main() {
    uint i = gl_GlobalInvocationID.x;
    if (i >= column.data.length()) return;
    column.data[i] = add_mod4(column.data[i], other.data[i]); // ✅ 正确访问
}
     "#
}
