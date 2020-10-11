fn main() {
    println!("cargo:rustc-cdylib-link-arg=/DEF:driver.def");
}
