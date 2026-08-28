fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rerun-if-changed=src/calendar_bridge.m");
        println!("cargo:rustc-link-lib=framework=EventKit");
        println!("cargo:rustc-link-lib=framework=Foundation");
        cc::Build::new()
            .file("src/calendar_bridge.m")
            .flag("-fobjc-arc")
            .compile("calendar_bridge");
    }
    tauri_build::build();
}
