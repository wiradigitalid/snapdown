use desktop_lib::vault_migration::VaultMigrator;
use std::fs;
use std::io;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn vault_move_reports_a_source_file_it_could_not_remove() {
    // SCN-01 / BR-29: When all files are copied and verified, a source file that
    // refuses deletion does NOT fail the move. Ok(report) is returned with the unremoved path.
    let src_tmp = TempDir::new().unwrap();
    let dest_tmp = TempDir::new().unwrap();

    let src_path = src_tmp.path();
    let dest_path = dest_tmp.path();

    let src_f1 = src_path.join("finding_1.png");
    let src_f2 = src_path.join("finding_2.png");
    let src_f3 = src_path.join("finding_3.png");

    fs::write(&src_f1, b"FINDING_1_BYTES").unwrap();
    fs::write(&src_f2, b"FINDING_2_BYTES").unwrap();
    fs::write(&src_f3, b"FINDING_3_BYTES").unwrap();

    // Canonical source paths because migrate_vault canonicalizes
    let canonical_src_f2 = src_f2.canonicalize().unwrap();

    // Delete seam: refuses to delete finding_2.png
    let custom_deleter = |p: &Path| -> io::Result<()> {
        if p == canonical_src_f2 || p == src_f2 {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Simulated lock/permission denied",
            ))
        } else {
            fs::remove_file(p)
        }
    };

    let result = VaultMigrator::migrate_vault_with_deleter(src_path, dest_path, custom_deleter);

    assert!(
        result.is_ok(),
        "Migration must succeed (Ok) when all files copied and verified, even if source deletion fails"
    );

    let report = result.unwrap();
    assert_eq!(
        report.unremoved_sources.len(),
        1,
        "Report must contain exactly 1 unremoved source file"
    );
    assert_eq!(
        report.unremoved_sources[0], canonical_src_f2,
        "Unremoved source file in report must match finding_2.png"
    );

    // Destination must contain all 3 copied and verified files
    assert_eq!(
        fs::read(dest_path.join("finding_1.png")).unwrap(),
        b"FINDING_1_BYTES"
    );
    assert_eq!(
        fs::read(dest_path.join("finding_2.png")).unwrap(),
        b"FINDING_2_BYTES"
    );
    assert_eq!(
        fs::read(dest_path.join("finding_3.png")).unwrap(),
        b"FINDING_3_BYTES"
    );

    // Source finding_2 must still exist, while finding_1 and finding_3 were deleted
    assert!(!src_f1.exists(), "finding_1 must be removed from source");
    assert!(src_f2.exists(), "finding_2 must remain in source");
    assert!(!src_f3.exists(), "finding_3 must be removed from source");
}

#[test]
fn vault_move_reports_a_destination_copy_it_could_not_clean_up() {
    // When migration fails and rollback occurs, destination copies that cannot be deleted
    // are surfaced in VaultMigrationError.uncleaned_destinations.
    let src_tmp = TempDir::new().unwrap();
    let dest_tmp = TempDir::new().unwrap();

    let src_path = src_tmp.path();
    let dest_path = dest_tmp.path();

    let src_f1 = src_path.join("finding_1.png");
    let src_f2 = src_path.join("finding_2.png");
    let src_f3 = src_path.join("finding_3.png");

    fs::write(&src_f1, b"FINDING_1_BYTES").unwrap();
    fs::write(&src_f2, b"FINDING_2_BYTES").unwrap();
    fs::write(&src_f3, b"FINDING_3_BYTES").unwrap();

    // Create a directory at dest_path/finding_2.png to cause copy error at file 2
    fs::create_dir(dest_path.join("finding_2.png")).unwrap();

    let canonical_dest = dest_path.canonicalize().unwrap();
    let dest_f1 = canonical_dest.join("finding_1.png");

    // Delete seam: fails to delete destination finding_1.png during rollback
    let custom_deleter = |p: &Path| -> io::Result<()> {
        if p == dest_f1 || p == dest_path.join("finding_1.png") {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Simulated lock on destination file during rollback",
            ))
        } else {
            fs::remove_file(p)
        }
    };

    let result = VaultMigrator::migrate_vault_with_deleter(src_path, dest_path, custom_deleter);

    assert!(result.is_err(), "Migration must fail and return Err");
    let err = result.unwrap_err();

    assert!(
        err.uncleaned_destinations.contains(&dest_f1),
        "Error must report uncleaned destination finding_1.png, got: {:?}",
        err.uncleaned_destinations
    );
    assert!(
        err.to_string()
            .contains("could not clean up destination files"),
        "Formatted error must describe uncleaned destination files"
    );

    // All source files must remain intact
    assert_eq!(fs::read(&src_f1).unwrap(), b"FINDING_1_BYTES");
    assert_eq!(fs::read(&src_f2).unwrap(), b"FINDING_2_BYTES");
    assert_eq!(fs::read(&src_f3).unwrap(), b"FINDING_3_BYTES");
}

#[test]
fn vault_move_failing_at_file_n_leaves_every_source_file_in_place() {
    // BR-29 / AD-2: If migration fails midway at file n, all source files are left completely untouched.
    let src_tmp = TempDir::new().unwrap();
    let dest_tmp = TempDir::new().unwrap();

    let src_path = src_tmp.path();
    let dest_path = dest_tmp.path();

    let src_f1 = src_path.join("finding_1.png");
    let src_f2 = src_path.join("finding_2.png");
    let src_f3 = src_path.join("finding_3.png");

    fs::write(&src_f1, b"FINDING_1_BYTES").unwrap();
    fs::write(&src_f2, b"FINDING_2_BYTES").unwrap();
    fs::write(&src_f3, b"FINDING_3_BYTES").unwrap();

    // Trigger copy failure at file 2
    fs::create_dir(dest_path.join("finding_2.png")).unwrap();

    let result = VaultMigrator::migrate_vault(src_path, dest_path);
    assert!(result.is_err(), "Migration must fail at file 2");

    // Every source file must still exist and have identical contents
    assert_eq!(fs::read(&src_f1).unwrap(), b"FINDING_1_BYTES");
    assert_eq!(fs::read(&src_f2).unwrap(), b"FINDING_2_BYTES");
    assert_eq!(fs::read(&src_f3).unwrap(), b"FINDING_3_BYTES");

    // Successfully copied destination file 1 must have been rolled back and deleted
    assert!(
        !dest_path.join("finding_1.png").exists(),
        "Rolled-back destination file must be deleted"
    );
}
