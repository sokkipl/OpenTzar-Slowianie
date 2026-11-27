//! Builds the Core Audio, AppKit and AVFoundation bridges on macOS, or the
//! XAudio2, Media Foundation and GDI bridges on Windows, using the target's
//! C++ compiler: cl for windows-msvc, the configured CXX for windows-gnu.
//! With MSVC the game's executable also gets Tzar's icon.

fn main() {
    println!("cargo:rerun-if-changed=tools/audio_mac.cpp");
    println!("cargo:rerun-if-changed=tools/dialog_mac.mm");
    println!("cargo:rerun-if-changed=tools/media_mac.mm");
    println!("cargo:rerun-if-changed=tools/audio_win.cpp");
    println!("cargo:rerun-if-changed=tools/media_win.cpp");
    println!("cargo:rerun-if-changed=tools/text_win.cpp");
    println!("cargo:rerun-if-env-changed=CXX");
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("apple-darwin") {
        cc::Build::new()
            .cpp(true)
            .file("tools/audio_mac.cpp")
            .flag("-std=c++17")
            .flag("-fno-exceptions")
            .flag("-fno-rtti")
            .opt_level(2)
            .compile("tz_audio");
        println!("cargo:rustc-link-lib=framework=AudioToolbox");
        println!("cargo:rustc-link-lib=framework=AudioUnit");
        cc::Build::new()
            .cpp(true)
            .file("tools/dialog_mac.mm")
            .file("tools/media_mac.mm")
            .flag("-std=c++17")
            .flag("-fobjc-arc")
            .opt_level(2)
            .compile("tz_dialog");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=AVFoundation");
        println!("cargo:rustc-link-lib=framework=CoreMedia");
        println!("cargo:rustc-link-lib=framework=CoreVideo");
        return;
    }
    if !target.contains("windows") {
        return;
    }
    // TEST3: do not embed the original Tzar icon: its redistribution rights
    // have not been established. The Windows executable uses a neutral icon.
    let mut build = cc::Build::new();
    build
        .cpp(true)
        // The bridge uses no C++ library features: plain C ABI, no exceptions.
        .cpp_link_stdlib(None)
        .file("tools/audio_win.cpp")
        .file("tools/media_win.cpp")
        .file("tools/text_win.cpp")
        .opt_level(2);
    if target.ends_with("windows-gnu") {
        build
            .flag("-std=c++17")
            .flag("-fno-exceptions")
            .flag("-fno-rtti");
    } else {
        build.flag_if_supported("/std:c++17");
    }
    build.compile("tz_audio");
    println!("cargo:rustc-link-lib=ole32");
    println!("cargo:rustc-link-lib=winmm");
    println!("cargo:rustc-link-lib=uuid");
    println!("cargo:rustc-link-lib=gdi32");
    println!("cargo:rustc-link-lib=mfplat");
    println!("cargo:rustc-link-lib=mfreadwrite");
    println!("cargo:rustc-link-lib=mfuuid");
    println!("cargo:rustc-link-lib=shlwapi");
    if target.ends_with("windows-gnu") {
        // Keep third-party cache reusable while removing oversized DWARF data
        // from final Windows binaries and test executables.
        println!("cargo:rustc-link-arg=-Wl,--strip-debug");
    }
}

