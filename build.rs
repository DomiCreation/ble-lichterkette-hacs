// macOS stops a program that uses Bluetooth unless its Info.plist says why
// (NSBluetoothAlwaysUsageDescription). A plain program has no bundle, so the
// Info.plist is embedded into the executable's __TEXT,__info_plist section.
fn main() {
    println!("cargo:rerun-if-changed=Info.plist");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let plist = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("Info.plist");
        println!("cargo:rustc-link-arg-bins=-Wl,-sectcreate,__TEXT,__info_plist,{}", plist.display());
    }
}
