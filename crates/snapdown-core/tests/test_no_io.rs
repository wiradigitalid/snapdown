use cargo_metadata::{DependencyKind, MetadataCommand, PackageId};
use std::collections::HashSet;

#[test]
fn snapdown_core_has_no_io_dependency() {
    let target_triple = if cfg!(target_os = "windows") {
        "x86_64-pc-windows-msvc"
    } else if cfg!(target_os = "macos") {
        "x86_64-apple-darwin"
    } else {
        "x86_64-unknown-linux-gnu"
    };

    let metadata = MetadataCommand::new()
        .other_options(vec![
            "--filter-platform".to_string(),
            target_triple.to_string(),
        ])
        .exec()
        .expect("cargo metadata must execute successfully");

    let resolve = metadata
        .resolve
        .as_ref()
        .expect("cargo metadata resolve graph must exist");

    let core_pkg = metadata
        .packages
        .iter()
        .find(|p| p.name == "snapdown-core")
        .expect("snapdown-core package must exist in workspace metadata");

    // Walk the resolved graph transitively from snapdown-core
    let mut visited: HashSet<PackageId> = HashSet::new();
    let mut to_visit: Vec<PackageId> = vec![core_pkg.id.clone()];
    let mut transitive_packages: Vec<String> = Vec::new();

    while let Some(current_id) = to_visit.pop() {
        if !visited.insert(current_id.clone()) {
            continue;
        }

        let pkg = metadata
            .packages
            .iter()
            .find(|p| p.id == current_id)
            .expect("package must exist in metadata");

        if pkg.name != "snapdown-core" {
            transitive_packages.push(pkg.name.clone());
        }

        if let Some(node) = resolve.nodes.iter().find(|n| n.id == current_id) {
            for dep in &node.deps {
                let dep_pkg = metadata.packages.iter().find(|p| p.id == dep.pkg);
                let dep_name = dep_pkg.map(|p| p.name.as_str()).unwrap_or(&dep.name);

                let is_normal = pkg
                    .dependencies
                    .iter()
                    .any(|d| d.name == dep_name && d.kind == DependencyKind::Normal);

                let is_normal_dep_kind = dep
                    .dep_kinds
                    .iter()
                    .any(|k| k.kind == DependencyKind::Normal);

                if is_normal || is_normal_dep_kind {
                    to_visit.push(dep.pkg.clone());
                }
            }
        }
    }

    println!(
        "Transitive normal dependencies of snapdown-core: {:?}",
        transitive_packages
    );

    // Prohibited crates: any crate related to I/O, network, clock, or OS filesystems / entropy
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
        "chrono",
        "getrandom",
        "rand",
        "libc",
        "uuid",
    ];

    for forbidden in &forbidden_names {
        assert!(
            !transitive_packages.contains(&forbidden.to_string()),
            "snapdown-core must not have transitive dependency on forbidden crate: {forbidden}"
        );
    }

    // Explicit allowlist of allowed transitive crate names for snapdown-core
    let allowed_crates = [
        "serde",
        "serde_core",
        "serde_derive",
        "serde_json",
        "thiserror",
        "thiserror-impl",
        "proc-macro2",
        "quote",
        "syn",
        "unicode-ident",
        "itoa",
        "memchr",
        "zmij",
    ];

    for pkg_name in &transitive_packages {
        assert!(
            allowed_crates.contains(&pkg_name.as_str()),
            "snapdown-core has unexpected transitive dependency `{pkg_name}`. Only explicitly permitted crates are allowed.",
        );
    }
}
