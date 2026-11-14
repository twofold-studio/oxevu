use std::{env, fs, path::Path};
fn main() -> std::io::Result<()> {
    let root = Path::new("../../apps/web/dist");
    println!("cargo:rerun-if-changed=../../apps/web/dist");
    let mut entries = Vec::new();
    fn visit(root: &Path, dir: &Path, out: &mut Vec<String>) -> std::io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                visit(root, &path, out)?;
            } else {
                let abs = fs::canonicalize(&path)?;
                let key = path
                    .strip_prefix(root)
                    .map_err(std::io::Error::other)?
                    .to_string_lossy();
                out.push(format!(
                    "({key:?}, include_bytes!({:?}) as &'static [u8])",
                    abs
                ));
            }
        }
        Ok(())
    }
    if root.exists() {
        visit(root, root, &mut entries)?;
    }
    fs::write(
        Path::new(&env::var("OUT_DIR").map_err(std::io::Error::other)?).join("assets.rs"),
        format!(
            "pub static ASSETS: &[(&str, &[u8])] = &[{}];",
            entries.join(",")
        ),
    )?;
    Ok(())
}
