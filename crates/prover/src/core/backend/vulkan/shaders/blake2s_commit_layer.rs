vulkano_shaders::shader! {
     ty: "compute",
    src: r#"
#version 450

#extension GL_EXT_shader_8bit_storage : require
#extension GL_EXT_shader_explicit_arithmetic_types : require

const uint BLOCK_SIZE_WORDS = 16u; // 64 bytes = 16 uints
const uint HASH_SIZE = 32u;

const uint IV[8] = {
    0x6A09E667u, 0xBB67AE85u, 0x3C6EF372u, 0xA54FF53Au,
    0x510E527Fu, 0x9B05688Cu, 0x1F83D9ABu, 0x5BE0CD19u
};

const uint SIGMA[10][16] = {
    { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,10,11,12,13,14,15 },
    {14,10, 4, 8, 9,15,13, 6, 1,12, 0, 2,11, 7, 5, 3 },
    {11, 8,12, 0, 5, 2,15,13,10,14, 3, 6, 7, 1, 9, 4 },
    { 7, 9, 3, 1,13,12,11,14, 2, 6, 5,10, 4, 0,15, 8 },
    { 9, 0, 5, 7, 2, 4,10,15,14, 1,11,12, 6, 8, 3,13 },
    { 2,12, 6,10, 0,11, 8, 3, 4,13, 7, 5,15,14, 1, 9 },
    {12, 5, 1,15,14,13, 4,10, 0, 7, 6, 3, 9, 2, 8,11 },
    {13,11, 7,14,12, 1, 3, 9, 5, 0,15, 4, 8, 6, 2,10 },
    { 6,15,14, 9,11, 3, 0, 8,12, 2,13, 7, 1, 4,10, 5 },
    {10, 2, 8, 4, 7, 6, 1, 5,15,11, 9,14, 3,12,13, 0 }
};

struct blake2s_state {
    uint h[8];
    uint t[2];          // total bytes (not words!)
    uint buf[16];       // 16 words = 64 bytes
    uint wordlen;       // number of uints in buf (0..16)
};

uint rotr(uint x, uint n) {
    return (x >> n) | (x << (32u - n));
}

void G(inout uint a, inout uint b, inout uint c, inout uint d, uint m_i, uint m_j) {
    uint x = a, y = b, z = c, w = d;
    x += y + m_i;   w = rotr(w ^ x, 16u);
    z += w;         y = rotr(y ^ z, 12u);
    x += y + m_j;   w = rotr(w ^ x, 8u);
    z += w;         y = rotr(y ^ z, 7u);
    a = x; b = y; c = z; d = w;
}

void blake2s_init(out blake2s_state S) {
    for (int i = 0; i < 8; ++i) S.h[i] = IV[i];
    S.h[0] ^= 0x01010000u ^ 32u; // outlen=32
    S.t[0] = 0u; S.t[1] = 0u;
    S.wordlen = 0u;
}

void blake2s_compress(inout blake2s_state S, bool is_last) {
    uint v[16];
    for (int i = 0; i < 8; ++i) {
        v[i] = S.h[i];
        v[i+8] = IV[i];
    }

    v[12] ^= S.t[0];
    v[13] ^= S.t[1];
    if (is_last) v[14] ^= 0xFFFFFFFFu;

    for (int r = 0; r < 10; ++r) {
        G(v[0],v[4],v[8],v[12],S.buf[SIGMA[r][0]],S.buf[SIGMA[r][1]]);
        G(v[1],v[5],v[9],v[13],S.buf[SIGMA[r][2]],S.buf[SIGMA[r][3]]);
        G(v[2],v[6],v[10],v[14],S.buf[SIGMA[r][4]],S.buf[SIGMA[r][5]]);
        G(v[3],v[7],v[11],v[15],S.buf[SIGMA[r][6]],S.buf[SIGMA[r][7]]);
        G(v[0],v[5],v[10],v[15],S.buf[SIGMA[r][8]],S.buf[SIGMA[r][9]]);
        G(v[1],v[6],v[11],v[12],S.buf[SIGMA[r][10]],S.buf[SIGMA[r][11]]);
        G(v[2],v[7],v[8],v[13],S.buf[SIGMA[r][12]],S.buf[SIGMA[r][13]]);
        G(v[3],v[4],v[9],v[14],S.buf[SIGMA[r][14]],S.buf[SIGMA[r][15]]);
    }

    for (int i = 0; i < 8; ++i)
        S.h[i] ^= v[i] ^ v[i+8];
}

// 直接追加一个 uint（小端，4 字节）
void blake2s_update_u32(inout blake2s_state S, uint val) {
    S.buf[S.wordlen] = val;
    S.wordlen++;

    if (S.wordlen == BLOCK_SIZE_WORDS) {
        S.t[0] += 64u; // 16 words = 64 bytes
        blake2s_compress(S, false);
        S.wordlen = 0u;
    }
}

// 批量设置整个块（用于子哈希）
void blake2s_set_block(inout blake2s_state S, uint block[16]) {
    // 假设当前 buf 为空
    for (int i = 0; i < 16; ++i) S.buf[i] = block[i];
    S.wordlen = 16u;
    S.t[0] = 64u; // 子哈希共 64 字节
    blake2s_compress(S, false);
    S.wordlen = 0u;
}

void blake2s_final(inout blake2s_state S, out uint8_t hash_out[32]) {
    // Add final bytes count
    S.t[0] += S.wordlen * 4u;

    // Pad with zeros to full block
    for (uint i = S.wordlen; i < 16u; ++i) {
        S.buf[i] = 0u;
    }

    blake2s_compress(S, true);

    // Output as little-endian bytes
    for (uint i = 0; i < 32u; ++i) {
        hash_out[i] = uint8_t((S.h[i/4] >> (8u * (i%4))) & 0xFFu);
    }
}

// ===========================================================
//                     主计算入口
// ===========================================================

layout(local_size_x = 256) in;


layout(std430, binding = 0) readonly buffer Columns    { uint data[];    } columns;
layout(std430, binding = 1) writeonly buffer OutputHashes { uint8_t data[]; } out_hashes;
layout(std430, binding = 2) readonly buffer PrevHashes { uint8_t data[]; } prev_hashes;

layout(push_constant) uniform Params {
    uint log_size;
    uint num_columns;
} params;

void main() {
    uint i = gl_GlobalInvocationID.x;
    uint layer_size = 1u << params.log_size;
    if (i >= layer_size) return;

    blake2s_state state;
    blake2s_init(state);

    // (1) 处理子哈希（64 字节 = 16 uints）
        uint child_block[16];
        uint offset = (2u * i) * 32u; // 2 children × 32 bytes each

        // 将 64 字节转换为 16 个小端 uint
        for (int j = 0; j < 16; ++j) {
            child_block[j] =
                (prev_hashes.data[offset + j*4 + 0] & 0xFFu) |
                ((prev_hashes.data[offset + j*4 + 1] & 0xFFu) << 8u) |
                ((prev_hashes.data[offset + j*4 + 2] & 0xFFu) << 16u) |
                ((prev_hashes.data[offset + j*4 + 3] & 0xFFu) << 24u);
        }
        blake2s_set_block(state, child_block);

    // (2) 处理列数据（每个 uint 对齐写入）
    for (uint col = 0; col < params.num_columns; ++col) {
        uint val = columns.data[col * layer_size + i];
        blake2s_update_u32(state, val);
    }

    // (3) Finalize and output
    uint8_t hash[32];
    blake2s_final(state, hash);

    uint out_off = i * 32u;
    for (uint j = 0; j < 32u; ++j) {
        out_hashes.data[out_off + j] = hash[j];
    }
}
    "#
}
