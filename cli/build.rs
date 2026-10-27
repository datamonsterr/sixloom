// include_dir! does not tell Cargo about the files it embeds, so a new or edited
// skill would otherwise ship a stale binary. Rebuild whenever packs/ or templates/ change.
fn main() {
    println!("cargo:rerun-if-changed=../packs");
    println!("cargo:rerun-if-changed=../templates");
}
