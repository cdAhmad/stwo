// build.rs
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // 获取目标平台
    let target = env::var("TARGET").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // 确定 SDK（macOS 或 iOS）
    let sdk = if target.contains("apple-darwin") {
        "macosx"
    } else if target.contains("ios") {
        "iphoneos"
    } else {
        // 非 Apple 平台，跳过编译（Metal 仅 Apple 支持）
        return;
    };

    // Shader 源文件路径（相对于 crate root）
    let shader_src = "src/core/backend/metal/shaders/bit_reverse.metal";
    let air_path = out_dir.join("bit_reverse.air");
    let metallib_path = out_dir.join("bit_reverse.metallib");

    // 1. 编译 .metal → .air
    let status = Command::new("xcrun")
        .args(&[
            "-sdk", sdk,
            "metal",
            "-c",
            shader_src,
            "-o",
            air_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run 'xcrun metal'");

    if !status.success() {
        panic!("Metal shader compilation to .air failed");
    }

    // 2. 链接 .air → .metallib
    let status = Command::new("xcrun")
        .args(&[
            "-sdk", sdk,
            "metallib",
            air_path.to_str().unwrap(),
            "-o",
            metallib_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run 'xcrun metallib'");

    if !status.success() {
        panic!("Metal library linking failed");
    }

    // 告诉 Cargo：如果 shader 文件变化，重新构建
    println!("cargo:rerun-if-changed={}", shader_src);
}