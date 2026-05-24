use std::io::IsTerminal;

use crate::error::AppError;

#[derive(Clone, Copy, Debug)]
pub enum Format {
    Json,
    Human,
}

impl Format {
    pub fn detect(json_flag: bool) -> Self {
        if json_flag || !std::io::stdout().is_terminal() {
            Format::Json
        } else {
            Format::Human
        }
    }
}

#[derive(Clone, Copy)]
pub struct Ctx {
    pub format: Format,
    pub quiet: bool,
}

impl Ctx {
    pub fn new(json_flag: bool, quiet: bool) -> Self {
        Self {
            format: Format::detect(json_flag),
            quiet,
        }
    }
}

pub fn print_success<T, F>(ctx: Ctx, data: &T, human: F)
where
    T: serde::Serialize,
    F: FnOnce(&T),
{
    match ctx.format {
        Format::Json => {
            let envelope = serde_json::json!({
                "version": "1",
                "status": "success",
                "data": data,
            });
            println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
        }
        Format::Human if !ctx.quiet => human(data),
        Format::Human => {}
    }
}

pub fn print_error(format: Format, err: &AppError) {
    match format {
        Format::Json => {
            let envelope = serde_json::json!({
                "version": "1",
                "status": "error",
                "error": {
                    "code": err.error_code(),
                    "message": err.to_string(),
                    "suggestion": err.suggestion(),
                },
            });
            eprintln!("{}", serde_json::to_string_pretty(&envelope).unwrap());
        }
        Format::Human => {
            eprintln!("error: {}", err);
            let hint = err.suggestion();
            if !hint.is_empty() {
                eprintln!("  hint: {}", hint);
            }
        }
    }
}

pub fn print_raw<T: serde::Serialize>(value: &T) {
    println!("{}", serde_json::to_string_pretty(value).unwrap());
}
