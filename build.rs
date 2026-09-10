fn main() {
    let compiler = std::env::var("CXX").unwrap_or_else(|_| "c++".into());
    let profile = if std::env::var("PROFILE").as_deref() == Ok("release") {
        "Release"
    } else {
        "Debug"
    };
    let native = cmake::Config::new("cpp")
        .define("BUILD_TESTING", "OFF")
        .define("HELIX_SANITIZERS", "OFF")
        .define("CMAKE_CXX_COMPILER", &compiler)
        .profile(profile)
        .build();
    cxx_build::bridge("src/bridge.rs")
        .file("cpp/ffi/adapter.cpp")
        .include("cpp/include")
        .include("cpp/ffi")
        .compiler(compiler)
        .std("c++20")
        .flag("-ffp-contract=off")
        .warnings(true)
        .compile("helix_bridge");
    println!("cargo:rustc-link-search=native={}/lib", native.display());
    println!("cargo:rustc-link-lib=static=helix_kernel");
    println!("cargo:rerun-if-env-changed=CXX");
    for path in [
        "cpp/CMakeLists.txt",
        "cpp/src/kernel.cpp",
        "cpp/include/helix/kernel.hpp",
        "cpp/ffi/adapter.hpp",
        "cpp/ffi/adapter.cpp",
        "src/bridge.rs",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }
}
