fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = [
        "proto/rockbox/v1alpha1/browse.proto",
        "proto/rockbox/v1alpha1/genre.proto",
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
