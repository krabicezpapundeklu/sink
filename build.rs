use std::io::Result;

use cc::Build;
use static_files::NpmBuild;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=sqlite-shell");

    Build::new()
        .file("./sqlite-shell/shell.c")
        .compile("sqlite-shell");

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
