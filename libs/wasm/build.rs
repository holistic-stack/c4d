use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    if target.contains("wasm32") {
        cc::Build::new()
            .file("src/libc_stubs.c")
            .compile("libc_stubs");
    }
}
