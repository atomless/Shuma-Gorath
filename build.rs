use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Detect if building for WASM (Spin production)
    let target = env::var("TARGET").unwrap_or_default();
    let is_wasm = target.starts_with("wasm32");
    let cargo_toml = Path::new("Cargo.toml");
    let orig = fs::read_to_string(cargo_toml).expect("read Cargo.toml");
    let mut lines: Vec<_> = orig.lines().collect();
    let mut lib_idx = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("[lib]") {
            lib_idx = Some(i);
            break;
        }
    }
    if let Some(idx) = lib_idx {
        // Find crate-type line
        let crate_type_idx = idx + 1;
        if crate_type_idx < lines.len() && lines[crate_type_idx].trim().starts_with("crate-type") {
            if is_wasm {
                lines[crate_type_idx] = "crate-type = [\"cdylib\"]";
            } else {
                lines[crate_type_idx] = "crate-type = [\"rlib\"]";
            }
        }
        let new_contents = lines.join("\n");
        fs::write(cargo_toml, new_contents).expect("write Cargo.toml");
    }
}
