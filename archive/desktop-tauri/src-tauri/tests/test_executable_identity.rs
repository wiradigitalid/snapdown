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

#[test]
fn tauri_configuration_has_snapdown_product_and_snapdown_editor_window_title() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let tauri_conf_path = Path::new(manifest_dir).join("tauri.conf.json");
    let tauri_conf_content = fs::read_to_string(&tauri_conf_path)
        .unwrap_or_else(|e| panic!("Failed to read tauri.conf.json at {tauri_conf_path:?}: {e}"));

    let conf: serde_json::Value = serde_json::from_str(&tauri_conf_content)
        .unwrap_or_else(|e| panic!("Failed to parse tauri.conf.json: {e}"));

    let product_name = conf["productName"]
        .as_str()
        .expect("productName must be present");
    assert_eq!(
        product_name, "Snapdown",
        "productName in tauri.conf.json must be 'Snapdown'"
    );

    let windows = conf["app"]["windows"]
        .as_array()
        .expect("app.windows must be an array");
    let main_window = windows
        .iter()
        .find(|w| w["label"].as_str() == Some("main"))
        .expect("main window must exist");

    assert_eq!(
        main_window["title"].as_str(),
        Some("Snapdown Editor"),
        "FR-27 / DEC-003: Main window title in tauri.conf.json must be 'Snapdown Editor'"
    );
}
