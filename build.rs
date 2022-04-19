use std::env;
use std::path::PathBuf;

fn main() {
    build_hdfs3_lib();
    //
    // Generate the Rust wrapper bindings
    // build_hdfs3_ffi();
}

fn build_hdfs3_lib() {
    // Build the native C/C++ libraries
    // Will not be needed if we use system installed versions of libhdfs3. Using pre-installed
    // system version of the libhdfs3 will also save on compile time dependencies of Rust projects
    // as well as significant compilation time.
    //
    // Uncomment the following 4 lines for static linking
    //
    // println!("cargo:rerun-if-changed={}", get_hdfs3_file_path("src/client/hdfs.h"));
    // let dst = cmake::build("libhdfs3");
    // println!("cargo:rustc-link-search=native={}/lib", dst.display());
    // println!("cargo:rustc-link-lib=static=hdfs3");
    // println!("cargo:rustc-link-lib=dylib=stdc++");
    // println!("cargo:rustc-link-lib=dylib=protobuf");
    // println!("cargo:rustc-link-lib=dylib=gsasl");
    // println!("cargo:rustc-link-lib=dylib=uuid");
    // println!("cargo:rustc-link-lib=dylib=xml2");
    // println!("cargo:rustc-link-lib=dylib=krb5");

    // Comment out this, if statically linking
    println!("cargo:rustc-link-lib=dylib=hdfs3");
}

fn get_hdfs3_file_path(filename: &'static str) -> String {
    format!("{}/{}", get_hdfs3_source_dir(), filename)
}

fn get_hdfs3_source_dir() -> &'static str {
    "libhdfs3"
}

#[allow(unused)]
// This is the FFI builder for the Apache Hawq hdfs3 C++ library with its C bindings
fn build_hdfs3_ffi() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes.
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        // The input headers we would like to generate bindings for
        .header("wrapper.h")
        .allowlist_function("nmd.*")
        .allowlist_function("hdfs.*")
        .allowlist_function("hadoop.*")
        .rustified_enum("tObjectKind")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings!");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("hdfs3_bindings.rs"))
        .expect("Couldn't write bindings!");
}

