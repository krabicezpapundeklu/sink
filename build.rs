use std::{env, io::Result};

use static_files::NpmBuild;

fn main() -> Result<()> {
    let build_web = env::var("BUILD_WEB").map(|v| v == "1").unwrap_or(true);

    if build_web {
        NpmBuild::new("web")
            .install()?
            .run("build")?
            .target("web/build")
            .change_detection()
            .to_resource_dir()
            .build()
    } else {
        Ok(())
    }
}
