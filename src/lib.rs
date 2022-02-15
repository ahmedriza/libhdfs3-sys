#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Include the bindings directly instead of autogenerating them
// at build time. This is because some systems may not have the
// required clang runtime libraries such as `libclangAST.so` etc
// that are used by `bindgen` to parse C and C++ headers.
// include!(concat!(env!("OUT_DIR"), "/hdfs3_bindings.rs"));
include!("hdfs3_bindings.rs");

pub mod err;
pub mod hdfs3;
