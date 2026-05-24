use crate::cli::ConfigCmd;
use crate::config;
use crate::error::Result;
use crate::output::{print_raw, print_success, Ctx};

pub fn run(cmd: ConfigCmd, ctx: Ctx) -> Result<()> {
    match cmd {
        ConfigCmd::Show => {
            let cfg = config::load()?;
            print_raw(&cfg);
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
                    cfg.open_pdf = value.parse().unwrap_or(true);
                }
                "self_update" => {
                    cfg.self_update = value.parse().unwrap_or(true);
                }
                _ => {
                    return Err(crate::error::AppError::InvalidInput(format!(
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
