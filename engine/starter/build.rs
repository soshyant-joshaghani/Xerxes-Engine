//! Turns `assets/` into Rust modules and packs the art into bundles (`public/bundles/*.zip`).

fn main() {
    xerxes_build::generate();
}
