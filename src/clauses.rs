// ═══════════════════════════════════════════════════════════════════════════
// Clauses — embedded clause packs (per kind × pack slug) + variable expansion.
//
// Packs live in `clauses/<kind>/<pack>.toml` and are baked into the binary
// via rust-embed. Each pack lists the default clause set (slug + order) plus
// the heading and Markdown body for every available clause. Body templates
// substitute `{{var}}` tokens at render time using a small ContractVars dict
// the Rust side computes from the DB row + terms JSON.
// ═══════════════════════════════════════════════════════════════════════════

use std::collections::BTreeMap;

use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

#[derive(RustEmbed)]
#[folder = "clauses/"]
pub struct PackAssets;

#[derive(Debug, Clone, Deserialize)]
pub struct Pack {
    pub pack: PackMeta,
    /// slug -> ClauseDef
    pub clauses: BTreeMap<String, ClauseDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackMeta {
    pub slug: String,
    pub name: String,
    pub version: String,
    pub kind: String,
    /// Slugs of clauses that should be included by default (in order).
    pub default_clauses: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClauseDef {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedClause {
    pub position: i64,
    pub slug: String,
    pub heading: String,
    pub body: String,
}

pub fn load_pack(kind: &str, pack_slug: &str) -> Result<Pack> {
    let path = format!("{kind}/{pack_slug}.toml");
    let file = PackAssets::get(&path)
        .ok_or_else(|| AppError::NotFound(format!("clause pack '{kind}/{pack_slug}'")))?;
    let text = std::str::from_utf8(&file.data)
        .map_err(|e| AppError::Other(format!("non-utf8 pack {path}: {e}")))?;
    toml::from_str::<Pack>(text)
        .map_err(|e| AppError::Other(format!("invalid pack {path}: {e}")))
}

pub fn list_packs() -> Vec<(String, String)> {
    PackAssets::iter()
        .filter_map(|p| {
            let p = p.as_ref();
            let (kind, file) = p.split_once('/')?;
            let pack_slug = file.strip_suffix(".toml")?;
            Some((kind.to_string(), pack_slug.to_string()))
        })
        .collect()
}

/// Resolve a body template against a flat variable map. `{{name}}` tokens are
/// replaced; unknown tokens are left in place (so previews don't blow up).
pub fn expand(body: &str, vars: &BTreeMap<String, String>) -> String {
    let mut out = body.to_string();
    for (k, v) in vars {
        let token = format!("{{{{{k}}}}}");
        out = out.replace(&token, v);
    }
    out
}

/// Apply a pack + overrides to produce the final ordered clause list.
///
/// `included` is the contract's chosen clause order (typically pack defaults
/// modulo --include/--exclude). `overrides` maps slug -> (heading?, body?)
/// for clauses the user has customised via `contract clauses edit`.
pub fn resolve(
    pack: &Pack,
    included: &[String],
    overrides: &BTreeMap<String, (Option<String>, Option<String>)>,
    vars: &BTreeMap<String, String>,
) -> Result<Vec<ResolvedClause>> {
    let mut out = Vec::with_capacity(included.len());
    for (i, slug) in included.iter().enumerate() {
        let def = pack
            .clauses
            .get(slug)
            .ok_or_else(|| AppError::NotFound(format!("clause '{slug}' not in pack")))?;
        let (head_override, body_override) = overrides
            .get(slug)
            .cloned()
            .unwrap_or((None, None));
        let heading = head_override.unwrap_or_else(|| def.heading.clone());
        let body_template = body_override.unwrap_or_else(|| def.body.clone());
        out.push(ResolvedClause {
            position: i as i64,
            slug: slug.clone(),
            heading: expand(&heading, vars),
            body: expand(&body_template, vars),
        });
    }
    Ok(out)
}
