use crate::cli::{Cli, Commands, ContractCmd};
use crate::error::Result;
use crate::output::Ctx;

pub mod agent_info;
pub mod clauses;
pub mod clients;
pub mod config;
pub mod contracts;
pub mod doctor;
pub mod issuers;
pub mod pack;
pub mod skill;
pub mod template;
pub mod update;

pub fn dispatch(cli: Cli, ctx: Ctx) -> Result<()> {
    crate::config::ensure_dirs()?;
    crate::typst_assets::ensure_extracted()?;

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
        Commands::Pack(cmd) => pack::run(cmd, ctx),
        Commands::Template(cmd) => template::run(cmd, ctx),
        Commands::Config(cmd) => config::run(cmd, ctx),
        Commands::AgentInfo => agent_info::run(ctx),
        Commands::Skill(cmd) => skill::run(cmd, ctx),
        Commands::Doctor => doctor::run(ctx),
        Commands::Update { check } => update::run(ctx, check),
    }
}

pub(crate) fn split_multiline_arg(value: &str) -> Vec<String> {
    let normalized = value.replace("\\n", "\n");
    normalized.split('\n').map(|s| s.to_string()).collect()
}
