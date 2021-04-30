fn main() {
    println!("cargo:rustc-link-lib=static=dragon");
    println!("cargo:rustc-link-search=/usr/local/mips64-elf/lib")
}