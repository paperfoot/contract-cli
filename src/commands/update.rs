// Distribution-aware update, per agent-cli-framework docs/update-standard.md:
// the binary is upgraded by whichever channel owns it. Homebrew installs use
// `brew upgrade`, everything else defers to cargo. `--check` never mutates.

use std::process::Command;

use crate::error::{AppError, Result};
use crate::output::{print_success, Ctx};

const CRATES_IO_URL: &str = "https://crates.io/api/v1/crates/contract-cli";
const BREW_FORMULA: &str = "paperfoot/tap/contract";
const RELEASE_URL: &str = "https://github.com/paperfoot/contract-cli/releases";

#[derive(serde::Serialize)]
struct UpdateReport {
    current_version: String,
    latest_version: Option<String>,
    status: &'static str,
    install_source: &'static str,
    update_mode: &'static str,
    upgrade_command: String,
    release_url: &'static str,
    requires_skill_reinstall: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

pub fn run(ctx: Ctx, check: bool) -> Result<()> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let (install_source, update_mode, upgrade_command) = if running_under_brew() {
        (
            "homebrew",
            "package_manager",
            format!("brew upgrade {BREW_FORMULA}"),
        )
    } else {
        (
            "cargo",
            "package_manager",
            "cargo install --locked --force contract-cli".to_string(),
        )
    };

    if let Ok(cfg) = crate::config::load() {
        if !cfg.self_update {
            let report = UpdateReport {
                current_version: current,
                latest_version: None,
                status: "disabled",
                install_source,
                update_mode: "disabled",
                upgrade_command,
                release_url: RELEASE_URL,
                requires_skill_reinstall: false,
                note: Some("self_update = false in config; upgrade via the package manager".into()),
            };
            print_success(ctx, &report, |r| {
                println!("updates disabled by config. Upgrade manually: {}", r.upgrade_command)
            });
            return Ok(());
        }
    }

    let latest = match fetch_latest_version() {
        Ok(v) => v,
        Err(e) => {
            // --check must stay exit 0 on lookup trouble per the standard's
            // "safe check" contract; report unknown and move on.
            let report = UpdateReport {
                current_version: current,
                latest_version: None,
                status: "up_to_date",
                install_source,
                update_mode,
                upgrade_command,
                release_url: RELEASE_URL,
                requires_skill_reinstall: false,
                note: Some(format!("could not query crates.io: {e}")),
            };
            print_success(ctx, &report, |r| {
                println!(
                    "current: {} — latest unknown ({})",
                    r.current_version,
                    r.note.as_deref().unwrap_or("")
                )
            });
            return Ok(());
        }
    };

    let is_newer = version_newer_than(&latest, &current);
    let status = if is_newer { "update_available" } else { "up_to_date" };

    if check || !is_newer {
        let report = UpdateReport {
            current_version: current,
            latest_version: Some(latest),
            status,
            install_source,
            update_mode,
            upgrade_command,
            release_url: RELEASE_URL,
            requires_skill_reinstall: is_newer,
            note: None,
        };
        print_success(ctx, &report, |r| {
            println!("current: {}", r.current_version);
            println!("latest:  {}", r.latest_version.as_deref().unwrap_or("?"));
            if r.status == "update_available" {
                println!("update available — run: {}", r.upgrade_command);
            } else {
                println!("up to date");
            }
        });
        return Ok(());
    }

    // Apply through the owning channel.
    eprintln!("upgrading {current} → {latest} via: {upgrade_command}");
    let ok = if install_source == "homebrew" {
        run_cmd("brew", &["upgrade", BREW_FORMULA])?
    } else {
        run_cmd(
            "cargo",
            &["install", "--locked", "--force", "contract-cli"],
        )?
    };
    if !ok {
        return Err(AppError::Transient(
            "upgrade command failed — run it manually to see the full output".into(),
        ));
    }
    let report = UpdateReport {
        current_version: current,
        latest_version: Some(latest.clone()),
        status: "updated",
        install_source,
        update_mode,
        upgrade_command,
        release_url: RELEASE_URL,
        requires_skill_reinstall: true,
        note: Some("run `contract skill install` to refresh the installed skill".into()),
    };
    print_success(ctx, &report, |_| {
        println!("upgraded to {latest}. Refresh the skill: contract skill install");
    });
    Ok(())
}

fn run_cmd(program: &str, args: &[&str]) -> Result<bool> {
    let status = Command::new(program)
        .args(args)
        .env("HOMEBREW_NO_AUTO_UPDATE", "1")
        .status()
        .map_err(|e| AppError::Transient(format!("failed to launch {program}: {e}")))?;
    Ok(status.success())
}

fn fetch_latest_version() -> Result<String> {
    let out = Command::new("curl")
        .args([
            "-sSL",
            "--max-time",
            "10",
            "-H",
            "User-Agent: contract-cli",
            "-H",
            "Accept: application/json",
            CRATES_IO_URL,
        ])
        .output()
        .map_err(|e| AppError::Transient(format!("curl not available: {e}")))?;
    if !out.status.success() {
        return Err(AppError::Transient(format!(
            "crates.io query failed (exit {})",
            out.status.code().unwrap_or(-1)
        )));
    }
    let body: serde_json::Value = serde_json::from_slice(&out.stdout)
        .map_err(|e| AppError::Transient(format!("bad crates.io response: {e}")))?;
    if let Some(errors) = body.get("errors").and_then(|e| e.as_array()) {
        let detail = errors
            .first()
            .and_then(|e| e.get("detail"))
            .and_then(|d| d.as_str())
            .unwrap_or("unknown");
        return Err(AppError::Transient(format!("crates.io: {detail}")));
    }
    body.get("crate")
        .and_then(|c| c.get("max_stable_version"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Transient("crates.io response missing max_stable_version".into()))
}

fn version_newer_than(a: &str, b: &str) -> bool {
    match (parse_version(a), parse_version(b)) {
        (Some(a), Some(b)) => a > b,
        _ => false,
    }
}

fn parse_version(v: &str) -> Option<(u32, u32, u32)> {
    let core = v.split(['-', '+']).next()?;
    let mut parts = core.split('.');
    Some((
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next().unwrap_or("0").parse().unwrap_or(0),
    ))
}

fn running_under_brew() -> bool {
    std::env::current_exe()
        .map(|p| {
            let s = p.to_string_lossy().to_string();
            s.contains("/homebrew/") || s.contains("/Cellar/") || s.contains("/opt/homebrew/")
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_semver_versions() {
        assert!(version_newer_than("0.2.0", "0.1.0"));
        assert!(!version_newer_than("0.1.0", "0.1.0"));
        assert!(!version_newer_than("0.1.0", "0.2.0"));
    }
}
