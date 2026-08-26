fn main() {
    println!("cargo:rerun-if-changed=ui/appwindow.slint");
    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=app.manifest");

    #[cfg(windows)]
    {
        let _ = embed_resource::compile("app.rc", embed_resource::NONE);
    }

    slint_build::compile("ui/appwindow.slint").unwrap();
}
