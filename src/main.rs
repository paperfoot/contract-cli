use std::process::exit;

use clap::Parser;

use contract_cli::{cli, commands, error, output};

fn has_json_flag() -> bool {
    std::env::args_os().any(|a| a == "--json")
}

fn main() {
    let json_flag = has_json_flag();

    let cli = match cli::Cli::try_parse_from(std::env::args_os()) {
        Ok(cli) => cli,
        Err(e) => {
            // Help and --version are informational requests, not errors: exit 0.
            // When piped, wrap the text in the success envelope so `| jq` parses.
            if matches!(
                e.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) {
                match output::Format::detect(json_flag) {
                    output::Format::Json => {
                        output::print_help_envelope(&e.render().to_string());
                    }
                    output::Format::Human => {
                        let _ = e.print();
                    }
                }
                exit(0);
            }
            // Parse errors: we own the exit code, not clap. Always 3.
            let fmt = output::Format::detect(json_flag);
            output::print_error(fmt, &error::AppError::InvalidInput(e.to_string()));
            exit(3);
        }
    };

    let ctx = output::Ctx::new(cli.json, cli.quiet);
    if let Err(err) = commands::dispatch(cli, ctx) {
        output::print_error(ctx.format, &err);
        exit(err.exit_code());
    }
}
