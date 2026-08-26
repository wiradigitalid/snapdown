fn main() {
    println!("cargo:rerun-if-changed=ui/appwindow.slint");
    println!("cargo:rerun-if-changed=assets");
    slint_build::compile("ui/appwindow.slint").unwrap();
}
