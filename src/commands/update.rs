use crate::error::Result;
use crate::output::{print_success, Ctx};

#[derive(serde::Serialize)]
struct Out {
    current: String,
    note: String,
}

pub fn run(ctx: Ctx, _check: bool) -> Result<()> {
    // Placeholder. Real updater (brew tap / cargo install --force) lives in the
    // release pipeline; this command just reports the current version so the
    // CLI surface matches the rest of the suite.
    let out = Out {
        current: env!("CARGO_PKG_VERSION").to_string(),
        note: "Self-update will run via Homebrew (brew upgrade contract-cli) or cargo install --force contract-cli once published.".into(),
    };
    print_success(ctx, &out, |o| {
        println!("contract-cli v{} — {}", o.current, o.note);
    });
    Ok(())
}
