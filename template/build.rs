use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=sdk/shims/wrapper.h");
    
    // Some headers might need __MRE_SDK__ and other defines to compile properly.
    // Based on the Makefile, we have:
    // -D _MINIGUI_LIB_ -D _USE_MINIGUIENTRY -D _NOUNIX_ -D _FOR_WNC 
    // -D __MRE_SDK__ -D __MRE_VENUS_NORMAL__ -D __MMI_MAINLCD_240X320__ -D MRE -D GCC -D __MRE_COMPILER_GCC__
    
    let bindings = bindgen::Builder::default()
        .header("sdk/shims/wrapper.h")
        .clang_arg("-Isdk/include")
        .clang_arg("-D_MINIGUI_LIB_")
        .clang_arg("-D_USE_MINIGUIENTRY")
        .clang_arg("-D_NOUNIX_")
        .clang_arg("-D_FOR_WNC")
        .clang_arg("-D__MRE_SDK__")
        .clang_arg("-D__MRE_VENUS_NORMAL__")
        .clang_arg("-D__MMI_MAINLCD_240X320__")
        .clang_arg("-DGCC")
        .clang_arg("-D__MRE_COMPILER_GCC__")
        .clang_arg("--target=arm-none-eabi")
        .clang_arg("-nostdinc")
        .clang_arg("-I/usr/lib/gcc/arm-none-eabi/10.3.1/include")
        .clang_arg("-I/usr/lib/gcc/arm-none-eabi/10.3.1/include-fixed")
        .clang_arg("-I/usr/arm-none-eabi/include")
        .clang_arg("-I/usr/include/newlib")
        .clang_arg("-I/home/cipl1192/sandbox/llvm/clang+llvm-17.0.6-x86_64-linux-gnu-ubuntu-22.04/lib/clang/17/include")
        .use_core() // Since this is a no_std / embedded-like target (or custom libc)
        .blocklist_type("timeval")
        .blocklist_item("vm_main")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
