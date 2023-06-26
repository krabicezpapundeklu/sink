use std::io::Result;

use cc::Build;
use static_files::NpmBuild;

fn build_sqlite_shell() {
    println!("cargo:rerun-if-changed=sqlite-shell");

    Build::new()
        .file("./sqlite-shell/shell.c")
        .flag_if_supported("-Wno-implicit-fallthrough")
        .flag_if_supported("-Wno-unused-function")
        .flag_if_supported("-Wno-sign-compare")
        .compile("sqlite-shell");
}

fn main() -> Result<()> {
    build_sqlite_shell();

    println!("cargo:rerun-if-changed=web/adapter");
    println!("cargo:rerun-if-changed=web/src");
    println!("cargo:rerun-if-changed=web/static");
    println!("cargo:rerun-if-changed=web/package.json");
    println!("cargo:rerun-if-changed=web/purgecss.config.cjs");
    println!("cargo:rerun-if-changed=web/svelte.config.js");
    println!("cargo:rerun-if-changed=web/tsconfig.json");
    println!("cargo:rerun-if-changed=web/vite.config.ts");

    NpmBuild::new("web")
        .install()?
        .run("build")?
        .target("web/build/client")
        .to_resource_dir()
        .build()
}
