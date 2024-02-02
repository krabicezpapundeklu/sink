use std::{env::var_os, io::Result};

use cc::Build;
use static_files::NpmBuild;

fn build_sqlite_shell() {
    println!("cargo:rerun-if-changed=sqlite-shell");

    let mut build = Build::new();

    build
        .file("./sqlite-shell/shell.c")
        .flag_if_supported("-Wno-cast-function-type")
        .flag_if_supported("-Wno-implicit-fallthrough")
        .flag_if_supported("-Wno-unused-function")
        .flag_if_supported("-Wno-unused-variable")
        .flag_if_supported("-Wno-sign-compare");

    if var_os("CARGO_CFG_UNIX").is_some() {
        println!("cargo:rerun-if-changed=linenoise");

        build
            .define("HAVE_LINENOISE", None)
            .file("./linenoise/linenoise.c")
            .include("./linenoise");
    }

    build.compile("sqlite-shell");
}

fn build_web() -> Result<()> {
    println!("cargo:rerun-if-changed=event.types.json");
    println!("cargo:rerun-if-changed=item.types.json");

    println!("cargo:rerun-if-changed=web/package.json");
    println!("cargo:rerun-if-changed=web/purgecss.js");
    println!("cargo:rerun-if-changed=web/src");
    println!("cargo:rerun-if-changed=web/static");
    println!("cargo:rerun-if-changed=web/svelte.config.js");
    println!("cargo:rerun-if-changed=web/tsconfig.json");
    println!("cargo:rerun-if-changed=web/vite.config.ts");

    NpmBuild::new("web")
        .install()?
        .run("build")?
        .target("web/build")
        .to_resource_dir()
        .build()
}

fn main() -> Result<()> {
    build_sqlite_shell();
    build_web()
}
