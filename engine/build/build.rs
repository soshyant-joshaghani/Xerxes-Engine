//! Embeds the engine's add-ons (`engine/addons/<feature>/<name>.rs`) into this crate, so the
//! editor, the backend and the CLI all offer the same add-ons with no files to find at run time.
//! Writes `$OUT_DIR/addons_library.rs`: one `(path, source)` pair per file, in sorted order.

use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let here = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let library = here.join("../addons");
    println!("cargo:rerun-if-changed={}", library.display());

    let mut files = Vec::new();
    collect(&library, &library, &mut files);
    files.sort();

    let mut out = String::from("pub const LIBRARY: &[(&str, &str)] = &[\n");
    for rel in &files {
        let absolute = library.join(rel).canonicalize().unwrap();
        let absolute = absolute.to_string_lossy().replace('\\', "/");
        let absolute = absolute.strip_prefix("//?/").unwrap_or(&absolute);
        out.push_str(&format!("    ({rel:?}, include_str!({absolute:?})),\n"));
    }
    out.push_str("];\n");

    let target = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("addons_library.rs");
    if fs::read_to_string(&target).ok().as_deref() != Some(out.as_str()) {
        fs::write(target, out).unwrap();
    }
}

/// Every `.rs` file inside a feature folder (`hud/hud.rs`), as a `/`-separated relative path.
fn collect(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(read) = fs::read_dir(dir) else { return };
    for entry in read.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, out);
        } else if path.extension().is_some_and(|e| e == "rs") && path.parent() != Some(root) {
            let rel = path.strip_prefix(root).unwrap();
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
}
