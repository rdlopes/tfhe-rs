use std::path::PathBuf;

fn get_linux_distribution_name() -> Option<String> {
    let content = std::fs::read_to_string("/etc/os-release").ok()?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("NAME=") {
            return Some(value.trim_matches('"').to_string());
        }
    }
    None
}

fn main() {
    if let Ok(val) = std::env::var("DOCS_RS") {
        if val.parse::<u32>() == Ok(1) {
            return;
        }
    }

    if std::env::var("_CBINDGEN_IS_RUNNING").is_ok() {
        return;
    }

    println!("cargo::rerun-if-changed=cuda/include");
    println!("cargo::rerun-if-changed=cuda/src");
    println!("cargo::rerun-if-changed=cuda/CMakeLists.txt");
    println!("cargo::rerun-if-changed=src");

    if std::env::consts::OS == "linux" || std::env::consts::OS == "windows" {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR must be set by cargo during build");

        if std::env::consts::OS == "linux" && get_linux_distribution_name().as_deref() != Some("Ubuntu") {
            println!(
                "cargo:warning=This Linux distribution is not officially supported. \
                Only Ubuntu is supported by tfhe-cuda-common at this time. Build may fail\n"
            );
        }

        let mut cmake_config = cmake::Config::new("cuda");

        if cfg!(feature = "profile") {
            cmake_config.define("USE_NVTOOLS", "ON");
        } else {
            cmake_config.define("USE_NVTOOLS", "OFF");
        }

        if cfg!(feature = "debug") {
            cmake_config.define("CMAKE_BUILD_TYPE", "Debug");
        }

        let dest = cmake_config.build();

        println!(
            "cargo:rustc-link-search=native={}",
            dest.join("lib").display()
        );
        if std::env::consts::OS == "windows" {
            println!(
                "cargo:rustc-link-search=native={}",
                dest.join("lib/Release").display()
            );
            println!(
                "cargo:rustc-link-search=native={}",
                dest.join("lib/Debug").display()
            );
        }
        println!("cargo:rustc-link-lib=static=tfhe_cuda_common");

        if std::env::consts::OS == "windows" {
            if let Ok(cuda_path) = std::env::var("CUDA_PATH") {
                println!("cargo:rustc-link-search=native={}\\lib\\x64", cuda_path);
            }
            println!("cargo:rustc-link-lib=cudart");
        } else {
            if pkg_config::Config::new()
                .atleast_version("10")
                .probe("cuda")
                .is_err()
            {
                println!("cargo:rustc-link-search=native=/usr/local/cuda/lib64");
            }
            println!("cargo:rustc-link-lib=cudart");
            println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu/");
            println!("cargo:rustc-link-lib=stdc++");
        }

        // When a build script emits `cargo:KEY=VALUE` and the crate declares
        // `links = "foo"` in Cargo.toml, Cargo exposes it to dependent crates
        // as the env var DEP_FOO_KEY.
        //
        // "include" is not a built-in Cargo directive, just a convention. In this case:
        //   - links = "tfhe_cuda_common" + cargo:include=<path>
        //   - dependents see DEP_TFHE_CUDA_COMMON_INCLUDE=<path>
        let include_dir = PathBuf::from(&manifest_dir).join("cuda/include");
        println!("cargo:include={}", include_dir.display());

        // Same mechanism: dependents see DEP_TFHE_CUDA_COMMON_CHECK_CUDA_DIR.
        let cuda_dir = PathBuf::from(&manifest_dir).join("cuda");
        println!("cargo:check_cuda_dir={}", cuda_dir.display());

        generate_cuda_bind_bindings(&manifest_dir, &include_dir);
    } else {
        panic!(
            "Error: platform not supported, tfhe-cuda-common not built (only Linux and Windows are supported)"
        );
    }
}

fn generate_cuda_bind_bindings(manifest_dir: &str, include_dir: &PathBuf) {
    let header_path = include_dir.join("device.h");
    let build_script_path = PathBuf::from(manifest_dir).join("build.rs");
    let headers = [header_path.to_str().unwrap(), build_script_path.to_str().unwrap()];
    let out_path = PathBuf::from(manifest_dir).join("src").join("cuda_bind.rs");

    let bindings_modified = if out_path.exists() {
        std::fs::metadata(&out_path).unwrap().modified().unwrap()
    } else {
        std::time::SystemTime::UNIX_EPOCH
    };

    let mut headers_modified = bindings_modified;
    for header in &headers {
        println!("cargo:rerun-if-changed={header}");
        let header_modified = std::fs::metadata(header).unwrap().modified().unwrap();
        if header_modified > headers_modified {
            headers_modified = header_modified;
        }
    }

    if headers_modified > bindings_modified {
        let mut builder = bindgen::Builder::default()
            .header(header_path.to_str().unwrap())
            .allowlist_function("cuda_.*")
            .blocklist_type("CUstream_st")
            .blocklist_type("CUevent_st")
            .blocklist_type("cudaStream_t")
            .blocklist_type("cudaEvent_t")
            .clang_arg("-x")
            .clang_arg("c++")
            .clang_arg("-std=c++17")
            .clang_arg(format!("-I{}", include_dir.display()))
            .clang_arg("-I/usr/include")
            .clang_arg("-I/usr/local/include")
            .clang_arg("-I/usr/local/cuda/include");

        if let Ok(cuda_path) = std::env::var("CUDA_PATH") {
            builder = builder.clang_arg(format!("-I{}/include", cuda_path));
        }

        let bindings = builder
            .ctypes_prefix("ffi")
            .raw_line("use crate::ffi;")
            .generate()
            .expect("Unable to generate cuda_bind bindings");

        bindings
            .write_to_file(&out_path)
            .expect("Couldn't write cuda_bind bindings!");
    }
}
