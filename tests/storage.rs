use contract_cli::{clauses, db};
use rusqlite::{Connection, params};

fn object_exists(conn: &Connection, object_type: &str, name: &str) -> bool {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = ?1 AND name = ?2)",
        params![object_type, name],
        |row| row.get(0),
    )
    .expect("inspect sqlite schema")
}

fn assert_database_health(conn: &Connection) {
    let foreign_keys: i64 = conn
        .pragma_query_value(None, "foreign_keys", |row| row.get(0))
        .expect("read foreign_keys pragma");
    assert_eq!(foreign_keys, 1, "foreign key enforcement remains enabled");

    let broken_references: i64 = conn
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .expect("run foreign_key_check");
    assert_eq!(
        broken_references, 0,
        "migration leaves no broken references"
    );
}

fn insert_minimal_contract(
    conn: &Connection,
    number: &str,
    kind: &str,
    issuer_id: i64,
    client_id: i64,
) -> i64 {
    conn.execute(
        "INSERT INTO contracts (
             number, kind, issuer_id, client_id, title, effective_date,
             governing_law, clause_pack, clause_pack_version
         ) VALUES (?1, ?2, ?3, ?4, ?5, '2026-09-10', 'England and Wales',
                   'standard', '2.0')",
        params![
            number,
            kind,
            issuer_id,
            client_id,
            format!("{kind} contract")
        ],
    )
    .unwrap_or_else(|error| panic!("insert {kind} contract: {error}"));
    conn.last_insert_rowid()
}

#[test]
fn upgrades_populated_v7_database_without_losing_data_or_schema_objects() {
    let database = tempfile::NamedTempFile::new().expect("create temporary database file");
    let database_path = database.path().to_path_buf();

    let legacy =
        finance_core::db::open_at(&database_path).expect("create finance-core V7 database");
    legacy
        .execute(
            "INSERT INTO issuers (slug, name, jurisdiction, address)
             VALUES ('legacy-issuer', 'Legacy Issuer Ltd', 'uk', '1 Old Street')",
            [],
        )
        .expect("insert minimum valid issuer");
    let issuer_id = legacy.last_insert_rowid();
    legacy
        .execute(
            "INSERT INTO clients (slug, name, address)
             VALUES ('legacy-client', 'Legacy Client Ltd', '2 Old Street')",
            [],
        )
        .expect("insert minimum valid client");
    let client_id = legacy.last_insert_rowid();
    let contract_id =
        insert_minimal_contract(&legacy, "LEGACY-2026-0001", "nda", issuer_id, client_id);
    legacy
        .execute(
            "INSERT INTO contract_clauses (contract_id, position, slug, heading, body)
             VALUES (?1, 0, 'purpose', 'Legacy purpose', 'Preserve this wording.')",
            [contract_id],
        )
        .expect("insert legacy clause");
    legacy
        .execute_batch(
            "CREATE TABLE custom_contract_audit (
                 id INTEGER PRIMARY KEY,
                 contract_id INTEGER NOT NULL,
                 contract_kind TEXT NOT NULL
             );
             CREATE INDEX custom_contract_title_idx ON contracts(title);
             CREATE TRIGGER custom_contract_insert_audit
             AFTER INSERT ON contracts
             BEGIN
                 INSERT INTO custom_contract_audit (contract_id, contract_kind)
                 VALUES (NEW.id, NEW.kind);
             END;",
        )
        .expect("create custom contract index and trigger");
    legacy
        .execute(
            "UPDATE sqlite_sequence SET seq = 9000 WHERE name = 'contracts'",
            [],
        )
        .expect("raise contracts AUTOINCREMENT watermark");
    drop(legacy);

    let upgraded = db::open_at(&database_path).expect("upgrade legacy contract schema");
    let contract = db::contract_get(&upgraded, "LEGACY-2026-0001")
        .expect("migrated contract remains readable");
    assert_eq!(contract.id, contract_id);
    assert_eq!(contract.kind, "nda");
    assert_eq!(contract.issuer_id, issuer_id);
    assert_eq!(contract.client_id, client_id);
    assert_eq!(contract.title, "nda contract");
    assert_eq!(contract.effective_date, "2026-09-10");
    assert_eq!(contract.governing_law, "England and Wales");
    assert_eq!(contract.status, "draft");
    assert_eq!(contract.terms_json, "{}");
    assert_eq!(contract.clause_pack, "standard");
    assert_eq!(contract.clause_pack_version, "2.0");

    let migrated_clauses =
        db::clauses_for(&upgraded, contract_id).expect("migrated clauses remain readable");
    assert_eq!(migrated_clauses.len(), 1);
    assert_eq!(migrated_clauses[0].slug, "purpose");
    assert_eq!(
        migrated_clauses[0].heading.as_deref(),
        Some("Legacy purpose")
    );
    assert_eq!(
        migrated_clauses[0].body.as_deref(),
        Some("Preserve this wording.")
    );
    assert!(object_exists(
        &upgraded,
        "index",
        "custom_contract_title_idx"
    ));
    assert!(object_exists(
        &upgraded,
        "trigger",
        "custom_contract_insert_audit"
    ));
    assert_database_health(&upgraded);
    let sequence: i64 = upgraded
        .query_row(
            "SELECT seq FROM sqlite_sequence WHERE name = 'contracts'",
            [],
            |row| row.get(0),
        )
        .expect("read contracts sequence after migration");
    assert_eq!(sequence, 9000);
    drop(upgraded);

    let reopened = db::open_at(&database_path).expect("reopen upgraded database idempotently");
    let contract_count: i64 = reopened
        .query_row("SELECT COUNT(*) FROM contracts", [], |row| row.get(0))
        .expect("count contracts after reopening");
    let clause_count: i64 = reopened
        .query_row("SELECT COUNT(*) FROM contract_clauses", [], |row| {
            row.get(0)
        })
        .expect("count clauses after reopening");
    assert_eq!(contract_count, 1);
    assert_eq!(clause_count, 1);
    assert!(object_exists(
        &reopened,
        "index",
        "custom_contract_title_idx"
    ));
    assert!(object_exists(
        &reopened,
        "trigger",
        "custom_contract_insert_audit"
    ));
    assert_database_health(&reopened);
    let sequence: i64 = reopened
        .query_row(
            "SELECT seq FROM sqlite_sequence WHERE name = 'contracts'",
            [],
            |row| row.get(0),
        )
        .expect("read contracts sequence after reopening");
    assert_eq!(sequence, 9000);

    let loan_id =
        insert_minimal_contract(&reopened, "LOAN-2026-0001", "loan", issuer_id, client_id);
    let ncnda_id =
        insert_minimal_contract(&reopened, "NCNDA-2026-0001", "ncnda", issuer_id, client_id);
    assert_eq!(loan_id, 9001, "migration preserves the high watermark");
    assert_eq!(ncnda_id, 9002);
    let audit_count: i64 = reopened
        .query_row("SELECT COUNT(*) FROM custom_contract_audit", [], |row| {
            row.get(0)
        })
        .expect("count custom trigger audit rows");
    assert_eq!(audit_count, 2, "restored trigger remains functional");

    db::contract_delete(&reopened, "LEGACY-2026-0001", false)
        .expect("delete migrated draft contract");
    let remaining_legacy_clauses: i64 = reopened
        .query_row(
            "SELECT COUNT(*) FROM contract_clauses WHERE contract_id = ?1",
            [contract_id],
            |row| row.get(0),
        )
        .expect("count clauses after deleting migrated contract");
    assert_eq!(
        remaining_legacy_clauses, 0,
        "draft deletion cascades to clauses"
    );
    assert_database_health(&reopened);
}

fn assert_pack_is_self_consistent(pack: &clauses::Pack) {
    assert!(!pack.pack.default_clauses.is_empty());
    for slug in &pack.pack.default_clauses {
        assert!(
            pack.clauses.contains_key(slug),
            "{}/{} {} names missing default clause `{slug}`",
            pack.pack.kind,
            pack.pack.slug,
            pack.pack.version
        );
    }
}

#[test]
fn embedded_clause_packs_and_historical_versions_remain_loadable() {
    let packs = clauses::list_packs();
    assert!(!packs.is_empty(), "current pack discovery is non-empty");
    assert!(
        packs.iter().all(|(kind, _)| kind != "archive"),
        "historical archive is excluded from current-pack discovery"
    );

    for (kind, slug) in &packs {
        let pack = clauses::load_pack(kind, slug)
            .unwrap_or_else(|error| panic!("load current pack {kind}/{slug}: {error}"));
        assert_eq!(&pack.pack.kind, kind);
        assert_eq!(&pack.pack.slug, slug);
        assert_pack_is_self_consistent(&pack);
    }

    for (kind, version) in [
        ("consulting", "1.1"),
        ("loan", "1.0"),
        ("msa", "1.0"),
        ("ncnda", "1.0"),
        ("nda", "1.1"),
        ("service", "1.0"),
        ("sow", "1.0"),
    ] {
        let historical = clauses::load_pack_version(kind, "standard", version)
            .unwrap_or_else(|error| panic!("load historical {kind}/standard {version}: {error}"));
        assert_eq!(historical.pack.kind, kind);
        assert_eq!(historical.pack.slug, "standard");
        assert_eq!(historical.pack.version, version);
        assert_pack_is_self_consistent(&historical);
    }

    let error = clauses::load_pack_version("nda", "standard", "0.0")
        .expect_err("unknown historical version must be rejected");
    assert!(
        error.to_string().contains("version 0.0 is unavailable"),
        "unknown-version error identifies the unavailable version: {error}"
    );
}
