// Framework-contract tests: exit codes, envelope discipline, agent-info
// routability, help behavior. Every invocation runs against an isolated
// HOME (fresh config/DB per process) so tests never touch the user's real
// shared accounting DB and parallel test processes can't contend on one
// SQLite file ("database is locked" flakes in CI).

use assert_cmd::Command;

fn contract() -> Command {
    let mut cmd = Command::cargo_bin("contract").expect("binary builds");
    let home = tempfile::Builder::new()
        .prefix("contract-cli-test-home-")
        .tempdir()
        .expect("temp home");
    cmd.env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join(".config"))
        .env("XDG_DATA_HOME", home.path().join(".local/share"))
        .env("XDG_CACHE_HOME", home.path().join(".cache"));
    // Leak the tempdir so it outlives the child process; the OS cleans /tmp.
    std::mem::forget(home);
    cmd
}

// ─── exit-code contract (via the hidden hook) ────────────────────────────

#[test]
fn exit_hook_covers_all_codes() {
    for code in 0..=4 {
        contract()
            .args(["contract", &code.to_string()])
            .assert()
            .code(code);
    }
}

#[test]
fn exit_hook_rejects_unknown_code() {
    contract().args(["contract", "9"]).assert().code(3);
}

// ─── help / version discipline ───────────────────────────────────────────

#[test]
fn help_exits_zero_and_teaches_usage() {
    let out = contract().arg("--help").assert().success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).to_string();
    // Piped (non-TTY) help is wrapped in the success envelope.
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("piped help is JSON");
    assert_eq!(v["status"], "success");
    let usage = v["data"]["usage"].as_str().expect("data.usage");
    assert!(usage.contains("Tips:"), "help teaches Tips");
    assert!(usage.contains("Examples:"), "help teaches Examples");
}

#[test]
fn version_exits_zero_enveloped() {
    let out = contract().arg("--version").assert().success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).to_string();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("piped version is JSON");
    assert_eq!(v["status"], "success");
}

#[test]
fn unknown_command_exits_3_stderr_envelope_stdout_empty() {
    let out = contract().arg("definitely-not-a-command").assert().code(3);
    let output = out.get_output();
    assert!(output.stdout.is_empty(), "nothing on stdout");
    let v: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr is a JSON envelope");
    assert_eq!(v["status"], "error");
    assert!(
        !v["error"]["suggestion"].as_str().unwrap_or("").is_empty(),
        "error carries a suggestion"
    );
}

// ─── agent-info contract ──────────────────────────────────────────────────

fn manifest() -> serde_json::Value {
    let out = contract().arg("agent-info").assert().success();
    serde_json::from_slice(&out.get_output().stdout).expect("agent-info emits raw JSON")
}

#[test]
fn agent_info_has_required_schema_keys() {
    let m = manifest();
    for key in [
        "name",
        "version",
        "description",
        "commands",
        "global_flags",
        "exit_codes",
        "envelope",
        "config",
        "auto_json_when_piped",
    ] {
        assert!(m.get(key).is_some(), "manifest missing `{key}`");
    }
    assert_eq!(m["auto_json_when_piped"], true);
    for code in ["0", "1", "2", "3", "4"] {
        assert!(
            m["exit_codes"].get(code).is_some(),
            "exit code {code} documented"
        );
    }
    assert!(m["global_flags"].get("--json").is_some());
    assert!(m["global_flags"].get("--quiet").is_some());
    assert!(m["config"]["path"].as_str().is_some());
    assert!(m["config"]["env_prefix"].as_str().is_some());
}

#[test]
fn agent_info_commands_are_canonical_objects_and_routable() {
    let m = manifest();
    let commands = m["commands"].as_object().expect("commands object");
    assert!(!commands.is_empty());
    for (key, value) in commands {
        let obj = value
            .as_object()
            .unwrap_or_else(|| panic!("`{key}` is an object"));
        assert!(obj.contains_key("description"), "`{key}` has description");
        assert!(obj.contains_key("args"), "`{key}` has args");
        assert!(obj.contains_key("options"), "`{key}` has options");
        // Every advertised command must be routable: `contract <key> --help` exits 0.
        let mut cmd = contract();
        for part in key.split_whitespace() {
            cmd.arg(part);
        }
        cmd.arg("--help").assert().success();
    }
}

// ─── discovery ────────────────────────────────────────────────────────────

#[test]
fn template_list_returns_metadata() {
    let out = contract().args(["template", "list"]).assert().success();
    let v: serde_json::Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["status"], "success");
    let templates = v["data"].as_array().expect("template array");
    assert!(!templates.is_empty());
    for t in templates {
        assert!(t["name"].as_str().is_some());
        assert!(t["description"].as_str().is_some());
    }
}

#[test]
fn kinds_find_resolves_non_circumvention_language() {
    let out = contract()
        .args([
            "kinds",
            "find",
            "stop them going around me to steal my contact",
        ])
        .assert()
        .success();
    let v: serde_json::Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["data"][0]["kind"], "ncnda");
}

#[test]
fn kinds_find_gibberish_exits_3() {
    contract()
        .args(["kinds", "find", "xyzzyplughfrobozz"])
        .assert()
        .code(3);
}

#[test]
fn template_find_ranks_by_description() {
    let out = contract()
        .args(["template", "find", "swiss minimal monochrome"])
        .assert()
        .success();
    let v: serde_json::Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let first = v["data"][0]["name"].as_str().unwrap();
    assert!(
        first == "helvetica-nera" || first == "basel",
        "swiss query resolves to a swiss template, got {first}"
    );
}

// ─── input validation (all fail before any DB write) ─────────────────────

#[test]
fn new_rejects_unknown_kind() {
    contract()
        .args(["new", "--kind", "franchise", "--client", "nobody"])
        .assert()
        .code(3);
}

#[test]
fn new_rejects_bad_fee_specs() {
    for fee in [
        "fixed:nan:SGD",
        "fixed:0:SGD",
        "fixed:-5:SGD",
        "retainer:5000:SGD/month/x:y",
    ] {
        contract()
            .args([
                "new",
                "--kind",
                "consulting",
                "--client",
                "nobody",
                "--fee",
                fee,
            ])
            .assert()
            .code(3);
    }
}

#[test]
fn new_rejects_bad_enum_values() {
    contract()
        .args([
            "new",
            "--kind",
            "nda",
            "--client",
            "nobody",
            "--mutuality",
            "Mutual-ish",
        ])
        .assert()
        .code(3);
    contract()
        .args([
            "new",
            "--kind",
            "nda",
            "--client",
            "nobody",
            "--term-months",
            "0",
        ])
        .assert()
        .code(3);
}

#[test]
fn quiet_suppresses_human_not_json() {
    // --json --quiet: JSON must still emit.
    let out = contract()
        .args(["kinds", "list", "--json", "--quiet"])
        .assert()
        .success();
    let v: serde_json::Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["status"], "success");
}
