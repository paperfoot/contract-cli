// Embed contract templates + shared helpers. Extract on first use into the
// shared accounting-suite assets dir, namespaced under `contracts/`.

use rust_embed::RustEmbed;

use crate::config;
use crate::error::Result;

#[derive(RustEmbed)]
#[folder = "typst/"]
#[prefix = ""]
pub struct Assets;

fn root() -> Result<std::path::PathBuf> {
    Ok(config::assets_path()?.join("contracts"))
}

pub fn ensure_extracted() -> Result<()> {
    let root = root()?;
    std::fs::create_dir_all(&root)?;
    for path in Assets::iter() {
        let file = Assets::get(&path).expect("embedded asset");
        let dest = root.join(path.as_ref());
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let needs_write = match std::fs::read(&dest) {
            Ok(existing) => existing != file.data.as_ref(),
            Err(_) => true,
        };
        if needs_write {
            std::fs::write(&dest, file.data.as_ref())?;
        }
    }
    Ok(())
}

pub fn template_dir() -> Result<std::path::PathBuf> {
    Ok(root()?.join("templates"))
}

pub fn template_path(name: &str) -> Result<std::path::PathBuf> {
    Ok(template_dir()?.join(format!("{name}.typ")))
}

pub fn list_templates() -> Result<Vec<String>> {
    let dir = template_dir()?;
    let mut names = Vec::new();
    if dir.exists() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("typ") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    names.push(name.to_string());
                }
            }
        }
    }
    names.sort();
    Ok(names)
}

pub fn has_template(name: &str) -> Result<bool> {
    Ok(template_path(name)?.exists())
}

pub fn project_root() -> Result<std::path::PathBuf> {
    root()
}
