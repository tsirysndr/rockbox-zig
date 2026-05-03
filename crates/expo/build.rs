use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ── proto codegen (unchanged) ──────────────────────────────────────────
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
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .type_attribute(".", "#[derive(serde::Serialize)]")
        .compile_protos(&protos, &["proto"])?;
    for p in &protos {
        println!("cargo:rerun-if-changed={p}");
    }

    // ── embedded-daemon: link C firmware archives ──────────────────────────
    if env::var_os("CARGO_FEATURE_EMBEDDED_DAEMON").is_some() {
        link_firmware()?;
    }

    Ok(())
}

fn link_firmware() -> Result<(), Box<dyn std::error::Error>> {
    let target_os = env::var("CARGO_CFG_TARGET_OS")?;
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH")?;

    // Per-ABI build dir produced by `tools/configure --target=205`.
    // Override via ROCKBOX_FIRMWARE_DIR for non-standard layouts.
    let abi_dir = match (target_os.as_str(), target_arch.as_str()) {
        ("android", "aarch64") => "build-android-arm64",
        ("android", "arm") => "build-android-armv7",
        ("android", "x86_64") => "build-android-x86_64",
        // Desktop dev builds with embedded-daemon enabled (rare — mostly
        // for typecheck/proof-of-life) reuse the existing sdlapp build.
        _ => "build-lib",
    };

    let workspace_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .ancestors()
        .nth(2)
        .ok_or("could not find workspace root")?
        .to_path_buf();

    let firmware_root = env::var("ROCKBOX_FIRMWARE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_root.join(abi_dir));

    if !firmware_root.exists() {
        panic!(
            "embedded-daemon enabled but firmware build dir not found: {}\n\
             Run `tools/configure --target=205` and `make` in that dir, OR set\n\
             ROCKBOX_FIRMWARE_DIR to a directory that contains libfirmware.a etc.",
            firmware_root.display()
        );
    }

    println!("cargo:rerun-if-env-changed=ROCKBOX_FIRMWARE_DIR");
    println!(
        "cargo:rerun-if-changed={}/static-libs.stamp",
        firmware_root.display()
    );

    // Search paths for the linker.
    println!("cargo:rustc-link-search=native={}", firmware_root.display());
    println!(
        "cargo:rustc-link-search=native={}/firmware",
        firmware_root.display()
    );
    println!(
        "cargo:rustc-link-search=native={}/lib",
        firmware_root.display()
    );
    println!(
        "cargo:rustc-link-search=native={}/lib/rbcodec/codecs",
        firmware_root.display()
    );

    // Order matters: librockbox depends on libfirmware which depends on
    // librbcodec which depends on the codec helper libs + per-codec .a's
    // (per-codec entries hold the renamed `__header_<name>` symbols).
    // GNU `ld` needs producers before consumers; LLVM's lld is more lenient
    // but the order below is safe on both.
    for lib in &[
        "rockbox",
        "firmware",
        "rbcodec",
        "skin_parser",
        "fixedpoint",
        "tlsf",
    ] {
        println!("cargo:rustc-link-lib=static={lib}");
    }

    // librbnetstream sometimes lives at workspace-root level rather than
    // under lib/. Both are valid via the link-search paths above.
    if firmware_root.join("librbnetstream.a").exists() {
        println!("cargo:rustc-link-lib=static=rbnetstream");
    }

    // Glob every codec / helper .a — order doesn't matter inside this
    // group, the linker resolves them at section level.
    let codec_dir = firmware_root.join("lib/rbcodec/codecs");
    if codec_dir.exists() {
        for entry in std::fs::read_dir(&codec_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|s| s.to_str()) != Some("a") {
                continue;
            }
            let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
            // Codec helper libs are named lib*.a (libfaad.a, libffmpegFLAC.a)
            // and resolve normally via `static=NAME`. Codec entry-point
            // archives (flac.a, opus.a, ape.a, ...) lack the "lib" prefix
            // because Rockbox's lc_open() looks them up by their codec name.
            // Use Rust's `+verbatim` modifier (stable since 1.67) to pass
            // those filenames literally, bypassing cargo's libNAME.a mangling.
            if let Some(rest) = stem.strip_prefix("lib") {
                println!("cargo:rustc-link-lib=static={rest}");
            } else {
                println!("cargo:rustc-link-lib=static:+verbatim={stem}.a");
            }
        }
    }

    // Android system libs needed by our cdylib files (pcm-aaudio.c uses
    // AAudio, system-android.c uses __android_log_print).
    if target_os == "android" {
        println!("cargo:rustc-link-lib=dylib=log");
        println!("cargo:rustc-link-lib=dylib=android");
        println!("cargo:rustc-link-lib=dylib=aaudio");
    }

    Ok(())
}
