//! Reserve stack space for source-backed browser game calls.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        // The default 1 MiB WebAssembly stack is smaller than nested startup,
        // snapshot and script calls on the complete regional game. Reserve a
        // bounded 16 MiB before the heap so those calls cannot overwrite it.
        println!("cargo:rustc-link-arg-bin=crystal-bevy=-zstack-size=16777216");
    }
}
