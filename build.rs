use std::io::Result;

use cc::Build;
use static_files::NpmBuild;

fn build_sqlite_shell() {
    println!("cargo:rerun-if-changed=linenoise");
    println!("cargo:rerun-if-changed=sqlite-shell");

    Build::new()
        .define("HAVE_LINENOISE", None)
        .files(["./linenoise/linenoise.c", "./sqlite-shell/shell.c"])
        .flag_if_supported("-Wno-implicit-fallthrough")
        .flag_if_supported("-Wno-unused-function")
        .flag_if_supported("-Wno-sign-compare")
        .include("./linenoise")
        .compile("sqlite-shell");
}

fn main() -> Result<()> {
    build_sqlite_shell();

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
