extern crate cmake;
use cmake::Config;

fn main() {
    let dst = Config::new("src/true_libopenvpn3")
        .define("COMPILE_TARGET", "DESKTOP_x86_64")
        //.define("FLAVOR", "DESKTOP")
        //.define("LIBOPENVPN3_NOT_BUILD_EXAMPLES", "TRUE")
        .build();

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=static=libopenvpn3");
    //println!("cargo:rustc-link-lib=static=ssl");
    println!("cargo:rustc-link-lib=dylib=ssl");
    println!("cargo:rustc-link-lib=dylib=crypto");
    println!("cargo:rustc-link-lib=dylib=lz4");
    println!("cargo:rustc-link-lib=dylib=lzo2");
    println!("cargo:rustc-link-lib=static=tins");

    #[cfg(target_os="android")]
    {
        println!("cargo:rustc-link-lib=static=crypto");
        println!("cargo:rustc-link-lib=static=lzo");
        println!("cargo:rustc-link-lib=static=lz4");
        println!("cargo:rustc-link-lib=static=ssl");
    }
}
