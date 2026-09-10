// Embed contract templates + shared helpers. Extract on first use into the
// shared accounting-suite assets dir, namespaced under `contracts/`.

use rust_embed::RustEmbed;

use crate::config;
use crate::error::{AppError, Result};

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
    validate_name(name)?;
    Ok(template_dir()?.join(format!("{name}.typ")))
}

pub fn list_templates() -> Result<Vec<String>> {
    ensure_extracted()?;
    let dir = template_dir()?;
    let mut names = Vec::new();
    if dir.exists() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("typ")
                && let Some(name) = path.file_stem().and_then(|s| s.to_str())
            {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::InvalidInput(
            "template names may contain only letters, digits, hyphens and underscores".into(),
        ));
    }
    Ok(())
}
pub fn has_template(name: &str) -> Result<bool> {
    validate_name(name)?;
    if Assets::get(&format!("templates/{name}.typ")).is_some() {
        return Ok(true);
    }
    Ok(template_path(name)?.is_file())
}

/// Per-template metadata, parsed from the `//!` doc-comment header at the top
/// of each .typ file. Metadata travels with the design, so user-dropped
/// custom templates participate in discovery the moment they document
/// themselves the same way.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TemplateMeta {
    pub name: String,
    pub description: String,
    pub mood: Vec<String>,
    pub tags: Vec<String>,
    pub fonts: String,
    pub paper: String,
}

pub fn template_meta(name: &str) -> Result<TemplateMeta> {
    let src = std::fs::read_to_string(template_path(name)?)?;
    let mut meta = TemplateMeta {
        name: name.to_string(),
        description: String::new(),
        mood: Vec::new(),
        tags: Vec::new(),
        fonts: String::new(),
        paper: String::new(),
    };
    for line in src.lines() {
        let Some(rest) = line.trim().strip_prefix("//!") else {
            if line.trim().is_empty() || line.trim().starts_with("//") {
                continue;
            }
            break; // first real code line ends the header block
        };
        if let Some((key, value)) = rest.split_once(':') {
            let value = value.trim().to_string();
            match key.trim() {
                "description" => meta.description = value,
                "mood" => meta.mood = split_list(&value),
                "tags" => meta.tags = split_list(&value),
                "fonts" => meta.fonts = value,
                "paper" => meta.paper = value,
                _ => {}
            }
        }
    }
    if meta.description.is_empty() {
        meta.description = "(custom template — no //! metadata header)".into();
    }
    Ok(meta)
}

pub fn list_template_meta() -> Result<Vec<TemplateMeta>> {
    list_templates()?.iter().map(|n| template_meta(n)).collect()
}

fn split_list(s: &str) -> Vec<String> {
    s.split(',')
        .map(|t| t.trim().to_lowercase())
        .filter(|t| !t.is_empty())
        .collect()
}

pub fn project_root() -> Result<std::path::PathBuf> {
    root()
}
