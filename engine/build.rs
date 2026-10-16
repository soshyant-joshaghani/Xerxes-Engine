//! Android: Bevy's audio uses cpal → oboe, which is C++. NativeActivity loads our library with
//! no C++ runtime around, so link the NDK's libc++ statically (nothing extra to ship in the
//! APK); without it the app dies on start with `cannot locate symbol "__cxa_pure_virtual"`.
//! Every game links the engine, so this covers them too. (Not `.cargo/config.toml` rustflags:
//! cargo-apk sets its own and they would be ignored.)
//!
//! Editor: the template games (`games/__templates__/*`) are packed into the editor binary
//! (`$OUT_DIR/templates.zip`), so New Project works on any machine without the repo or the
//! backend. On Windows the exe gets its icon (`platform/windows/xerxes.rc`, resource 1), which the
//! window and taskbar use too.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        println!("cargo:rustc-link-lib=c++_static");
        println!("cargo:rustc-link-lib=c++abi");
    }
    #[cfg(feature = "editor")]
    pack_templates();
    #[cfg(feature = "editor")]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=windows/xerxes.rc");
        println!("cargo:rerun-if-changed=windows/xerxes.ico");
        embed_resource::compile("platform/windows/xerxes.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("compile platform/windows/xerxes.rc");
    }
}

#[cfg(feature = "editor")]
fn pack_templates() {
    use std::path::{Path, PathBuf};
    use xerxes_build::scaffold;

    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let dir = manifest.join("../games/__templates__");
    let mut templates = Vec::new();
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|e| e.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    entries.sort();
    for template in entries.iter().filter(|p| p.join("Cargo.toml").is_file()) {
        // Watch what gets packed (not target/, which changes on every template build).
        for entry in std::fs::read_dir(template).into_iter().flatten().flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !scaffold::EXCLUDED.contains(&name.as_str()) {
                println!("cargo:rerun-if-changed={}", entry.path().display());
            }
        }
        let name = template.file_name().unwrap().to_string_lossy().into_owned();
        let files = scaffold::read_dir(template).unwrap_or_else(|e| panic!("template {name}: {e}"));
        templates.push((name, files));
    }
    let packed = scaffold::pack(&templates).expect("pack templates");
    let out = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("templates.zip");
    std::fs::write(out, packed).expect("write templates.zip");
}
