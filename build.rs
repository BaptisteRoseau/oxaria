//! Compiles the vendored litehtml engine (C++), its gumbo HTML parser (C) and
//! the C wrapper that `src/litehtml/sys.rs` binds to, into static libraries
//! linked into oxaria. See vendor/PATCHES.md.

use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let csrc_dir = manifest_dir.join("vendor/litehtml_c");

    build_vendored(&manifest_dir, &out_dir, &csrc_dir);

    println!("cargo:rerun-if-changed={}", csrc_dir.display());
}

fn build_vendored(manifest_dir: &Path, out_dir: &Path, csrc_dir: &Path) {
    let vendor_dir = manifest_dir.join("vendor/litehtml");
    let gumbo_src = vendor_dir.join("src/gumbo");
    let gumbo_include = gumbo_src.join("include");
    let gumbo_private_include = gumbo_include.join("gumbo");
    let litehtml_src = vendor_dir.join("src");
    let litehtml_include = vendor_dir.join("include");

    // Gumbo (C99)
    let mut gumbo = cc::Build::new();
    gumbo
        .cargo_metadata(false)
        .files(
            [
                "attribute.c",
                "char_ref.c",
                "error.c",
                "parser.c",
                "string_buffer.c",
                "string_piece.c",
                "tag.c",
                "tokenizer.c",
                "utf8.c",
                "util.c",
                "vector.c",
            ]
            .iter()
            .map(|f| gumbo_src.join(f)),
        )
        .include(&gumbo_include)
        .include(&gumbo_private_include)
        .std("c99")
        .warnings(false);
    // MSVC has no <strings.h>; gumbo ships a shim for it, as its CMakeLists does.
    if is_msvc() {
        gumbo.include(gumbo_src.join("visualc/include"));
    }
    gumbo.compile("gumbo");

    // litehtml (C++17)
    cpp_build()
        .files(
            [
                "codepoint.cpp",
                "css_length.cpp",
                "css_selector.cpp",
                "css_tokenizer.cpp",
                "css_parser.cpp",
                "document.cpp",
                "document_container.cpp",
                "el_anchor.cpp",
                "el_base.cpp",
                "el_before_after.cpp",
                "el_body.cpp",
                "el_break.cpp",
                "el_cdata.cpp",
                "el_comment.cpp",
                "el_div.cpp",
                "element.cpp",
                "el_font.cpp",
                "el_image.cpp",
                "el_link.cpp",
                "el_para.cpp",
                "el_script.cpp",
                "el_space.cpp",
                "el_style.cpp",
                "el_table.cpp",
                "el_td.cpp",
                "el_text.cpp",
                "el_title.cpp",
                "el_tr.cpp",
                "encodings.cpp",
                "html.cpp",
                "html_tag.cpp",
                "html_microsyntaxes.cpp",
                "iterators.cpp",
                "media_query.cpp",
                "style.cpp",
                "stylesheet.cpp",
                "table.cpp",
                "tstring_view.cpp",
                "url.cpp",
                "url_path.cpp",
                "utf8_strings.cpp",
                "web_color.cpp",
                "num_cvt.cpp",
                "strtod.cpp",
                "string_id.cpp",
                "css_properties.cpp",
                "line_box.cpp",
                "css_borders.cpp",
                "render_item.cpp",
                "render_block_context.cpp",
                "render_block.cpp",
                "render_inline_context.cpp",
                "render_table.cpp",
                "render_flex.cpp",
                "render_image.cpp",
                "formatting_context.cpp",
                "flex_item.cpp",
                "flex_line.cpp",
                "background.cpp",
                "gradient.cpp",
            ]
            .iter()
            .map(|f| litehtml_src.join(f)),
        )
        .include(&litehtml_include)
        .include(litehtml_include.join("litehtml"))
        .include(&litehtml_src)
        .include(&gumbo_include)
        .compile("litehtml");

    // C wrapper (C++17)
    cpp_build()
        .file(csrc_dir.join("litehtml_c.cpp"))
        .include(&litehtml_include)
        .include(&gumbo_include)
        .compile("litehtml_c");

    // Link order: dependents first
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=litehtml_c");
    println!("cargo:rustc-link-lib=static=litehtml");
    println!("cargo:rustc-link-lib=static=gumbo");
    link_cpp_stdlib();

    println!("cargo:rerun-if-changed={}", vendor_dir.display());
}

/// A C++17 build. On MSVC, `/utf-8` as litehtml's CMakeLists sets it: without
/// it, MSVC reads the sources' non-ASCII literals in the system code page.
fn cpp_build() -> cc::Build {
    let mut build = cc::Build::new();
    build
        .cargo_metadata(false)
        .cpp(true)
        .std("c++17")
        .warnings(false);
    if is_msvc() {
        build.flag("/utf-8");
    }
    build
}

/// `cfg!` in a build script reads the host, not the target, so cross builds
/// need `CARGO_CFG_TARGET_*`.
fn is_msvc() -> bool {
    env::var("CARGO_CFG_TARGET_ENV").is_ok_and(|target_env| target_env == "msvc")
}

/// Link the target's C++ standard library. MSVC links its C++ runtime on its
/// own and has no `stdc++` to link.
fn link_cpp_stdlib() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    match target_os.as_str() {
        _ if is_msvc() => {}
        "macos" | "ios" => println!("cargo:rustc-link-lib=c++"),
        _ => println!("cargo:rustc-link-lib=stdc++"),
    }
}
