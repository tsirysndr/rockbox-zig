const std = @import("std");

// Although this function looks imperative, note that its job is to
// declaratively construct a build graph that will be executed by an external
// runner.
pub fn build(b: *std.Build) void {
    // Standard target options allows the person running `zig build` to choose
    // what target to build for. Here we do not override the defaults, which
    // means any target is allowed, and the default is native. Other options
    // for restricting supported target set are available.
    const target = b.standardTargetOptions(.{});
    //const target = b.resolveTargetQuery(.{
    //    .cpu_arch = .x86_64,
    //   .os_tag = .linux,
    //   .abi = .gnu,
    //});

    // Standard optimization options allow the person running `zig build` to select
    // between Debug, ReleaseSafe, ReleaseFast, and ReleaseSmall. Here we do not
    // set a preferred release mode, allowing the user to decide how to optimize.
    const optimize = b.standardOptimizeOption(.{});

    const lib = b.addStaticLibrary(.{
        .name = "rockbox",
        // In this case the main source file is merely a path, however, in more
        // complicated build scripts, this could be a generated file.
        .root_source_file = b.path("src/root.zig"),
        .target = target,
        .optimize = optimize,
    });

    // This declares intent for the library to be installed into the standard
    // location when the user invokes the "install" step (the default step when
    // running `zig build`).
    b.installArtifact(lib);

    const exe = b.addExecutable(.{
        .name = "rockbox",
        // .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    // This declares intent for the executable to be installed into the
    // standard location when the user invokes the "install" step (the default
    // step when running `zig build`).
    b.installArtifact(exe);

    // This *creates* a Run step in the build graph, to be executed when another
    // step is evaluated that depends on it. The next line below will establish
    // such a dependency.
    const run_cmd = b.addRunArtifact(exe);

    // By making the run step depend on the install step, it will be run from the
    // installation directory rather than directly from within the cache directory.
    // This is not necessary, however, if the application depends on other installed
    // files, this ensures they will be present and in the expected location.
    run_cmd.step.dependOn(b.getInstallStep());

    // This allows the user to pass arguments to the application in the build
    // command itself, like this: `zig build run -- arg1 arg2 etc`
    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    // This creates a build step. It will be visible in the `zig build --help` menu,
    // and can be selected like this: `zig build run`
    // This will evaluate the `run` step rather than the default, which is "install".
    const run_step = b.step("run", "Run the app");
    run_step.dependOn(&run_cmd.step);

    // Creates a step for unit testing. This only builds the test executable
    // but does not run it.
    const lib_unit_tests = b.addTest(.{
        .root_source_file = b.path("src/root.zig"),
        .target = target,
        .optimize = optimize,
    });

    const run_lib_unit_tests = b.addRunArtifact(lib_unit_tests);

    const exe_unit_tests = b.addTest(.{
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    const run_exe_unit_tests = b.addRunArtifact(exe_unit_tests);

    // Similar to creating the run step earlier, this exposes a `test` step to
    // the `zig build --help` menu, providing a way for the user to request
    // running the unit tests.
    const test_step = b.step("test", "Run unit tests");
    test_step.dependOn(&run_lib_unit_tests.step);
    test_step.dependOn(&run_exe_unit_tests.step);

    exe.addCSourceFiles(.{
        // .files = &[_][]const u8{},
        .files = &all_sources,
        .flags = &cflags,
    });

    lib.addCSourceFiles(.{
        //.files = &all_sources,
        .files = &[_][]const u8{},
        .flags = &cflags,
    });

    const libfirmware = b.addStaticLibrary(.{
        .name = "firmware",
        .target = target,
        .optimize = optimize,
    });
    b.installArtifact(libfirmware);

    libfirmware.addCSourceFiles(.{
        .files = &libfirmware_sources,
        .flags = &cflags,
    });

    defineCMacros(libfirmware);
    addIncludePaths(libfirmware);

    const libspeex_voice = b.addStaticLibrary(.{
        .name = "speex-voice",
        .target = target,
        .optimize = optimize,
    });
    b.installArtifact(libspeex_voice);

    libspeex_voice.addCSourceFiles(.{
        .files = &libspeex_voice_sources,
        .flags = &cflags,
    });

    libspeex_voice.defineCMacro("HAVE_CONFIG_H", null);
    libspeex_voice.defineCMacro("ROCKBOX_VOICE_CODEC", null);
    libspeex_voice.defineCMacro("SPEEX_DISABLE_ENCODER", null);
    defineCMacros(libspeex_voice);
    addIncludePaths(libspeex_voice);

    const librbcodec = b.addStaticLibrary(.{
        .name = "rbcodec",
        .target = target,
        .optimize = optimize,
    });
    b.installArtifact(librbcodec);

    librbcodec.addCSourceFiles(.{
        .files = &librbcodec_sources,
        .flags = &cflags,
    });

    defineCMacros(librbcodec);
    addIncludePaths(librbcodec);

    const libskinparser = b.addStaticLibrary(.{
        .name = "skinparser",
        .target = target,
        .optimize = optimize,
    });
    b.installArtifact(libskinparser);

    libskinparser.addCSourceFiles(.{
        .files = &libskinparser_sources,
        .flags = &cflags,
    });

    defineCMacros(libskinparser);
    addIncludePaths(libskinparser);

    const libfixedpoint = b.addStaticLibrary(.{
        .name = "fixedpoint",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libfixedpoint);

    libfixedpoint.addCSourceFiles(.{
        .files = &libfixedpoint_sources,
        .flags = &cflags,
    });

    defineCMacros(libfixedpoint);
    addIncludePaths(libfixedpoint);

    const libuisimulator = b.addStaticLibrary(.{
        .name = "uisimulator",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libuisimulator);

    libuisimulator.addCSourceFiles(.{
        .files = &libuisimulator_sources,
        .flags = &cflags,
    });

    libuisimulator.defineCMacro("HAVE_CONFIG_H", null);
    defineCMacros(libuisimulator);
    addIncludePaths(libuisimulator);

    const libcodec = b.addStaticLibrary(.{
        .name = "codec",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libcodec);

    libcodec.addCSourceFiles(.{
        .files = &libcodec_sources,
        .flags = &cflags,
    });

    defineCMacros(libcodec);
    addIncludePaths(libcodec);

    const libtlsf = b.addStaticLibrary(.{
        .name = "tlsf",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libtlsf);

    libtlsf.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/tlsf/src/tlsf.c",
        },
        .flags = &cflags,
    });

    defineCMacros(libtlsf);
    addIncludePaths(libtlsf);

    // const libopus = b.addStaticLibrary(.{
    //   .name = "opus",
    //    .target = target,
    //    .optimize = optimize,
    // });

    // b.installArtifact(libopus);

    // libopus.addCSourceFiles(.{
    //      .files = &libopus_sources,
    //      .flags = &cflags,
    //  });

    // libopus.defineCMacro("CODEC", null);
    // libopus.defineCMacro("HAVE_CONFIG_H", null);
    // libopus.defineCMacro("ENABLE_ASSERTIONS", null);
    // libopus.defineCMacro("VAR_ARRAYS", null);
    // libopus.defineCMacro("USE_ALLOCA", null);
    // defineCMacros(libopus);
    // addIncludePaths(libopus);

    const libmad = b.addStaticLibrary(.{
        .name = "mad",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libmad);

    libmad.addCSourceFiles(.{
        .files = &libmad_sources,
        .flags = &cflags,
    });

    libmad.defineCMacro("CODEC", null);
    defineCMacros(libmad);
    addIncludePaths(libmad);

    const libasf = b.addStaticLibrary(.{
        .name = "asf",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libasf);

    libasf.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/libasf/asf.c",
        },
        .flags = &cflags,
    });

    libasf.defineCMacro("CODEC", null);
    libasf.defineCMacro("HAVE_CONFIG_H", null);
    defineCMacros(libasf);
    addIncludePaths(libasf);

    const flac = b.addStaticLibrary(.{
        .name = "flac",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(flac);

    flac.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/flac.c",
            "lib/rbcodec/codecs/libffmpegFLAC/decoder.c",
            "lib/rbcodec/codecs/libffmpegFLAC/shndec.c",
        },
        .flags = &cflags,
    });

    flac.defineCMacro("CODEC", null);
    defineCMacros(flac);
    addIncludePaths(flac);

    const wav = b.addStaticLibrary(.{
        .name = "wav",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(wav);

    wav.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/wav.c",
            "lib/rbcodec/codecs/libpcm/linear_pcm.c",
            "lib/rbcodec/codecs/libpcm/itut_g711.c",
            "lib/rbcodec/codecs/libpcm/dvi_adpcm.c",
            "lib/rbcodec/codecs/libpcm/ieee_float.c",
            "lib/rbcodec/codecs/libpcm/adpcm_seek.c",
            "lib/rbcodec/codecs/libpcm/dialogic_oki_adpcm.c",
            "lib/rbcodec/codecs/libpcm/ms_adpcm.c",
            "lib/rbcodec/codecs/libpcm/yamaha_adpcm.c",
            "lib/rbcodec/codecs/libpcm/ima_adpcm_common.c",
            "lib/rbcodec/codecs/libpcm/qt_ima_adpcm.c",
            "lib/rbcodec/codecs/libpcm/swf_adpcm.c",
        },
        .flags = &cflags,
    });

    wav.defineCMacro("CODEC", null);
    defineCMacros(wav);
    addIncludePaths(wav);

    const liba52 = b.addStaticLibrary(.{
        .name = "a52",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(liba52);

    liba52.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/a52.c",
            "lib/rbcodec/codecs/liba52/bit_allocate.c",
            "lib/rbcodec/codecs/liba52/bitstream.c",
            "lib/rbcodec/codecs/liba52/downmix.c",
            "lib/rbcodec/codecs/liba52/imdct.c",
            "lib/rbcodec/codecs/liba52/parse.c",
        },
        .flags = &cflags,
    });

    liba52.defineCMacro("CODEC", null);
    defineCMacros(liba52);
    addIncludePaths(liba52);

    const libwavpack = b.addStaticLibrary(.{
        .name = "wavpack",
        .target = target,
        .optimize = optimize,
    });

    // b.installArtifact(libwavpack);

    libwavpack.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/wavpack.c",
            "lib/rbcodec/codecs/libwavpack/bits.c",
            "lib/rbcodec/codecs/libwavpack/float.c",
            "lib/rbcodec/codecs/libwavpack/metadata.c",
            "lib/rbcodec/codecs/libwavpack/unpack.c",
            "lib/rbcodec/codecs/libwavpack/pack.c",
            "lib/rbcodec/codecs/libwavpack/words.c",
            "lib/rbcodec/codecs/libwavpack/wputils.c",
        },
        .flags = &cflags,
    });

    libwavpack.defineCMacro("CODEC", null);
    defineCMacros(libwavpack);
    addIncludePaths(libwavpack);

    const alac = b.addStaticLibrary(.{
        .name = "alac",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(alac);

    alac.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/alac.c",
            "lib/rbcodec/codecs/libalac/alac.c",
        },
        .flags = &cflags,
    });

    alac.defineCMacro("CODEC", null);
    defineCMacros(alac);
    addIncludePaths(alac);

    const libm4a = b.addStaticLibrary(.{
        .name = "m4a",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libm4a);

    libm4a.addCSourceFiles(.{ .files = &[_][]const u8{
        "lib/rbcodec/codecs/libm4a/m4a.c",
        "lib/rbcodec/codecs/libm4a/demux.c",
    }, .flags = &cflags });

    libm4a.defineCMacro("CODEC", null);
    defineCMacros(libm4a);
    addIncludePaths(libm4a);

    const libcook = b.addStaticLibrary(.{
        .name = "cook",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libcook);

    libcook.addCSourceFiles(.{ .files = &[_][]const u8{
        "./lib/rbcodec/codecs/cook.c",
        "./lib/rbcodec/codecs/libcook/cook.c",
    }, .flags = &cflags });

    libcook.defineCMacro("CODEC", null);
    defineCMacros(libcook);
    addIncludePaths(libcook);

    const libfaad = b.addStaticLibrary(.{
        .name = "faad",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libfaad);

    libfaad.addCSourceFiles(.{ .files = &[_][]const u8{
        "lib/rbcodec/codecs/raac.c",
        "lib/rbcodec/codecs/libfaad/bits.c",
        "lib/rbcodec/codecs/libfaad/common.c",
        "lib/rbcodec/codecs/libfaad/decoder.c",
        "lib/rbcodec/codecs/libfaad/drc.c",
        "lib/rbcodec/codecs/libfaad/error.c",
        "lib/rbcodec/codecs/libfaad/filtbank.c",
        "lib/rbcodec/codecs/libfaad/huffman.c",
        "lib/rbcodec/codecs/libfaad/is.c",
        "lib/rbcodec/codecs/libfaad/mp4.c",
        "lib/rbcodec/codecs/libfaad/ms.c",
        "lib/rbcodec/codecs/libfaad/pns.c",
        "lib/rbcodec/codecs/libfaad/ps_dec.c",
        "lib/rbcodec/codecs/libfaad/ps_syntax.c",
        "lib/rbcodec/codecs/libfaad/pulse.c",
        "lib/rbcodec/codecs/libfaad/sbr_dct.c",
        "lib/rbcodec/codecs/libfaad/sbr_dec.c",
        "lib/rbcodec/codecs/libfaad/sbr_e_nf.c",
        "lib/rbcodec/codecs/libfaad/sbr_fbt.c",
        "lib/rbcodec/codecs/libfaad/sbr_hfadj.c",
        "lib/rbcodec/codecs/libfaad/sbr_hfgen.c",
        "lib/rbcodec/codecs/libfaad/sbr_huff.c",
        "lib/rbcodec/codecs/libfaad/sbr_qmf.c",
        "lib/rbcodec/codecs/libfaad/sbr_syntax.c",
        "lib/rbcodec/codecs/libfaad/sbr_tf_grid.c",
        "lib/rbcodec/codecs/libfaad/specrec.c",
        "lib/rbcodec/codecs/libfaad/syntax.c",
        "lib/rbcodec/codecs/libfaad/tns.c",
    }, .flags = &cflags });

    libfaad.defineCMacro("CODEC", null);
    defineCMacros(libfaad);
    addIncludePaths(libfaad);

    const a52_rm = b.addStaticLibrary(.{
        .name = "a52_rm",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(a52_rm);

    a52_rm.addCSourceFiles(.{ .files = &[_][]const u8{
        "lib/rbcodec/codecs/a52_rm.c",
    }, .flags = &cflags });

    a52_rm.defineCMacro("CODEC", null);
    defineCMacros(a52_rm);
    addIncludePaths(a52_rm);

    const libatrac = b.addStaticLibrary(.{
        .name = "atrac",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libatrac);

    libatrac.addCSourceFiles(.{ .files = &[_][]const u8{
        "lib/rbcodec/codecs/atrac3_rm.c",
        "lib/rbcodec/codecs/libatrac/atrac3.c",
    }, .flags = &cflags });

    libatrac.defineCMacro("CODEC", null);
    defineCMacros(libatrac);
    addIncludePaths(libatrac);

    const atrac3_oma = b.addStaticLibrary(.{
        .name = "atrac3_oma",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(atrac3_oma);

    atrac3_oma.addCSourceFiles(.{ .files = &[_][]const u8{
        "lib/rbcodec/codecs/atrac3_oma.c",
    }, .flags = &cflags });

    atrac3_oma.defineCMacro("CODEC", null);
    defineCMacros(atrac3_oma);
    addIncludePaths(atrac3_oma);

    const musepack = b.addStaticLibrary(.{
        .name = "musepack",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(musepack);

    musepack.addCSourceFiles(.{ .files = &[_][]const u8{
        "lib/rbcodec/codecs/mpc.c",
        "lib/rbcodec/codecs/libmusepack/crc32.c",
        "lib/rbcodec/codecs/libmusepack/huffman.c",
        "lib/rbcodec/codecs/libmusepack/mpc_bits_reader.c",
        "lib/rbcodec/codecs/libmusepack/mpc_decoder.c",
        "lib/rbcodec/codecs/libmusepack/mpc_demux.c",
        "lib/rbcodec/codecs/libmusepack/requant.c",
        "lib/rbcodec/codecs/libmusepack/streaminfo.c",
        "lib/rbcodec/codecs/libmusepack/synth_filter.c",
    }, .flags = &cflags });

    musepack.defineCMacro("CODEC", null);
    defineCMacros(musepack);
    addIncludePaths(musepack);

    const wma = b.addStaticLibrary(.{
        .name = "wma",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(wma);

    wma.addCSourceFiles(.{ .files = &[_][]const u8{
        "lib/rbcodec/codecs/wma.c",
        "lib/rbcodec/codecs/libwma/wmadeci.c",
        "lib/rbcodec/codecs/libwma/wmafixed.c",
    }, .flags = &cflags });

    wma.defineCMacro("CODEC", null);
    defineCMacros(wma);
    addIncludePaths(wma);

    const libdemac = b.addStaticLibrary(.{
        .name = "demac",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libdemac);

    libdemac.addCSourceFiles(.{ .files = &[_][]const u8{
        "lib/rbcodec/codecs/ape.c",
        "lib/rbcodec/codecs/demac/libdemac/predictor.c",
        "lib/rbcodec/codecs/demac/libdemac/entropy.c",
        "lib/rbcodec/codecs/demac/libdemac/decoder.c",
        "lib/rbcodec/codecs/demac/libdemac/parser.c",
        "lib/rbcodec/codecs/demac/libdemac/filter_1280_15.c",
        "lib/rbcodec/codecs/demac/libdemac/filter_16_11.c",
        "lib/rbcodec/codecs/demac/libdemac/filter_256_13.c",
        "lib/rbcodec/codecs/demac/libdemac/filter_32_10.c",
        "lib/rbcodec/codecs/demac/libdemac/filter_64_11.c",
    }, .flags = &cflags });

    libdemac.defineCMacro("CODEC", null);
    defineCMacros(libdemac);
    addIncludePaths(libdemac);

    const libasap = b.addStaticLibrary(.{
        .name = "asap",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libasap);

    libasap.addCSourceFiles(.{ .files = &[_][]const u8{
        "lib/rbcodec/codecs/asap.c",
        "lib/rbcodec/codecs/libasap/acpu.c",
        "lib/rbcodec/codecs/libasap/asap.c",
        "lib/rbcodec/codecs/libasap/apokeysnd.c",
    }, .flags = &cflags });

    libasap.defineCMacro("CODEC", null);
    defineCMacros(libasap);
    addIncludePaths(libasap);

    const aac = b.addStaticLibrary(.{
        .name = "aac",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(aac);

    aac.addCSourceFiles(.{
        .files = &[_][]const u8{"lib/rbcodec/codecs/aac.c"},
        .flags = &cflags,
    });

    aac.defineCMacro("CODEC", null);
    defineCMacros(aac);
    addIncludePaths(aac);

    const libspc = b.addStaticLibrary(.{
        .name = "spc",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libspc);

    libspc.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/spc.c",
            "lib/rbcodec/codecs/libspc/spc_cpu.c",
            "lib/rbcodec/codecs/libspc/spc_dsp.c",
            "lib/rbcodec/codecs/libspc/spc_emu.c",
            "lib/rbcodec/codecs/libspc/spc_profiler.c",
        },
        .flags = &cflags,
    });

    libspc.defineCMacro("CODEC", null);
    defineCMacros(libspc);
    addIncludePaths(libspc);

    const mod = b.addStaticLibrary(.{
        .name = "mod",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(mod);

    mod.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/mod.c",
        },
        .flags = &cflags,
    });

    mod.defineCMacro("CODEC", null);
    defineCMacros(mod);
    addIncludePaths(mod);

    const shorten = b.addStaticLibrary(.{
        .name = "shorten",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(shorten);

    shorten.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/shorten.c",
        },
        .flags = &cflags,
    });

    shorten.defineCMacro("CODEC", null);
    defineCMacros(shorten);
    addIncludePaths(shorten);

    const aiff = b.addStaticLibrary(.{
        .name = "aiff",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(aiff);

    aiff.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/aiff.c",
        },
        .flags = &cflags,
    });

    aiff.defineCMacro("CODEC", null);
    defineCMacros(aiff);
    addIncludePaths(aiff);

    const libspeex = b.addStaticLibrary(.{
        .name = "speex",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libspeex);

    libspeex.addCSourceFiles(.{
        .files = &libspeex_sources,
        .flags = &cflags,
    });

    libspeex.defineCMacro("CODEC", null);
    libspeex.defineCMacro("HAVE_CONFIG_H", null);
    libspeex.defineCMacro("SPEEX_DISABLE_ENCODER", null);

    defineCMacros(libspeex);
    addIncludePaths(libspeex);

    const adx = b.addStaticLibrary(.{
        .name = "adx",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(adx);

    adx.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/adx.c",
        },
        .flags = &cflags,
    });

    adx.defineCMacro("CODEC", null);
    defineCMacros(adx);
    addIncludePaths(adx);

    const smaf = b.addStaticLibrary(.{
        .name = "smaf",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(smaf);

    smaf.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/smaf.c",
        },
        .flags = &cflags,
    });

    smaf.defineCMacro("CODEC", null);
    defineCMacros(smaf);
    addIncludePaths(smaf);

    const au = b.addStaticLibrary(.{
        .name = "au",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(au);

    au.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/au.c",
        },
        .flags = &cflags,
    });

    au.defineCMacro("CODEC", null);
    defineCMacros(au);
    addIncludePaths(au);

    const vox = b.addStaticLibrary(.{
        .name = "vox",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(vox);

    vox.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/vox.c",
        },
        .flags = &cflags,
    });

    vox.defineCMacro("CODEC", null);
    defineCMacros(vox);
    addIncludePaths(vox);

    const wav64 = b.addStaticLibrary(.{
        .name = "wav64",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(wav64);

    wav64.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/wav64.c",
        },
        .flags = &cflags,
    });

    wav64.defineCMacro("CODEC", null);
    defineCMacros(wav64);
    addIncludePaths(wav64);

    const tta = b.addStaticLibrary(.{
        .name = "tta",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(tta);

    tta.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/tta.c",
            "lib/rbcodec/codecs/libtta/ttadec.c",
        },
        .flags = &cflags,
    });

    tta.defineCMacro("CODEC", null);
    defineCMacros(tta);
    addIncludePaths(tta);

    const wmapro = b.addStaticLibrary(.{
        .name = "wmapro",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(wmapro);

    wmapro.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/wmapro.c",
            "lib/rbcodec/codecs/libwmapro/wmaprodec.c",
            "lib/rbcodec/codecs/libwmapro/wma.c",
            "lib/rbcodec/codecs/libwmapro/mdct_tables.c",
        },
        .flags = &cflags,
    });

    wmapro.defineCMacro("CODEC", null);
    defineCMacros(wmapro);
    addIncludePaths(wmapro);

    const libay = b.addStaticLibrary(.{
        .name = "ay",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libay);

    libay.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/ay.c",
            "lib/rbcodec/codecs/libgme/ay_apu.c",
            "lib/rbcodec/codecs/libgme/ay_cpu.c",
            "lib/rbcodec/codecs/libgme/ay_emu.c",
            "lib/rbcodec/codecs/libgme/blip_buffer.c",
            "lib/rbcodec/codecs/libgme/multi_buffer.c",
            "lib/rbcodec/codecs/libgme/track_filter.c",
            "lib/rbcodec/codecs/libgme/z80_cpu.c",
        },
        .flags = &cflags,
    });

    libay.defineCMacro("CODEC", null);
    defineCMacros(libay);
    addIncludePaths(libay);

    const libayumi = b.addStaticLibrary(.{
        .name = "ayumi",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libayumi);

    libayumi.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/vtx.c",
            "lib/rbcodec/codecs/libayumi/ayumi_render.c",
            "lib/rbcodec/codecs/libayumi/ayumi.c",
            "lib/rbcodec/codecs/libayumi/lzh.c",
        },
        .flags = &cflags,
    });

    libayumi.defineCMacro("CODEC", null);
    defineCMacros(libayumi);
    addIncludePaths(libayumi);

    const libgbs = b.addStaticLibrary(.{
        .name = "gbs",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libgbs);

    libgbs.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/gbs.c",
            "lib/rbcodec/codecs/libgme/gb_apu.c",
            "lib/rbcodec/codecs/libgme/gb_cpu.c",
            "lib/rbcodec/codecs/libgme/gbs_cpu.c",
            "lib/rbcodec/codecs/libgme/gb_oscs.c",
            "lib/rbcodec/codecs/libgme/gbs_emu.c",
            "lib/rbcodec/codecs/libgme/rom_data.c",
        },
        .flags = &cflags,
    });

    libgbs.defineCMacro("CODEC", null);
    defineCMacros(libgbs);
    addIncludePaths(libgbs);

    const libhes = b.addStaticLibrary(.{
        .name = "hes",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libhes);

    libhes.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/hes.c",
            "lib/rbcodec/codecs/libgme/hes_apu.c",
            "lib/rbcodec/codecs/libgme/hes_apu_adpcm.c",
            "lib/rbcodec/codecs/libgme/hes_cpu.c",
            "lib/rbcodec/codecs/libgme/hes_emu.c",
        },
        .flags = &cflags,
    });

    libhes.defineCMacro("CODEC", null);
    defineCMacros(libhes);
    addIncludePaths(libhes);

    const libnsf = b.addStaticLibrary(.{
        .name = "nsf",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libnsf);

    libnsf.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/nsf.c",
            "lib/rbcodec/codecs/libgme/nes_apu.c",
            "lib/rbcodec/codecs/libgme/nes_cpu.c",
            "lib/rbcodec/codecs/libgme/nes_fds_apu.c",
            "lib/rbcodec/codecs/libgme/nes_fme7_apu.c",
            "lib/rbcodec/codecs/libgme/nes_namco_apu.c",
            "lib/rbcodec/codecs/libgme/nes_oscs.c",
            "lib/rbcodec/codecs/libgme/nes_vrc6_apu.c",
            "lib/rbcodec/codecs/libgme/nes_vrc7_apu.c",
            "lib/rbcodec/codecs/libgme/nsf_cpu.c",
            "lib/rbcodec/codecs/libgme/nsf_emu.c",
            "lib/rbcodec/codecs/libgme/nsfe_info.c",
        },
        .flags = &cflags,
    });

    libnsf.defineCMacro("CODEC", null);
    defineCMacros(libnsf);
    addIncludePaths(libnsf);

    const libemu2413 = b.addStaticLibrary(.{
        .name = "emu2413",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libemu2413);

    libemu2413.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/libgme/sms_apu.c",
            "lib/rbcodec/codecs/libgme/sms_fm_apu.c",
            "lib/rbcodec/codecs/libgme/emu2413.c",
            "lib/rbcodec/codecs/libgme/ym2413_emu.c",
        },
        .flags = &cflags,
    });

    libemu2413.defineCMacro("CODEC", null);
    defineCMacros(libemu2413);
    addIncludePaths(libemu2413);

    const libsgc = b.addStaticLibrary(.{
        .name = "sgc",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libsgc);

    libsgc.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/sgc.c",
            "lib/rbcodec/codecs/libgme/sgc_cpu.c",
            "lib/rbcodec/codecs/libgme/sgc_emu.c",
        },
        .flags = &cflags,
    });

    libsgc.defineCMacro("CODEC", null);
    defineCMacros(libsgc);
    addIncludePaths(libsgc);

    const libvgm = b.addStaticLibrary(.{
        .name = "vgm",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libvgm);

    libvgm.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/vgm.c",
            "lib/rbcodec/codecs/libgme/resampler.c",
            "lib/rbcodec/codecs/libgme/vgm_emu.c",
            "lib/rbcodec/codecs/libgme/ym2612_emu.c",
            "lib/rbcodec/codecs/libgme/inflate/bbfuncs.c",
            "lib/rbcodec/codecs/libgme/inflate/inflate.c",
            "lib/rbcodec/codecs/libgme/inflate/mallocer.c",
            "lib/rbcodec/codecs/libgme/inflate/mbreader.c",
        },
        .flags = &cflags,
    });

    libvgm.defineCMacro("CODEC", null);
    defineCMacros(libvgm);
    addIncludePaths(libvgm);

    //const sid = b.addStaticLibrary(.{
    //    .name = "cRSID",
    //    .target = target,
    //       .optimize = optimize,
    // });

    // b.installArtifact(sid);

    // sid.addCSourceFiles(.{
    //     .files = &[_][]const u8{
    //        "lib/rbcodec/codecs/sid.c",
    //        "lib/rbcodec/codecs/cRSID/libcRSID.c",
    //    },
    //    .flags = &cflags,
    //});

    // sid.defineCMacro("CODEC", null);
    // defineCMacros(sid);
    // addIncludePaths(sid);

    defineCMacros(exe);
    addIncludePaths(exe);

    const libkss = b.addStaticLibrary(.{
        .name = "kss",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(libkss);

    libkss.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/kss.c",
            "lib/rbcodec/codecs/libgme/kss_cpu.c",
            "lib/rbcodec/codecs/libgme/kss_emu.c",
            "lib/rbcodec/codecs/libgme/kss_scc_apu.c",
            "lib/rbcodec/codecs/libgme/opl_apu.c",
            "lib/rbcodec/codecs/libgme/emu8950.c",
            "lib/rbcodec/codecs/libgme/emuadpcm.c",
        },
        .flags = &cflags,
    });

    libkss.defineCMacro("CODEC", null);
    defineCMacros(libkss);
    addIncludePaths(libkss);

    const aac_bsf = b.addStaticLibrary(.{
        .name = "aac_bsf",
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(aac_bsf);

    aac_bsf.addCSourceFiles(.{
        .files = &[_][]const u8{
            "lib/rbcodec/codecs/aac_bsf.c",
        },
        .flags = &cflags,
    });

    aac_bsf.defineCMacro("CODEC", null);
    defineCMacros(aac_bsf);
    addIncludePaths(aac_bsf);

    exe.linkLibrary(libfirmware);
    exe.linkLibrary(libspeex_voice);
    exe.linkLibrary(librbcodec);
    exe.linkLibrary(libskinparser);
    exe.linkLibrary(libfixedpoint);
    exe.linkLibrary(libuisimulator);
    exe.linkSystemLibrary("SDL");
    exe.linkLibC();
}

fn defineCMacros(c: *std.Build.Step.Compile) void {
    c.defineCMacro("_USE_MISC", null);
    c.defineCMacro("ROCKBOX", null);
    c.defineCMacro("MEMORYSIZE", "8");
    c.defineCMacro("SDLAPP", null);
    c.defineCMacro("TARGET_ID", "73");
    c.defineCMacro("TARGET_NAME", "\"sdlapp\"");
    c.defineCMacro("YEAR", "2024");
    c.defineCMacro("MONTH", "09");
    c.defineCMacro("DAY", "01");
    c.defineCMacro("OS_USE_BYTESWAP_H", null);
    c.defineCMacro("APPLICATION", null);
    c.defineCMacro("_GNU_SOURCE", "1");
    c.defineCMacro("_REENTRANT", null);
}

fn addIncludePaths(c: *std.Build.Step.Compile) void {
    c.addIncludePath(.{ .cwd_relative = "/usr/include" });
    c.addIncludePath(.{ .cwd_relative = "/usr/include/x86_64-linux-gnu" });
    c.addIncludePath(.{ .cwd_relative = "/usr/include/SDL" });
    c.addIncludePath(.{ .cwd_relative = "./firmware/export" });
    c.addIncludePath(.{ .cwd_relative = "./firmware/drivers" });
    c.addIncludePath(.{ .cwd_relative = "./build" });
    c.addIncludePath(.{ .cwd_relative = "./firmware/include" });
    c.addIncludePath(.{ .cwd_relative = "./firmware/target/hosted/sdl" });
    c.addIncludePath(.{ .cwd_relative = "./firmware/target/hosted" });
    c.addIncludePath(.{ .cwd_relative = "./firmware/target/hosted/sdl/app" });
    c.addIncludePath(.{ .cwd_relative = "./apps" });
    c.addIncludePath(.{ .cwd_relative = "./apps/gui" });
    c.addIncludePath(.{ .cwd_relative = "./apps/recorder" });
    c.addIncludePath(.{ .cwd_relative = "./lib/rbcodec" });
    c.addIncludePath(.{ .cwd_relative = "./lib/rbcodec/metadata" });
    c.addIncludePath(.{ .cwd_relative = "./lib/rbcodec/codecs/libspeex" });
    c.addIncludePath(.{ .cwd_relative = "./firmware/kernel/include" });
    c.addIncludePath(.{ .cwd_relative = "./firmware" });
    c.addIncludePath(.{ .cwd_relative = "./firmware/asm" });
    c.addIncludePath(.{ .cwd_relative = "./lib/rbcodec/dsp" });
    c.addIncludePath(.{ .cwd_relative = "./lib/rbcodec/codecs" });
    c.addIncludePath(.{ .cwd_relative = "./lib/skin_parser" });
    c.addIncludePath(.{ .cwd_relative = "./build/lang" });
    c.addIncludePath(.{ .cwd_relative = "./lib/skin_parser" });
    c.addIncludePath(.{ .cwd_relative = "./apps/gui/skin_engine" });
    c.addIncludePath(.{ .cwd_relative = "./lib/fixedpoint" });
    c.addIncludePath(.{ .cwd_relative = "./lib/rbcodec/codecs/lib" });
    c.addIncludePath(.{ .cwd_relative = "./lib/tlsf/src" });
    c.addIncludePath(.{ .cwd_relative = "./apps/plugins" });
    c.addIncludePath(.{ .cwd_relative = "./lib/rbcodec/codecs/libfaad" });
    c.addIncludePath(.{ .cwd_relative = "./uisimulator/common" });
    c.addIncludePath(.{ .cwd_relative = "./lib/rbcodec/codecs/libopus" });
    c.addIncludePath(.{ .cwd_relative = "./lib/rbcodec/codecs/libopus/celt" });
    c.addIncludePath(.{ .cwd_relative = "./lib/rbcodec/codecs/libopus/silk" });
    c.addIncludePath(.{ .cwd_relative = "./lib/rbcodec/codecs/libtremor" });
    c.addIncludePath(.{ .cwd_relative = "./lib/rbcodec/codecs/libm4a" });
    c.addIncludePath(.{ .cwd_relative = "./lib/rbcodec/codecs/libcook" });
    c.addIncludePath(.{ .cwd_relative = "./lib/rbcodec/codecs/libfaad" });
    c.addIncludePath(.{ .cwd_relative = "./lib/rbcodec/codecs/libatrac" });
    c.addIncludePath(.{ .cwd_relative = "./lib/rbcodec/codecs/libmusepack" });
    c.addIncludePath(.{ .cwd_relative = "./lib/rbcodec/codecs/libtta" });
    c.addIncludePath(.{ .cwd_relative = "./lib/rbcodec/codecs/libwmapro" });
}

const libfirmware_sources = [_][]const u8{
    "firmware/asm/ffs.c",
    "firmware/asm/memset16.c",
    "firmware/asm/mempcpy.c",
    "firmware/ata_idle_notify.c",
    "firmware/events.c",
    "firmware/backlight.c",
    "firmware/buflib_mempool.c",
    "firmware/core_alloc.c",
    "firmware/general.c",
    "firmware/powermgmt.c",
    "firmware/target/hosted/cpuinfo-linux.c",
    "firmware/target/hosted/cpufreq-linux.c",
    "firmware/target/hosted/rtc.c",
    "firmware/system.c",
    "firmware/usb.c",
    "firmware/logf.c",
    "firmware/panic.c",
    "firmware/target/hosted/sdl/button-sdl.c",
    "firmware/target/hosted/sdl/kernel-sdl.c",
    "firmware/target/hosted/sdl/lcd-bitmap.c",
    "firmware/target/hosted/sdl/lcd-sdl.c",
    "firmware/target/hosted/sdl/system-sdl.c",
    "firmware/target/hosted/sdl/load_code-sdl.c",
    "firmware/target/hosted/sdl/timer-sdl.c",
    "firmware/target/hosted/sdl/key_to_touch-sdl.c",
    "firmware/target/hosted/sdl/app/load_code-sdl-app.c",
    "firmware/target/hosted/sdl/app/button-application.c",
    "firmware/target/hosted/filesystem-unix.c",
    "firmware/target/hosted/filesystem-app.c",
    "firmware/chunk_alloc.c",
    "firmware/common/strptokspn.c",
    "firmware/common/ap_int.c",
    "firmware/common/version.c",
    "firmware/common/crc32.c",
    "firmware/common/loader_strerror.c",
    "firmware/common/pathfuncs.c",
    "firmware/common/fdprintf.c",
    "firmware/common/linked_list.c",
    "firmware/common/rectangle.c",
    "firmware/common/strcasecmp.c",
    "firmware/common/strcasestr.c",
    "firmware/common/strnatcmp.c",
    "firmware/common/strlcat.c",
    "firmware/common/strlcpy.c",
    "firmware/common/strmemccpy.c",
    "firmware/common/timefuncs.c",
    "firmware/common/unicode.c",
    "firmware/common/vuprintf.c",
    "firmware/common/zip.c",
    "firmware/common/adler32.c",
    "firmware/common/inflate.c",
    "firmware/scroll_engine.c",
    "firmware/arabjoin.c",
    "firmware/bidi.c",
    "firmware/font_cache.c",
    "firmware/font.c",
    "firmware/hangul.c",
    "firmware/lru.c",
    "firmware/screendump.c",
    "firmware/drivers/lcd-24bit.c",
    "firmware/common/diacritic.c",
    "firmware/drivers/led.c",
    "firmware/drivers/button.c",
    "firmware/drivers/touchscreen.c",
    "firmware/sound.c",
    "firmware/pcm_sampr.c",
    "firmware/pcm.c",
    "firmware/pcm_mixer.c",
    "firmware/pcm_sw_volume.c",
    "firmware/drivers/audio/audiohw-swcodec.c",
    "firmware/drivers/audio/sdl.c",
    "firmware/target/hosted/sdl/pcm-sdl.c",
    "firmware/kernel/mrsw_lock.c",
    "firmware/kernel/mutex.c",
    "firmware/kernel/queue.c",
    "firmware/kernel/semaphore.c",
    "firmware/kernel/thread.c",
    "firmware/kernel/thread-common.c",
    "firmware/kernel/tick.c",
    "firmware/kernel/timeout.c",
    "build/sysfont.c",
};

const libspeex_voice_sources = [_][]const u8{
    "lib/rbcodec/codecs/libspeex/bits.c",
    "lib/rbcodec/codecs/libspeex/cb_search.c",
    "lib/rbcodec/codecs/libspeex/exc_10_16_table.c",
    "lib/rbcodec/codecs/libspeex/exc_10_32_table.c",
    "lib/rbcodec/codecs/libspeex/exc_20_32_table.c",
    "lib/rbcodec/codecs/libspeex/exc_5_256_table.c",
    "lib/rbcodec/codecs/libspeex/exc_5_64_table.c",
    "lib/rbcodec/codecs/libspeex/exc_8_128_table.c",
    "lib/rbcodec/codecs/libspeex/filters.c",
    "lib/rbcodec/codecs/libspeex/gain_table.c",
    "lib/rbcodec/codecs/libspeex/gain_table_lbr.c",
    "lib/rbcodec/codecs/libspeex/hexc_10_32_table.c",
    "lib/rbcodec/codecs/libspeex/hexc_table.c",
    "lib/rbcodec/codecs/libspeex/high_lsp_tables.c",
    "lib/rbcodec/codecs/libspeex/lsp.c",
    "lib/rbcodec/codecs/libspeex/lsp_tables_nb.c",
    "lib/rbcodec/codecs/libspeex/ltp.c",
    "lib/rbcodec/codecs/libspeex/modes.c",
    "lib/rbcodec/codecs/libspeex/modes_wb.c",
    "lib/rbcodec/codecs/libspeex/nb_celp.c",
    "lib/rbcodec/codecs/libspeex/quant_lsp.c",
    "lib/rbcodec/codecs/libspeex/sb_celp.c",
    "lib/rbcodec/codecs/libspeex/speex.c",
    "lib/rbcodec/codecs/libspeex/speex_callbacks.c",
    "lib/rbcodec/codecs/libspeex/oggframing.c",
    "lib/rbcodec/codecs/libspeex/stereo.c",
    "lib/rbcodec/codecs/libspeex/speex_header.c",
    "lib/rbcodec/codecs/libspeex/lpc.c",
    "lib/rbcodec/codecs/libspeex/vbr.c",
    "lib/rbcodec/codecs/libspeex/vq.c",
    "lib/rbcodec/codecs/libspeex/window.c",
    "lib/rbcodec/codecs/libspeex/resample.c",
};

const librbcodec_sources = [_][]const u8{
    "lib/rbcodec/metadata/metadata.c",
    "lib/rbcodec/metadata/id3tags.c",
    "lib/rbcodec/metadata/mp3.c",
    "lib/rbcodec/metadata/mp3data.c",
    "lib/rbcodec/dsp/channel_mode.c",
    "lib/rbcodec/dsp/compressor.c",
    "lib/rbcodec/dsp/crossfeed.c",
    "lib/rbcodec/dsp/dsp_core.c",
    "lib/rbcodec/dsp/pbe.c",
    "lib/rbcodec/dsp/afr.c",
    "lib/rbcodec/dsp/surround.c",
    "lib/rbcodec/dsp/dsp_filter.c",
    "lib/rbcodec/dsp/dsp_misc.c",
    "lib/rbcodec/dsp/dsp_sample_io.c",
    "lib/rbcodec/dsp/dsp_sample_input.c",
    "lib/rbcodec/dsp/dsp_sample_output.c",
    "lib/rbcodec/dsp/eq.c",
    "lib/rbcodec/dsp/resample.c",
    "lib/rbcodec/dsp/pga.c",
    "lib/rbcodec/dsp/tdspeed.c",
    "lib/rbcodec/dsp/tone_controls.c",
    "lib/rbcodec/metadata/replaygain.c",
    "lib/rbcodec/metadata/metadata_common.c",
    "lib/rbcodec/metadata/a52.c",
    "lib/rbcodec/metadata/adx.c",
    "lib/rbcodec/metadata/aiff.c",
    "lib/rbcodec/metadata/ape.c",
    "lib/rbcodec/metadata/asap.c",
    "lib/rbcodec/metadata/asf.c",
    "lib/rbcodec/metadata/au.c",
    "lib/rbcodec/metadata/ay.c",
    "lib/rbcodec/metadata/vtx.c",
    "lib/rbcodec/metadata/flac.c",
    "lib/rbcodec/metadata/gbs.c",
    "lib/rbcodec/metadata/hes.c",
    "lib/rbcodec/metadata/kss.c",
    "lib/rbcodec/metadata/mod.c",
    "lib/rbcodec/metadata/monkeys.c",
    "lib/rbcodec/metadata/mp4.c",
    "lib/rbcodec/metadata/mpc.c",
    "lib/rbcodec/metadata/nsf.c",
    "lib/rbcodec/metadata/ogg.c",
    "lib/rbcodec/metadata/oma.c",
    "lib/rbcodec/metadata/rm.c",
    "lib/rbcodec/metadata/sgc.c",
    "lib/rbcodec/metadata/sid.c",
    "lib/rbcodec/metadata/smaf.c",
    "lib/rbcodec/metadata/spc.c",
    "lib/rbcodec/metadata/tta.c",
    "lib/rbcodec/metadata/vgm.c",
    "lib/rbcodec/metadata/vorbis.c",
    "lib/rbcodec/metadata/vox.c",
    "lib/rbcodec/metadata/wave.c",
    "lib/rbcodec/metadata/wavpack.c",
    "lib/rbcodec/metadata/aac.c",
};

const libskinparser_sources = [_][]const u8{
    "lib/skin_parser/skin_buffer.c",
    "lib/skin_parser/skin_parser.c",
    "lib/skin_parser/skin_scan.c",
    "lib/skin_parser/tag_table.c",
};

const libfixedpoint_sources = [_][]const u8{
    "lib/fixedpoint/fixedpoint.c",
};

const libuisimulator_sources = [_][]const u8{
    "uisimulator/common/dummylib.c",
};

const libcodec_sources = [_][]const u8{
    "lib/rbcodec/codecs/lib/codeclib.c",
    "lib/rbcodec/codecs/lib/ffmpeg_bitstream.c",
    "lib/rbcodec/codecs/lib/mdct_lookup.c",
    "lib/rbcodec/codecs/lib/fft-ffmpeg.c",
    "lib/rbcodec/codecs/lib/mdct.c",
};

const libopus_sources = [_][]const u8{
    "lib/rbcodec/codecs/opus.c",
    "lib/rbcodec/codecs/codec_crt0.c",
    "lib/rbcodec/codecs/libopus/celt/bands.c",
    "lib/rbcodec/codecs/libopus/celt/celt.c",
    "lib/rbcodec/codecs/libopus/celt/celt_decoder.c",
    "lib/rbcodec/codecs/libopus/celt/celt_lpc.c",
    "lib/rbcodec/codecs/libopus/celt/cwrs.c",
    "lib/rbcodec/codecs/libopus/celt/entcode.c",
    "lib/rbcodec/codecs/libopus/celt/entdec.c",
    "lib/rbcodec/codecs/libopus/celt/entenc.c",
    "lib/rbcodec/codecs/libopus/celt/kiss_fft.c",
    "lib/rbcodec/codecs/libopus/celt/laplace.c",
    "lib/rbcodec/codecs/libopus/celt/mathops.c",
    "lib/rbcodec/codecs/libopus/celt/mdct.c",
    "lib/rbcodec/codecs/libopus/celt/modes.c",
    "lib/rbcodec/codecs/libopus/celt/pitch.c",
    "lib/rbcodec/codecs/libopus/celt/quant_bands.c",
    "lib/rbcodec/codecs/libopus/celt/rate.c",
    "lib/rbcodec/codecs/libopus/celt/vq.c",
    "lib/rbcodec/codecs/libopus/silk/bwexpander_32.c",
    "lib/rbcodec/codecs/libopus/silk/bwexpander.c",
    "lib/rbcodec/codecs/libopus/silk/CNG.c",
    "lib/rbcodec/codecs/libopus/silk/code_signs.c",
    "lib/rbcodec/codecs/libopus/silk/dec_API.c",
    "lib/rbcodec/codecs/libopus/silk/decode_core.c",
    "lib/rbcodec/codecs/libopus/silk/decode_frame.c",
    "lib/rbcodec/codecs/libopus/silk/decode_indices.c",
    "lib/rbcodec/codecs/libopus/silk/decode_parameters.c",
    "lib/rbcodec/codecs/libopus/silk/decode_pitch.c",
    "lib/rbcodec/codecs/libopus/silk/decode_pulses.c",
    "lib/rbcodec/codecs/libopus/silk/decoder_set_fs.c",
    "lib/rbcodec/codecs/libopus/silk/gain_quant.c",
    "lib/rbcodec/codecs/libopus/silk/init_decoder.c",
    "lib/rbcodec/codecs/libopus/silk/lin2log.c",
    "lib/rbcodec/codecs/libopus/silk/log2lin.c",
    "lib/rbcodec/codecs/libopus/silk/LPC_analysis_filter.c",
    "lib/rbcodec/codecs/libopus/silk/LPC_fit.c",
    "lib/rbcodec/codecs/libopus/silk/LPC_inv_pred_gain.c",
    "lib/rbcodec/codecs/libopus/silk/NLSF2A.c",
    "lib/rbcodec/codecs/libopus/silk/NLSF_decode.c",
    "lib/rbcodec/codecs/libopus/silk/NLSF_stabilize.c",
    "lib/rbcodec/codecs/libopus/silk/NLSF_unpack.c",
    "lib/rbcodec/codecs/libopus/silk/NLSF_VQ_weights_laroia.c",
    "lib/rbcodec/codecs/libopus/silk/pitch_est_tables.c",
    "lib/rbcodec/codecs/libopus/silk/PLC.c",
    "lib/rbcodec/codecs/libopus/silk/resampler.c",
    "lib/rbcodec/codecs/libopus/silk/resampler_private_AR2.c",
    "lib/rbcodec/codecs/libopus/silk/resampler_private_down_FIR.c",
    "lib/rbcodec/codecs/libopus/silk/resampler_private_IIR_FIR.c",
    "lib/rbcodec/codecs/libopus/silk/resampler_private_up2_HQ.c",
    "lib/rbcodec/codecs/libopus/silk/resampler_rom.c",
    "lib/rbcodec/codecs/libopus/silk/shell_coder.c",
    "lib/rbcodec/codecs/libopus/silk/sort.c",
    "lib/rbcodec/codecs/libopus/silk/stereo_decode_pred.c",
    "lib/rbcodec/codecs/libopus/silk/stereo_MS_to_LR.c",
    "lib/rbcodec/codecs/libopus/silk/sum_sqr_shift.c",
    "lib/rbcodec/codecs/libopus/silk/table_LSF_cos.c",
    "lib/rbcodec/codecs/libopus/silk/tables_gain.c",
    "lib/rbcodec/codecs/libopus/silk/tables_LTP.c",
    "lib/rbcodec/codecs/libopus/silk/tables_NLSF_CB_NB_MB.c",
    "lib/rbcodec/codecs/libopus/silk/tables_NLSF_CB_WB.c",
    "lib/rbcodec/codecs/libopus/silk/tables_other.c",
    "lib/rbcodec/codecs/libopus/silk/tables_pitch_lag.c",
    "lib/rbcodec/codecs/libopus/silk/tables_pulses_per_block.c",
    "lib/rbcodec/codecs/libopus/opus.c",
    "lib/rbcodec/codecs/libopus/opus_decoder.c",
    "lib/rbcodec/codecs/libopus/opus_header.c",
    "lib/rbcodec/codecs/libopus/ogg/framing.c",
};

const vorbis_sources = [_][]const u8{
    "lib/rbcodec/codecs/vorbis.c",
    "lib/rbcodec/codecs/libtremor/bitwise.c",
    "lib/rbcodec/codecs/libtremor/block.c",
    "lib/rbcodec/codecs/libtremor/codebook.c",
    "lib/rbcodec/codecs/libtremor/floor0.c",
    "lib/rbcodec/codecs/libtremor/floor1.c",
    "lib/rbcodec/codecs/libtremor/framing.c",
    "lib/rbcodec/codecs/libtremor/info.c",
    "lib/rbcodec/codecs/libtremor/mapping0.c",
    "lib/rbcodec/codecs/libtremor/registry.c",
    "lib/rbcodec/codecs/libtremor/res012.c",
    "lib/rbcodec/codecs/libtremor/sharedbook.c",
    "lib/rbcodec/codecs/libtremor/synthesis.c",
    "lib/rbcodec/codecs/libtremor/vorbisfile.c",
    "lib/rbcodec/codecs/libtremor/window.c",
    "lib/rbcodec/codecs/libtremor/ctype.c",
    "lib/rbcodec/codecs/libtremor/oggmalloc.c",
};

const libmad_sources = [_][]const u8{
    "lib/rbcodec/codecs/mpa.c",
    "lib/rbcodec/codecs/libmad/bit.c",
    "lib/rbcodec/codecs/libmad/frame.c",
    "lib/rbcodec/codecs/libmad/huffman.c",
    "lib/rbcodec/codecs/libmad/layer12.c",
    "lib/rbcodec/codecs/libmad/layer3.c",
    "lib/rbcodec/codecs/libmad/stream.c",
    "lib/rbcodec/codecs/libmad/synth.c",
};

const libspeex_sources = [_][]const u8{
    "lib/rbcodec/codecs/speex.c",
    "lib/rbcodec/codecs/libspeex/bits.c",
    "lib/rbcodec/codecs/libspeex/cb_search.c",
    "lib/rbcodec/codecs/libspeex/exc_10_16_table.c",
    "lib/rbcodec/codecs/libspeex/exc_10_32_table.c",
    "lib/rbcodec/codecs/libspeex/exc_20_32_table.c",
    "lib/rbcodec/codecs/libspeex/exc_5_256_table.c",
    "lib/rbcodec/codecs/libspeex/exc_5_64_table.c",
    "lib/rbcodec/codecs/libspeex/exc_8_128_table.c",
    "lib/rbcodec/codecs/libspeex/filters.c",
    "lib/rbcodec/codecs/libspeex/gain_table.c",
    "lib/rbcodec/codecs/libspeex/gain_table_lbr.c",
    "lib/rbcodec/codecs/libspeex/hexc_10_32_table.c",
    "lib/rbcodec/codecs/libspeex/hexc_table.c",
    "lib/rbcodec/codecs/libspeex/high_lsp_tables.c",
    "lib/rbcodec/codecs/libspeex/lsp.c",
    "lib/rbcodec/codecs/libspeex/lsp_tables_nb.c",
    "lib/rbcodec/codecs/libspeex/ltp.c",
    "lib/rbcodec/codecs/libspeex/modes.c",
    "lib/rbcodec/codecs/libspeex/modes_wb.c",
    "lib/rbcodec/codecs/libspeex/nb_celp.c",
    "lib/rbcodec/codecs/libspeex/quant_lsp.c",
    "lib/rbcodec/codecs/libspeex/sb_celp.c",
    "lib/rbcodec/codecs/libspeex/speex.c",
    "lib/rbcodec/codecs/libspeex/speex_callbacks.c",
    "lib/rbcodec/codecs/libspeex/oggframing.c",
    "lib/rbcodec/codecs/libspeex/stereo.c",
    "lib/rbcodec/codecs/libspeex/speex_header.c",
};

const all_sources = [_][]const u8{
    "firmware/common/config.c",
    "apps/action.c",
    "apps/abrepeat.c",
    "build/apps/bitmaps/mono/default_icons.c",
    "build/apps/bitmaps/native/rockboxlogo.320x98x16.c",
    "build/apps/bitmaps/native/usblogo.176x48x16.c",
    "apps/bookmark.c",
    "apps/core_keymap.c",
    "apps/debug_menu.c",
    "apps/filetypes.c",
    "apps/fileop.c",
    "apps/language.c",
    "apps/main.c",
    "apps/menu.c",
    "apps/menus/menu_common.c",
    "apps/menus/display_menu.c",
    "apps/menus/theme_menu.c",
    "apps/menus/plugin_menu.c",
    "apps/menus/eq_menu.c",
    "apps/buffering.c",
    "apps/voice_thread.c",
    "apps/rbcodec_helpers.c",
    "apps/menus/main_menu.c",
    "apps/menus/playback_menu.c",
    "apps/menus/playlist_menu.c",
    "apps/menus/settings_menu.c",
    "apps/menus/sound_menu.c",
    "apps/menus/time_menu.c",
    "apps/misc.c",
    "apps/open_plugin.c",
    "apps/onplay.c",
    "apps/playlist.c",
    "apps/playlist_catalog.c",
    "apps/playlist_viewer.c",
    "apps/plugin.c",
    "apps/root_menu.c",
    "apps/screens.c",
    "apps/settings.c",
    "apps/settings_list.c",
    "apps/shortcuts.c",
    "apps/status.c",
    "apps/cuesheet.c",
    "apps/talk.c",
    "apps/tree.c",
    "apps/tagtree.c",
    "apps/filetree.c",
    "apps/screen_access.c",
    "apps/gui/icon.c",
    "apps/gui/list.c",
    "apps/gui/line.c",
    "apps/gui/bitmap/list.c",
    "apps/gui/bitmap/list-skinned.c",
    "apps/gui/option_select.c",
    "apps/gui/pitchscreen.c",
    "apps/gui/quickscreen.c",
    "apps/gui/folder_select.c",
    "apps/gui/mask_select.c",
    "apps/gui/wps.c",
    "apps/gui/scrollbar.c",
    "apps/gui/splash.c",
    "apps/gui/statusbar.c",
    "apps/gui/statusbar-skinned.c",
    "apps/gui/yesno.c",
    "apps/gui/viewport.c",
    "apps/gui/skin_engine/skin_backdrops.c",
    "apps/gui/skin_engine/skin_display.c",
    "apps/gui/skin_engine/skin_engine.c",
    "apps/gui/skin_engine/skin_parser.c",
    "apps/gui/skin_engine/skin_render.c",
    "apps/gui/skin_engine/skin_tokens.c",
    "apps/gui/skin_engine/skin_touchsupport.c",
    "apps/gui/backdrop.c",
    "apps/recorder/bmp.c",
    "apps/recorder/icons.c",
    "apps/recorder/keyboard.c",
    "apps/recorder/peakmeter.c",
    "apps/recorder/resize.c",
    "apps/recorder/jpeg_load.c",
    "apps/recorder/albumart.c",
    "apps/gui/color_picker.c",
    "apps/audio_thread.c",
    "apps/pcmbuf.c",
    "apps/codec_thread.c",
    "apps/playback.c",
    "apps/codecs.c",
    "apps/beep.c",
    "apps/tagcache.c",
    "apps/keymaps/keymap-touchscreen.c",
    "apps/keymaps/keymap-sdl.c",
    "build/lang/lang_core.c",
};

const cflags = [_][]const u8{
    "-W",
    "-Wall",
    "-Wextra",
    "-Os",
    "-Wstrict-prototypes",
    "-pipe",
    "-std=gnu11",
    "-Wno-gnu",
    "-fPIC",
    "-fvisibility=hidden",
    "-Wno-pointer-to-int-cast",
    "-fno-delete-null-pointer-checks",
    "-fno-strict-overflow",
    "-fno-builtin",
    "-g",
    "-Wno-unused-result",
    "-Wno-pointer-sign",
    "-Wno-override-init",
    "-Wno-shift-negative-value",
    "-Wno-unused-const-variable",
    "-Wno-unused-variable",
    "-Wno-unused-but-set-variable",
    "-O2",
    "-Wno-tautological-compare",
    "-Wno-expansion-to-defined",
};
