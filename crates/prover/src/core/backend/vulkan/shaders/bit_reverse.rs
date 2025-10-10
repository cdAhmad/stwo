vulkano_shaders::shader! {
    ty: "compute",
    src: r#"
#version 450

// 工作组大小（必须与 Rust 中 dispatch 一致）
layout(local_size_x = 256, local_size_y = 1, local_size_z = 1) in;

// 单个可读写缓冲区(in-place)
layout(set = 0, binding = 0) buffer Data {
    uint data[]; // 对应 Rust 的 u32 / BaseField(若为 u32)
};

// 使用 push constant 传递 log_n(高效！)
layout(push_constant) uniform Params {
    uint log_n;
} pc;

// 位反转函数：反转低 `bits` 位
uint bit_reverse(uint x, uint bits) {
    uint r = 0;
    for (uint i = 0; i < bits; ++i) {
        r = (r << 1) | (x & 1u);
        x >>= 1;
    }
    return r;
}

void main() {
    uint n = 1u << pc.log_n;          // 总元素数
    uint i = gl_GlobalInvocationID.x; // 当前线程索引

    // 边界检查（防止越界）
    if (i >= n) {
        return;
    }

    // 计算位反转后的索引
    uint j = bit_reverse(i, pc.log_n);

    // 仅当 j > i 时交换，避免竞态和重复交换
    if (j > i) {
        uint temp = data[i];
        data[i] = data[j];
        data[j] = temp;
    }
}
"#
}
