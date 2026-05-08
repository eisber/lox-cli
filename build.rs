fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    match std::env::var("CARGO_CFG_TARGET_ENV").as_deref() {
        Ok("msvc") => println!("cargo:rustc-link-arg-bin=lox=/STACK:8388608"),
        Ok("gnu") => println!("cargo:rustc-link-arg-bin=lox=-Wl,--stack,8388608"),
        _ => {}
    }
}
