use std::env;
use std::path::PathBuf;

fn main() {
    let vec_flags = vec![];
    build_hdfs3_lib(&vec_flags);
    build_hdfs3_ffi(&vec_flags);
}

// This is the FFI builder for the Apache Hawq hdfs3 C++ library with its C bindings
fn build_hdfs3_ffi(_flags: &[String]) {
    // Tell cargo to tell rustc to link to libhdfs3 dynamically
    // See also: https://doc.rust-lang.org/cargo/reference/build-scripts.html#-sys-packages
    // println!("cargo:rustc-link-search=/opt/libhdfs3/lib");
    // println!("cargo:rustc-link-lib=hdfs3");
    
    // Tell cargo to invalidate the built crate whenever the wrapper changes.
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point to bindgen, and lets you build up options
    // for the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input headers we would like to generate bindings for
        .header("wrapper.h")
        .allowlist_function("nmd.*")
        .allowlist_function("hdfs.*")
        .allowlist_function("hadoop.*")
        .rustified_enum("tObjectKind")
        // Tell cargo to invalidate the built crate whenever any of the included header files
        // changed
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings
        .generate()
        .expect("Unable to generate bindings!");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn build_hdfs3_lib(_flags: &[String]) {
    println!(
        "cargo:rerun-if-changed={}",
        get_hdfs3_file_path("src/client/hdfs.h")
    );

    let dst = cmake::build("libhdfs3");
    // Tell cargo to tell rustc to link to libhdfspp
    // See also: https://doc.rust-lang.org/cargo/reference/build-scripts.html#-sys-packages
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=hdfs3");
    // The following are required when linking statically with libhdfs3
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=dylib=protobuf");
    println!("cargo:rustc-link-lib=dylib=gsasl");
    println!("cargo:rustc-link-lib=dylib=uuid");
    println!("cargo:rustc-link-lib=dylib=xml2");
    println!("cargo:rustc-link-lib=dylib=krb5");
}

fn get_hdfs3_file_path(filename: &'static str) -> String {
    format!("{}/{}", get_hdfs3_source_dir(), filename)
}

fn get_hdfs3_source_dir() -> &'static str {
    "libhdfs3"
}
