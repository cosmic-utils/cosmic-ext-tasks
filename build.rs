// SPDX-License-Identifier: GPL-3.0-only

use std::{env, error::Error, fs, path::PathBuf};

use xdgen::{App, Context, FluentString};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=i18n");
    println!("cargo:rerun-if-changed=resources/app.desktop");
    println!("cargo:rerun-if-changed=resources/app.metainfo.xml");
    println!("cargo:rerun-if-env-changed=CARGO_TARGET_DIR");

    let context = Context::new("i18n", env!("CARGO_PKG_NAME"))?;
    let desktop = App::new(FluentString("desktop-name"))
        .comment(FluentString("appstream-summary"))
        .keywords(FluentString("app-keywords"))
        .expand_desktop("resources/app.desktop", &context)?;
    let metainfo = App::new(FluentString("appstream-name"))
        .comment(FluentString("appstream-summary"))
        .expand_metainfo("resources/app.metainfo.xml", &context)?;

    let mut output = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target"));
    output.push("xdgen");
    fs::create_dir_all(&output)?;
    fs::write(output.join("app.desktop"), desktop)?;
    fs::write(output.join("app.metainfo.xml"), metainfo)?;
    Ok(())
}
