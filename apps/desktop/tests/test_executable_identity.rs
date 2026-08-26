use std::fs;
use std::path::Path;

#[test]
fn desktop_crate_declares_exactly_one_binary_named_snapdown() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let cargo_toml_path = Path::new(manifest_dir).join("Cargo.toml");
    let cargo_toml_content = fs::read_to_string(&cargo_toml_path)
        .unwrap_or_else(|e| panic!("Failed to read Cargo.toml at {cargo_toml_path:?}: {e}"));

    // Find all [[bin]] definitions
    let mut bin_names = Vec::new();
    let mut in_bin = false;
    for line in cargo_toml_content.lines() {
        let trimmed = line.trim();
        if trimmed == "[[bin]]" {
            in_bin = true;
            continue;
        }
        if in_bin && trimmed.starts_with('[') {
            in_bin = false;
        }
        if in_bin && trimmed.starts_with("name") {
            if let Some((_, val)) = trimmed.split_once('=') {
                let name = val.trim().trim_matches('"').trim_matches('\'').to_string();
                bin_names.push(name);
            }
        }
    }

    assert_eq!(
        bin_names,
        vec!["Snapdown"],
        "AD-11 / BR-121: Desktop app must declare exactly one binary named 'Snapdown', found: {bin_names:?}"
    );
}
