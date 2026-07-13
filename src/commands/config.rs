use crate::cli::ConfigCmd;
use crate::config;
use crate::error::{AppError, Result};
use crate::output::{print_success, Ctx};

fn parse_bool(key: &str, value: &str) -> Result<bool> {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(AppError::InvalidInput(format!(
            "invalid value '{value}' for {key} — expected true or false"
        ))),
    }
}

pub fn run(cmd: ConfigCmd, ctx: Ctx) -> Result<()> {
    match cmd {
        ConfigCmd::Show => {
            let cfg = config::load()?;
            print_success(ctx, &cfg, |c| {
                match toml::to_string_pretty(c) {
                    Ok(t) => print!("{t}"),
                    Err(_) => println!("{c:?}"),
                }
            });
            Ok(())
        }
        ConfigCmd::Path => {
            let p = config::config_path()?.display().to_string();
            print_success(ctx, &p, |p| println!("{p}"));
            Ok(())
        }
        ConfigCmd::Set { key, value } => {
            let paths = finance_core::paths::Paths::resolve()?;
            let mut cfg = config::load()?;
            match key.as_str() {
                "default_issuer" => {
                    cfg.default_issuer = if value == "unset" { None } else { Some(value.clone()) };
                }
                "default_template" => {
                    cfg.default_template = value.clone();
                }
                "open_pdf" => {
                    cfg.open_pdf = parse_bool(&key, &value)?;
                }
                "self_update" => {
                    cfg.self_update = parse_bool(&key, &value)?;
                }
                _ => {
                    return Err(AppError::InvalidInput(format!(
                        "unknown config key '{key}' (try: default_issuer, default_template, open_pdf, self_update)"
                    )));
                }
            }
            cfg.save(&paths)?;
            print_success(ctx, &cfg, |_| println!("config saved"));
            Ok(())
        }
    }
}
