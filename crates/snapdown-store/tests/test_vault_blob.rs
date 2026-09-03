use snapdown_core::ports::BlobStore;
use snapdown_store::vault::VaultBlobStore;
use tempfile::TempDir;

#[test]
fn vault_blob_write_read_delete_exists_lifecycle() {
    let tmp_dir = TempDir::new().unwrap();
    let vault = VaultBlobStore::new(tmp_dir.path()).unwrap();

    let rel_path = "images/finding_1.png";
    let data = b"dummy png content 12345";

    // Check not exists
    assert!(!vault.blob_exists(rel_path).unwrap());

    // Write blob
    vault.write_blob(rel_path, data).unwrap();
    assert!(vault.blob_exists(rel_path).unwrap());

    // Read blob
    let read_back = vault.read_blob(rel_path).unwrap();
    assert_eq!(read_back, data);

    // Delete blob
    vault.delete_blob(rel_path).unwrap();
    assert!(!vault.blob_exists(rel_path).unwrap());

    // Reading deleted blob returns not found error
    let read_after_delete = vault.read_blob(rel_path);
    assert!(read_after_delete.is_err());
}

#[test]
fn vault_refuses_a_path_that_escapes_its_root() {
    let tmp_dir = TempDir::new().unwrap();
    let vault = VaultBlobStore::new(tmp_dir.path()).unwrap();

    let traversal_paths = [
        "../outside.png",
        "../../outside.png",
        "images/../../outside.png",
        "/etc/passwd",
        "\\windows\\system32",
        "C:\\outside.txt",
    ];

    for path in &traversal_paths {
        let write_res = vault.write_blob(path, b"evil");
        assert!(
            write_res.is_err(),
            "Path traversal write must be refused for: {path}"
        );

        let read_res = vault.read_blob(path);
        assert!(
            read_res.is_err(),
            "Path traversal read must be refused for: {path}"
        );

        let del_res = vault.delete_blob(path);
        assert!(
            del_res.is_err(),
            "Path traversal delete must be refused for: {path}"
        );

        let del_folder_res = vault.delete_folder(path);
        assert!(
            del_folder_res.is_err(),
            "Path traversal folder-delete must be refused for: {path}"
        );
    }
}

/// `delete_folder` - added for ticket 16 of the Bundle Library spec (Disassemble / Delete). Removes
/// a whole folder and everything under it, which no method on this store could do before: every
/// other one is per-file.
#[test]
fn delete_folder_removes_a_whole_folder_and_everything_in_it() {
    let tmp_dir = TempDir::new().unwrap();
    let vault = VaultBlobStore::new(tmp_dir.path()).unwrap();

    vault
        .write_blob("bundles/b1/bundle.md", b"# A Bundle")
        .unwrap();
    vault
        .write_blob("bundles/b1/finding_1_burned.png", b"burned pixels")
        .unwrap();
    // A sibling folder must survive - this proves the delete is scoped to the one folder asked for,
    // not the whole `bundles/` tree.
    vault
        .write_blob("bundles/b2/bundle.md", b"# Another Bundle")
        .unwrap();

    vault.delete_folder("bundles/b1").unwrap();

    assert!(!tmp_dir.path().join("bundles/b1").exists());
    assert!(
        vault.blob_exists("bundles/b2/bundle.md").unwrap(),
        "a sibling folder must be untouched"
    );
}

/// A folder that is already gone, or a path that names a file rather than a folder, must refuse
/// rather than silently succeed or panic - the same "say what refused" discipline every other method
/// on this store already follows.
#[test]
fn delete_folder_refuses_a_missing_folder_or_a_plain_file() {
    let tmp_dir = TempDir::new().unwrap();
    let vault = VaultBlobStore::new(tmp_dir.path()).unwrap();

    assert!(
        vault.delete_folder("bundles/does-not-exist").is_err(),
        "a folder that was never created must refuse, not succeed on nothing"
    );

    vault
        .write_blob("bundles/b1/bundle.md", b"# A Bundle")
        .unwrap();
    assert!(
        vault.delete_folder("bundles/b1/bundle.md").is_err(),
        "a plain file is not a folder, and must be refused rather than removed by this method"
    );
    assert!(
        vault.blob_exists("bundles/b1/bundle.md").unwrap(),
        "the refused call must leave the file untouched"
    );
}
