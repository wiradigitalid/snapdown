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
    }
}
