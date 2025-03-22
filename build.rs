use std::io::Result;

use static_files::NpmBuild;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=event.types.json");
    println!("cargo:rerun-if-changed=item.types.json");
    println!("cargo:rerun-if-changed=migrations");
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
