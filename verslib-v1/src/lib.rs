include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub fn version() -> i32 {
    unsafe { verslib_version() }
}
