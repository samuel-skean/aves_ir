#![allow(non_upper_case_globals, non_camel_case_types, unused)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(not(unix))]
compile_error!("This crate only works on Unix!");