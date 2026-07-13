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

fn humanize_slug(slug: &str) -> String {
    slug.split(['_', '-'])
        .filter(|w| !w.is_empty())
        .enumerate()
        .map(|(i, w)| {
            if i == 0 {
                let mut c = w.chars();
                c.next()
                    .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                    .unwrap_or_default()
            } else {
                w.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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
/// Single-pass scan: substituted values are never themselves re-expanded, so
/// a value containing `{{...}}` text renders literally instead of recursing.
pub fn expand(body: &str, vars: &BTreeMap<String, String>) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find("}}") {
            Some(end) => {
                let key = &after[..end];
                match vars.get(key) {
                    Some(v) => out.push_str(v),
                    None => {
                        out.push_str("{{");
                        out.push_str(key);
                        out.push_str("}}");
                    }
                }
                rest = &after[end + 2..];
            }
            None => {
                out.push_str("{{");
                rest = after;
            }
        }
    }
    out.push_str(rest);
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
        let def = pack.clauses.get(slug);
        let (head_override, body_override) = overrides
            .get(slug)
            .cloned()
            .unwrap_or((None, None));
        // Custom clauses (added with --body / --from-file) don't exist in the
        // pack — their heading/body live entirely in the override.
        let heading = match (head_override, def) {
            (Some(h), _) => h,
            (None, Some(d)) => d.heading.clone(),
            (None, None) => humanize_slug(slug),
        };
        let body_template = match (body_override, def) {
            (Some(b), _) => b,
            (None, Some(d)) => d.body.clone(),
            (None, None) => {
                return Err(AppError::NotFound(format!(
                    "clause '{slug}' is not in the pack and has no custom body. \
                     Set one with: contract clauses edit <number> {slug} --body \"…\""
                )))
            }
        };
        out.push(ResolvedClause {
            position: i as i64,
            slug: slug.clone(),
            heading: expand(&heading, vars),
            body: expand(&body_template, vars),
        });
    }
    Ok(out)
}
