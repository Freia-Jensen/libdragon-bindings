fn main() {
    println!("cargo:rustc-link-lib=dragon");
    println!("cargo:rustc-link-search=native=/usr/local/mips64-elf/lib")
}