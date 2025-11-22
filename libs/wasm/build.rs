fn main() {
    cc::Build::new()
        .file("src/libc_stubs.c")
        .compile("libc_stubs");
}
