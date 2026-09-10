use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

struct TestHome {
    root: TempDir,
}

impl TestHome {
    fn new() -> Self {
        Self {
            root: tempfile::Builder::new()
                .prefix("contract-cli-reliability-")
                .tempdir()
                .expect("create isolated test home"),
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_contract"));
        command
            .env("HOME", self.root.path())
            .env("XDG_CONFIG_HOME", self.root.path().join(".config"))
            .env("XDG_DATA_HOME", self.root.path().join(".local/share"))
            .env("XDG_CACHE_HOME", self.root.path().join(".cache"))
            .arg("--json");
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command()
            .args(args)
            .output()
            .unwrap_or_else(|error| panic!("run contract {args:?}: {error}"))
    }

    fn success(&self, args: &[&str]) -> Value {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "contract {args:?} failed\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let envelope: Value =
            serde_json::from_slice(&output.stdout).expect("success output is JSON");
        assert_eq!(envelope["status"], "success", "command: {args:?}");
        envelope["data"].clone()
    }

    fn rejects(&self, args: &[&str]) {
        let output = self.run(args);
        assert_eq!(
            output.status.code(),
            Some(3),
            "contract {args:?} should reject invalid input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let envelope: Value = serde_json::from_slice(&output.stderr).expect("error output is JSON");
        assert_eq!(envelope["status"], "error", "command: {args:?}");
    }

    fn add_parties(&self) {
        self.success(&[
            "issuer",
            "add",
            "acme",
            "--name",
            "Acme Example Ltd",
            "--legal-name",
            "Acme Example Limited",
            "--jurisdiction",
            "uk",
            "--address",
            "1 Example Street\nLondon\nEX1 1AA",
        ]);
        self.success(&[
            "clients",
            "add",
            "meridian",
            "--name",
            "Meridian Example Ltd",
            "--legal-name",
            "Meridian Example Limited",
            "--jurisdiction",
            "England and Wales",
            "--address",
            "2 Example Avenue\nLondon\nEX2 2BB",
        ]);
    }

    fn new_nda(&self, extra: &[&str]) -> Value {
        let mut args = vec![
            "new",
            "--kind",
            "nda",
            "--as",
            "acme",
            "--client",
            "meridian",
            "--purpose",
            "Evaluate a software partnership",
        ];
        args.extend_from_slice(extra);
        self.success(&args)
    }

    fn reject_new_nda(&self, extra: &[&str]) {
        let mut args = vec![
            "new",
            "--kind",
            "nda",
            "--as",
            "acme",
            "--client",
            "meridian",
            "--purpose",
            "Evaluate a software partnership",
        ];
        args.extend_from_slice(extra);
        self.rejects(&args);
    }
}

#[test]
fn all_seven_contract_kinds_can_be_created() {
    let home = TestHome::new();
    home.add_parties();

    for kind in [
        "nda",
        "ncnda",
        "consulting",
        "msa",
        "sow",
        "service",
        "loan",
    ] {
        let contract = home.success(&[
            "new",
            "--kind",
            kind,
            "--as",
            "acme",
            "--client",
            "meridian",
            "--purpose",
            "Evaluate a software partnership",
            "--legal-profile",
            "uk",
        ]);
        assert_eq!(contract["kind"], kind);
        assert!(contract["number"].as_str().is_some());
        assert_eq!(contract["status"], "draft");
    }
}

#[test]
fn new_rejects_reversed_dates_and_term_year_overflow() {
    let home = TestHome::new();
    home.add_parties();

    home.reject_new_nda(&["--effective", "2026-06-01", "--end", "2026-05-31"]);

    let too_many_years = i64::MAX.to_string();
    home.reject_new_nda(&["--term-years", &too_many_years]);
}

#[test]
fn new_rejects_invalid_confidentiality_and_unilateral_disclosure_terms() {
    let home = TestHome::new();
    home.add_parties();

    home.reject_new_nda(&["--term", "confidentiality_years=-1"]);
    home.reject_new_nda(&["--mutuality", "unilateral", "--disclosing-side", "both"]);
}

#[test]
fn new_rejects_template_path_traversal() {
    let home = TestHome::new();
    home.add_parties();

    home.reject_new_nda(&["--template", "../shared/contract"]);
}

#[test]
fn legal_profiles_require_their_mandatory_arguments() {
    let home = TestHome::new();
    home.add_parties();

    home.reject_new_nda(&["--legal-profile", "us"]);
    home.reject_new_nda(&[
        "--legal-profile",
        "global",
        "--venue",
        "Courts of New South Wales, Australia",
    ]);
    home.reject_new_nda(&[
        "--legal-profile",
        "global",
        "--governing-law",
        "New South Wales, Australia",
    ]);
}

#[test]
fn legal_profiles_select_expected_law_and_accept_global_venue() {
    let home = TestHome::new();
    home.add_parties();

    let uk = home.new_nda(&["--legal-profile", "uk"]);
    assert_eq!(uk["governing_law"], "England and Wales");

    let singapore = home.new_nda(&["--legal-profile", "singapore"]);
    assert_eq!(singapore["governing_law"], "Singapore");

    let us = home.new_nda(&["--legal-profile", "us", "--us-state", "Delaware"]);
    assert_eq!(us["governing_law"], "Delaware");

    let global = home.new_nda(&[
        "--legal-profile",
        "global",
        "--governing-law",
        "New South Wales, Australia",
        "--venue",
        "Courts of New South Wales, Australia",
    ]);
    assert_eq!(global["governing_law"], "New South Wales, Australia");
    assert_eq!(global["venue"], "Courts of New South Wales, Australia");
}

#[test]
fn sign_rejects_blank_signer_name() {
    let home = TestHome::new();
    home.add_parties();
    let contract = home.new_nda(&[]);
    let number = contract["number"].as_str().expect("contract number");

    home.rejects(&["sign", number, "--side", "us", "--name", "   "]);
}

#[test]
fn first_signature_locks_contract_edits_clause_edits_and_recall() {
    let home = TestHome::new();
    home.add_parties();
    let contract = home.new_nda(&[]);
    let number = contract["number"].as_str().expect("contract number");

    home.success(&["sign", number, "--side", "us", "--name", "Alex Example"]);
    home.rejects(&["edit", number, "--title", "Changed after signing"]);
    home.rejects(&[
        "contracts",
        "clauses",
        "edit",
        number,
        "purpose",
        "--body",
        "Changed after signing.",
    ]);
    home.success(&["mark", number, "sent"]);
    home.rejects(&["mark", number, "draft"]);
}

#[test]
fn terminated_contract_cannot_be_signed_even_with_force() {
    let home = TestHome::new();
    home.add_parties();
    let contract = home.new_nda(&[]);
    let number = contract["number"].as_str().expect("contract number");

    home.success(&["sign", number, "--side", "us", "--name", "Alex Example"]);
    home.success(&["sign", number, "--side", "them", "--name", "Morgan Example"]);
    home.success(&["mark", number, "terminated"]);
    home.rejects(&[
        "sign",
        number,
        "--side",
        "us",
        "--name",
        "Replacement Signer",
        "--force",
    ]);

    let shown = home.success(&["show", number]);
    assert_eq!(shown["status"], "terminated");
    assert_eq!(shown["signed_by_us_name"], "Alex Example");
}
