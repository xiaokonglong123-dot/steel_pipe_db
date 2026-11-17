//! 保证 rust-embed 的目录存在，让 `cargo build` 不依赖前端构建顺序。
//!
//! 真实产物由 `cd web && npm run build` 写入；本脚本只在目录缺失时
//! 放一个说明用的占位 index.html，不覆盖任何已存在的文件。

use std::fs;
use std::path::Path;

fn main() {
    let dist = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/dist");
    let index = dist.join("index.html");

    if !index.exists() {
        let _ = fs::create_dir_all(&dist);
        let _ = fs::write(
            &index,
            "<!doctype html><meta charset=\"utf-8\"><title>SteelPipe</title>\n<p>前端尚未构建：cd web && npm ci && npm run build</p>\n",
        );
    }

    println!("cargo:rerun-if-changed=../../web/dist");
}
