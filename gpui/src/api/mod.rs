pub mod v1alpha1 {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/api/rockbox.v1alpha1.rs"
    ));
}
