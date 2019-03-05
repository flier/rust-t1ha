use rustc_version::{version_meta, Channel};

fn main() {
    if version_meta().unwrap().channel == Channel::Nightly {
        println!("cargo:rustc-cfg=nightyly");
    }

    if cfg!(target_arch = "x86") || cfg!(target_arch = "x86_64") {
        println!("cargo:rustc-cfg=unaligned_access");
    }
}
