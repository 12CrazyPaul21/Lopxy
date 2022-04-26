#![allow(dead_code)]

use std::collections::HashMap;

macro_rules! collect_assets {
    () => {
        std::collections::HashMap::new()
    };
    ([$($k:tt = $v:expr),*]) => {
        {
            let mut assets_map = std::collections::HashMap::<String, Vec<u8>>::new();

            $(
                assets_map.insert($k.to_string(), include_bytes!($v).to_vec());
            )*

            assets_map
        }
    };
}

fn release_asset(path: &std::path::PathBuf, buffer: &[u8], force: bool) -> std::io::Result<()> {
    if path.exists() && !force {
        return Ok(());
    }

    match path.parent() {
        Some(parent) => {
            if !parent.exists() {
                std::fs::create_dir_all(&parent)?;
            }
        },
        None => { }
    }

    std::fs::write(path, buffer)
}

fn release(asset_folder: &std::path::PathBuf, force: bool) -> std::io::Result<()> {
    for (name, buffer) in collect() {
        let mut asset_path = asset_folder.clone();
        asset_path.push(name);
        match release_asset(&asset_path, &buffer, force) {
            Ok(_) => {},
            Err(err) => {
                if !force {
                    return Err(err);
                }
            }
        }
    }

    Ok(())
}

pub fn try_release(asset_folder: &std::path::PathBuf) -> std::io::Result<()> {
    release(asset_folder, false)
}

pub fn force_release(asset_folder: &std::path::PathBuf) -> std::io::Result<()> {
    release(asset_folder, true)
}

pub fn collect() -> HashMap::<String, Vec<u8>> {
    collect_assets!([
        "index.html" = "../static/index.html",
        "index.js" = "../static/index.js",
        "index.css" = "../static/index.css",
        "favicon.ico" = "../static/favicon.ico"
    ])
}