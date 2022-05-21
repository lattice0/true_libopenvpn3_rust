extern crate cmake;
use cmake::Config;
use std::env;

fn main() {
    let mut dst = Config::new("src/true_libopenvpn3");
    
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_os=="android" {
        if target_arch == "x86" {
            dst.define("ANDROID_ABI", "x86");
        } else if target_arch == "x86_64" {
            dst.define("ANDROID_ABI", "x86_64");
        } else if target_arch == "arm" {
            //TODO: is armeabi-v7a with NEON needed?
            dst.define("ANDROID_ABI", "armeabi-v7a");
        } else if target_arch == "aarch64" {
            dst.define("ANDROID_ABI", "arm64-v8a");
        } else {
            panic!("unsupported target_arch: {:?}", target_arch);
        }
    } else {
        //panic!("not android");
    }
    let dst = dst.build();

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=static=libopenvpn3");
    //println!("cargo:rustc-link-lib=static=ssl");
    println!("cargo:rustc-link-lib=dylib=ssl");
    println!("cargo:rustc-link-lib=dylib=crypto");
    println!("cargo:rustc-link-lib=dylib=lz4");
    println!("cargo:rustc-link-lib=dylib=lzo2");
    println!("cargo:rustc-link-lib=static=tins");

    if cfg!(target_os = "android") {
        println!("cargo:rustc-link-lib=static=crypto");
        println!("cargo:rustc-link-lib=static=lzo");
        println!("cargo:rustc-link-lib=static=lz4");
        println!("cargo:rustc-link-lib=static=ssl");
    }
}
