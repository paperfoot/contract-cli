use crate::cli::{Cli, Commands, ContractCmd};
use crate::error::{AppError, Result};
use crate::output::Ctx;

pub mod agent_info;
pub mod clauses;
pub mod clients;
pub mod config;
pub mod contracts;
pub mod doctor;
pub mod issuers;
pub mod kinds;
pub mod pack;
pub mod skill;
pub mod template;
pub mod update;

pub fn dispatch(cli: Cli, ctx: Ctx) -> Result<()> {
    crate::config::ensure_dirs()?;

    match cli.command {
        Commands::Issuer(cmd) => issuers::run(cmd, ctx),
        Commands::Clients(cmd) => clients::run(cmd, ctx),
        Commands::Contracts(cmd) => contracts::run(cmd, ctx),
        // Top-level shortcuts (delegate to the contracts handler)
        Commands::New(args) => contracts::run(ContractCmd::New(args), ctx),
        Commands::List(args) => contracts::run(ContractCmd::List(args), ctx),
        Commands::Show { number } => contracts::run(ContractCmd::Show { number }, ctx),
        Commands::Render(args) => contracts::run(ContractCmd::Render(args), ctx),
        Commands::Mark { number, status } => {
            contracts::run(ContractCmd::Mark { number, status }, ctx)
        }
        Commands::Sign(args) => contracts::run(ContractCmd::Sign(args), ctx),
        Commands::Edit(args) => contracts::run(ContractCmd::Edit(args), ctx),
        Commands::Duplicate(args) => contracts::run(ContractCmd::Duplicate(args), ctx),
        Commands::Delete(args) => contracts::run(ContractCmd::Delete(args), ctx),
        Commands::Pack(cmd) => pack::run(cmd, ctx),
        Commands::Template(cmd) => template::run(cmd, ctx),
        Commands::Kinds(cmd) => kinds::run(cmd, ctx),
        Commands::Config(cmd) => config::run(cmd, ctx),
        Commands::AgentInfo => agent_info::run(ctx),
        Commands::Skill(cmd) => skill::run(cmd, ctx),
        Commands::Doctor => doctor::run(ctx),
        Commands::Update { check } => update::run(ctx, check),
        Commands::ExitHook { code } => exit_hook(code, ctx),
    }
}

/// Hidden conformance hook: deterministically trigger each exit code so the
/// test suite and conformance.sh can verify the full 0-4 contract.
fn exit_hook(code: i32, ctx: Ctx) -> Result<()> {
    match code {
        0 => {
            let data = serde_json::json!({ "contract": true, "exit_code": 0 });
            crate::output::print_success(ctx, &data, |_| println!("contract: success"));
            Ok(())
        }
        1 => Err(AppError::Transient("contract: transient error".into())),
        2 => Err(AppError::Config("contract: config error".into())),
        3 => Err(AppError::InvalidInput("contract: bad input".into())),
        4 => Err(AppError::RateLimited("contract: rate limited".into())),
        other => Err(AppError::InvalidInput(format!(
            "unknown contract code: {other}. Valid: 0-4"
        ))),
    }
}

pub(crate) fn split_multiline_arg(value: &str) -> Vec<String> {
    let normalized = value.replace("\\n", "\n");
    normalized.split('\n').map(|s| s.to_string()).collect()
}
