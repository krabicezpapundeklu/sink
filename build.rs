use std::io::Result;

use cc::Build;
use static_files::NpmBuild;

fn main() -> Result<()> {
    Build::new()
        .file("./sqlite-shell/shell.c")
        .compile("sqlite-shell");

    NpmBuild::new("web")
        .install()?
        .run("build")?
        .target("web/build")
        .change_detection()
        .to_resource_dir()
        .build()
}
