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
                .prefix("contract-cli-draft-edits-")
                .tempdir()
                .expect("create isolated test home"),
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_contract"));
        command
            .env_clear()
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

    fn rejects(&self, args: &[&str]) -> Value {
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
        envelope["error"].clone()
    }

    fn add_fictional_parties(&self) {
        self.success(&[
            "issuer",
            "add",
            "fictionalholdings",
            "--name",
            "Fictional Holdings Pte Ltd",
            "--legal-name",
            "Fictional Holdings Pte. Ltd.",
            "--jurisdiction",
            "sg",
            "--address",
            "1 Fictional Street\nSingapore 000001",
        ]);
        for (slug, name, address) in [
            (
                "fictionalclient",
                "Fictional Client Ltd",
                "2 Fictional Street\nSingapore 000002",
            ),
            (
                "newfictionalclient",
                "New Fictional Client Ltd",
                "3 Fictional Street\nSingapore 000003",
            ),
        ] {
            self.success(&[
                "clients",
                "add",
                slug,
                "--name",
                name,
                "--legal-name",
                name,
                "--jurisdiction",
                "Singapore",
                "--address",
                address,
            ]);
        }
    }

    fn new_singapore_contract(&self, kind: &str) -> Value {
        self.success(&[
            "new",
            "--kind",
            kind,
            "--as",
            "fictionalholdings",
            "--client",
            "fictionalclient",
            "--purpose",
            "Evaluate a fictional commercial relationship",
            "--effective",
            "2026-09-10",
            "--legal-profile",
            "singapore",
        ])
    }
}

fn terms(contract: &Value) -> Value {
    serde_json::from_str(
        contract["terms_json"]
            .as_str()
            .expect("contract terms_json string"),
    )
    .expect("valid contract terms_json")
}

#[test]
fn singapore_profile_rejects_conflicting_draft_law_without_mutation() {
    let home = TestHome::new();
    home.add_fictional_parties();
    let created = home.new_singapore_contract("nda");
    let number = created["number"].as_str().expect("contract number");

    home.rejects(&["edit", number, "--governing-law", "Delaware"]);

    let unchanged = home.success(&["show", number]);
    assert_eq!(unchanged["governing_law"], "Singapore");
    assert_eq!(terms(&unchanged)["legal_profile"], "singapore");
}

#[test]
fn ncnda_draft_rejects_unilateral_terms_without_mutation() {
    let home = TestHome::new();
    home.add_fictional_parties();
    let created = home.new_singapore_contract("ncnda");
    let number = created["number"].as_str().expect("contract number");

    home.rejects(&[
        "edit",
        number,
        "--term",
        "mutuality=unilateral",
        "--term",
        "disclosing_side=us",
    ]);

    let unchanged = home.success(&["show", number]);
    let unchanged_terms = terms(&unchanged);
    assert_eq!(unchanged_terms["mutuality"], "mutual");
    assert_eq!(unchanged_terms["disclosing_side"], "both");
}

#[test]
fn legal_profile_is_reserved_from_free_form_draft_terms() {
    let home = TestHome::new();
    home.add_fictional_parties();
    let created = home.new_singapore_contract("nda");
    let number = created["number"].as_str().expect("contract number");

    let error = home.rejects(&["edit", number, "--term", "legal_profile=us"]);
    assert!(
        error.to_string().contains("use --legal-profile"),
        "reserved term rejection points to the supported flag: {error}"
    );

    let unchanged = home.success(&["show", number]);
    assert_eq!(terms(&unchanged)["legal_profile"], "singapore");
}

#[test]
fn config_rejects_unknown_template_and_preserves_valid_folio_setting() {
    let home = TestHome::new();

    home.success(&["config", "set", "default_template", "folio"]);
    home.rejects(&[
        "config",
        "set",
        "default_template",
        "bogus-fictional-template",
    ]);

    let config = home.success(&["config", "show"]);
    assert_eq!(config["default_template"], "folio");
}

#[test]
fn duplicate_for_new_client_recomputes_auto_title_and_clears_signatures() {
    let home = TestHome::new();
    home.add_fictional_parties();
    let source = home.new_singapore_contract("nda");
    let source_number = source["number"].as_str().expect("source contract number");

    home.success(&[
        "sign",
        source_number,
        "--side",
        "us",
        "--name",
        "Fictional Signer One",
    ]);
    home.success(&[
        "sign",
        source_number,
        "--side",
        "them",
        "--name",
        "Fictional Signer Two",
    ]);

    let duplicated = home.success(&["duplicate", source_number, "--client", "newfictionalclient"]);
    assert_eq!(
        duplicated["title"],
        "NDA — Fictional Holdings Pte Ltd & New Fictional Client Ltd"
    );
    assert_eq!(duplicated["governing_law"], "Singapore");
    assert_eq!(terms(&duplicated)["legal_profile"], "singapore");
    assert_eq!(duplicated["status"], "draft");
    for field in [
        "signed_at",
        "signed_by_us_name",
        "signed_by_us_title",
        "signed_by_us_at",
        "signed_by_them_name",
        "signed_by_them_title",
        "signed_by_them_at",
    ] {
        assert!(duplicated[field].is_null(), "duplicate retains {field}");
    }
}
