#!/usr/bin/env bash
set -euo pipefail
export PATH="/root/.cargo/bin:$PATH"
cd "$(dirname "$0")/.."
mkdir -p artifacts/g0-smoke/src artifacts/g0-smoke/native
cd artifacts/g0-smoke
cat > Cargo.toml <<'EOF'
[package]
name = "helix-g0"
version = "0.0.0"
edition = "2021"
[dependencies]
cxx = "1"
[build-dependencies]
cxx-build = "1"
cmake = "0.1"
EOF
cat > native/CMakeLists.txt <<'EOF'
cmake_minimum_required(VERSION 3.20)
project(helix_g0 LANGUAGES CXX)
add_library(g0 STATIC kernel.cpp)
target_compile_features(g0 PUBLIC cxx_std_20)
install(TARGETS g0 ARCHIVE DESTINATION lib)
EOF
cat > native/kernel.cpp <<'EOF'
#include <span>
double sum(const double* data, unsigned long size) {
    double result=0;
    for (double value : std::span<const double>(data, size)) result+=value;
    return result;
}
EOF
cat > native/adapter.hpp <<'EOF'
#pragma once
#include "rust/cxx.h"
double sum(const double*, unsigned long);
inline double sum_slice(rust::Slice<const double> values) {
    return sum(values.data(), values.size());
}
EOF
cat > build.rs <<'EOF'
fn main() {
    let dst = cmake::Config::new("native").build();
    cxx_build::bridge("src/main.rs").include("native").std("c++20").compile("g0-bridge");
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=g0");
}
EOF
cat > src/main.rs <<'EOF'
#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("adapter.hpp");
        fn sum_slice(values: &[f64]) -> Result<f64>;
    }
}
fn main() {
    assert_eq!(ffi::sum_slice(&[1., 2., 3.]).unwrap(), 6.);
    println!("G0 mixed Rust/CXX/CMake/C++20 smoke PASS");
}
EOF
cargo run -j 2
rustc --version
cargo --version
g++ --version
cmake --version
python3 --version
