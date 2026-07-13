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

/// Serialize without ever panicking or emitting invalid JSON. A payload that
/// fails to serialize degrades to an error envelope rather than a crash.
fn safe_json_string<T: serde::Serialize>(value: &T) -> String {
    match serde_json::to_string_pretty(value) {
        Ok(s) => s,
        Err(e) => {
            let fallback = serde_json::json!({
                "version": "1",
                "status": "error",
                "error": {
                    "code": "serialize",
                    "message": e.to_string(),
                    "suggestion": "Retry the command",
                },
            });
            serde_json::to_string_pretty(&fallback).unwrap_or_else(|_| {
                r#"{"version":"1","status":"error","error":{"code":"serialize","message":"serialization failed","suggestion":"Retry the command"}}"#.to_string()
            })
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
            println!("{}", safe_json_string(&envelope));
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
            eprintln!("{}", safe_json_string(&envelope));
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
    println!("{}", safe_json_string(value));
}

/// Help / version text requested while piped: wrap in the success envelope so
/// `contract --help | jq` parses. Informational requests always exit 0.
pub fn print_help_envelope(text: &str) {
    let envelope = serde_json::json!({
        "version": "1",
        "status": "success",
        "data": { "usage": text },
    });
    println!("{}", safe_json_string(&envelope));
}
