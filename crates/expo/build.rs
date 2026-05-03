fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate client-only bindings from the shared proto files. The local
    // `proto` directory is a symlink to `../rpc/proto` so we read through the
    // same path the rest of the workspace uses (`crates/rpc/proto`) without
    // duplicating files. We deliberately avoid depending on `rockbox-rpc` to
    // keep the mobile crate slim (no sqlx / typesense / library transitive
    // deps that fight cross-compilation).
    let protos = [
        "proto/rockbox/v1alpha1/browse.proto",
        "proto/rockbox/v1alpha1/library.proto",
        "proto/rockbox/v1alpha1/metadata.proto",
        "proto/rockbox/v1alpha1/playback.proto",
        "proto/rockbox/v1alpha1/playlist.proto",
        "proto/rockbox/v1alpha1/settings.proto",
        "proto/rockbox/v1alpha1/sound.proto",
        "proto/rockbox/v1alpha1/system.proto",
        "proto/rockbox/v1alpha1/saved_playlist.proto",
        "proto/rockbox/v1alpha1/smart_playlist.proto",
        "proto/rockbox/v1alpha1/bluetooth.proto",
    ];

    // Derive serde::Serialize on every generated proto type so we can ship
    // entire responses to JS as JSON in one line. (Deserialize is not derived
    // — we only serialize *out* of Rust today.)
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .type_attribute(".", "#[derive(serde::Serialize)]")
        .compile_protos(&protos, &["proto"])?;

    for p in &protos {
        println!("cargo:rerun-if-changed={p}");
    }

    Ok(())
}
