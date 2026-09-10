use clap::Parser;
use contract_cli::cli::{Cli, Commands, PAPER_SIZES, TemplateCmd};

fn render_args(extra: &[&str]) -> contract_cli::cli::ContractRenderArgs {
    let mut argv = vec!["contract", "render", "NDA-acme-2026-0001"];
    argv.extend_from_slice(extra);
    let cli = Cli::try_parse_from(argv).expect("render options should parse");
    match cli.command {
        Commands::Render(args) => args,
        other => panic!("expected render command, got {other:?}"),
    }
}

fn preview_options(extra: &[&str]) -> (String, String) {
    let mut argv = vec!["contract", "template", "preview", "folio"];
    argv.extend_from_slice(extra);
    let cli = Cli::try_parse_from(argv).expect("template preview options should parse");
    match cli.command {
        Commands::Template(TemplateCmd::Preview {
            paper, reference, ..
        }) => (paper, reference),
        other => panic!("expected template preview command, got {other:?}"),
    }
}

#[test]
fn render_and_preview_default_to_a4_with_reference_visible() {
    let render = render_args(&[]);
    assert_eq!(render.paper, "a4");
    assert_eq!(render.reference, "on");

    let (paper, reference) = preview_options(&[]);
    assert_eq!(paper, "a4");
    assert_eq!(reference, "on");
}

#[test]
fn render_and_preview_accept_all_supported_paper_sizes() {
    for paper in PAPER_SIZES {
        assert_eq!(render_args(&["--paper", paper]).paper, paper);
        assert_eq!(preview_options(&["--paper", paper]).0, paper);
    }
}

#[test]
fn render_and_preview_accept_reference_on_and_off() {
    for reference in ["on", "off"] {
        assert_eq!(
            render_args(&["--reference", reference]).reference,
            reference
        );
        assert_eq!(preview_options(&["--reference", reference]).1, reference);
    }
}

#[test]
fn render_and_preview_reject_invalid_paper_and_reference_values() {
    assert!(
        Cli::try_parse_from(["contract", "render", "NDA-acme-2026-0001", "--paper", "b5",])
            .is_err()
    );
    assert!(
        Cli::try_parse_from(["contract", "template", "preview", "folio", "--paper", "b5",])
            .is_err()
    );
    assert!(
        Cli::try_parse_from([
            "contract",
            "render",
            "NDA-acme-2026-0001",
            "--reference",
            "yes",
        ])
        .is_err()
    );
    assert!(
        Cli::try_parse_from([
            "contract",
            "template",
            "preview",
            "folio",
            "--reference",
            "yes",
        ])
        .is_err()
    );
}
