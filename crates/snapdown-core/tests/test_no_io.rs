use cargo_metadata::{DependencyKind, MetadataCommand};

#[test]
fn snapdown_core_has_no_io_dependency() {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("cargo metadata must execute successfully");

    let core_pkg = metadata
        .packages
        .iter()
        .find(|p| p.name == "snapdown-core")
        .expect("snapdown-core package must exist in workspace metadata");

    // Check normal dependencies declared in Cargo.toml
    let normal_deps: Vec<_> = core_pkg
        .dependencies
        .iter()
        .filter(|d| d.kind == DependencyKind::Normal)
        .map(|d| d.name.clone())
        .collect();

    println!("snapdown-core normal dependencies: {:?}", normal_deps);

    // Prohibited crates: any crate related to I/O, network, clock, or OS filesystems
    let forbidden_names = [
        "tokio",
        "async-std",
        "reqwest",
        "hyper",
        "ureq",
        "rusqlite",
        "sqlx",
        "sqlite",
        "walkdir",
        "tempfile",
        "notify",
        "open",
        "std_semaphore",
        "chrono", // chrono has clock/system time calls; domain core uses raw strings or pure serde/uuidv7
    ];

    for forbidden in &forbidden_names {
        assert!(
            !normal_deps.contains(&forbidden.to_string()),
            "snapdown-core must not depend on forbidden I/O or OS crate: {forbidden}"
        );
    }

    // Allowed normal dependencies for snapdown-core
    let allowed_prefixes = ["serde", "thiserror", "uuid"];
    for dep in &normal_deps {
        let is_allowed = allowed_prefixes
            .iter()
            .any(|allowed| dep.starts_with(allowed));
        assert!(
            is_allowed,
            "snapdown-core has unexpected normal dependency `{dep}`. Only pure domain dependencies ({:?}) are allowed.",
            allowed_prefixes
        );
    }
}
