// ═══════════════════════════════════════════════════════════════════════════
// Database layer — operates over the shared finance-core SQLite.
//
// Tables read/written: issuers (shared), clients (shared, w/ V7 legal fields),
// contracts (V7), contract_clauses (V7), number_series (shared).
// ═══════════════════════════════════════════════════════════════════════════

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::{AppError, Result};
use crate::tax::Jurisdiction;

pub use finance_core::entity::Issuer;

pub fn open() -> Result<Connection> {
    let paths = finance_core::paths::Paths::resolve()?;
    Ok(finance_core::db::open(&paths)?)
}

pub fn open_at(path: &Path) -> Result<Connection> {
    Ok(finance_core::db::open_at(path)?)
}

// ─── Issuers (shared with invoice-cli) ────────────────────────────────────

fn addr_to_text(lines: &[String]) -> String {
    lines.join("\n")
}
fn text_to_addr(s: &str) -> Vec<String> {
    s.split('\n').map(|l| l.to_string()).collect()
}

pub fn issuer_create(conn: &Connection, issuer: &Issuer) -> Result<i64> {
    conn.execute(
        "INSERT INTO issuers (slug, name, legal_name, jurisdiction, tax_registered,
                              tax_id, company_no, tagline, address, email, phone,
                              bank_details, default_template,
                              currency, symbol, number_format, logo_path,
                              default_output_dir, default_notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
        params![
            issuer.slug,
            issuer.name,
            issuer.legal_name,
            issuer.jurisdiction.as_str(),
            issuer.tax_registered as i32,
            issuer.tax_id,
            issuer.company_no,
            issuer.tagline,
            addr_to_text(&issuer.address),
            issuer.email,
            issuer.phone,
            issuer.bank_details,
            issuer.default_template,
            issuer.currency,
            issuer.symbol,
            issuer.number_format,
            issuer.logo_path,
            issuer.default_output_dir,
            issuer.default_notes,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn issuer_list(conn: &Connection) -> Result<Vec<Issuer>> {
    let mut stmt = conn.prepare(
        "SELECT id, slug, name, legal_name, jurisdiction, tax_registered,
                tax_id, company_no, tagline, address, email, phone,
                bank_details, default_template,
                currency, symbol, number_format, logo_path,
                default_output_dir, default_notes
         FROM issuers ORDER BY slug",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Issuer {
                id: row.get(0)?,
                slug: row.get(1)?,
                name: row.get(2)?,
                legal_name: row.get(3)?,
                jurisdiction: Jurisdiction::from_str(&row.get::<_, String>(4)?)
                    .unwrap_or(Jurisdiction::Custom),
                tax_registered: row.get::<_, i32>(5)? != 0,
                tax_id: row.get(6)?,
                company_no: row.get(7)?,
                tagline: row.get(8)?,
                address: text_to_addr(&row.get::<_, String>(9)?),
                email: row.get(10)?,
                phone: row.get(11)?,
                bank_details: row.get(12)?,
                default_template: row.get(13)?,
                currency: row.get(14)?,
                symbol: row.get(15)?,
                number_format: row.get(16)?,
                logo_path: row.get(17)?,
                default_output_dir: row.get(18)?,
                default_notes: row.get(19)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn issuer_by_slug(conn: &Connection, slug: &str) -> Result<Issuer> {
    for i in issuer_list(conn)? {
        if i.slug == slug {
            return Ok(i);
        }
    }
    let lower = slug.to_lowercase();
    let matches: Vec<Issuer> = issuer_list(conn)?
        .into_iter()
        .filter(|i| i.slug.contains(slug) || i.name.to_lowercase().contains(&lower))
        .collect();
    match matches.len() {
        0 => Err(AppError::NotFound(format!("issuer '{slug}'"))),
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => Err(AppError::Ambiguous(format!(
            "issuer '{slug}' matches {}",
            matches
                .iter()
                .map(|m| m.slug.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

pub fn issuer_update(conn: &Connection, issuer: &Issuer) -> Result<()> {
    let affected = conn.execute(
        "UPDATE issuers SET
             name = ?1, legal_name = ?2, jurisdiction = ?3, tax_registered = ?4,
             tax_id = ?5, company_no = ?6, tagline = ?7, address = ?8,
             email = ?9, phone = ?10, bank_details = ?11, default_template = ?12,
             currency = ?13, symbol = ?14, number_format = ?15, logo_path = ?16,
             default_output_dir = ?17, default_notes = ?18
         WHERE slug = ?19",
        params![
            issuer.name,
            issuer.legal_name,
            issuer.jurisdiction.as_str(),
            issuer.tax_registered as i32,
            issuer.tax_id,
            issuer.company_no,
            issuer.tagline,
            addr_to_text(&issuer.address),
            issuer.email,
            issuer.phone,
            issuer.bank_details,
            issuer.default_template,
            issuer.currency,
            issuer.symbol,
            issuer.number_format,
            issuer.logo_path,
            issuer.default_output_dir,
            issuer.default_notes,
            issuer.slug,
        ],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("issuer '{}'", issuer.slug)));
    }
    Ok(())
}

pub fn issuer_delete(conn: &Connection, slug: &str) -> Result<()> {
    let affected = conn.execute("DELETE FROM issuers WHERE slug = ?1", params![slug])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("issuer '{slug}'")));
    }
    Ok(())
}

// ─── Clients (shared, with V7 legal fields) ───────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub legal_name: Option<String>,
    pub company_no: Option<String>,
    pub legal_jurisdiction: Option<String>,
    pub attn: Option<String>,
    pub country: Option<String>,
    pub tax_id: Option<String>,
    pub address: Vec<String>,
    pub email: Option<String>,
    pub notes: Option<String>,
    pub default_issuer_slug: Option<String>,
    pub default_template: Option<String>,
}

pub fn client_create(conn: &Connection, c: &Client) -> Result<i64> {
    conn.execute(
        "INSERT INTO clients (slug, name, attn, country, tax_id, address, email, notes,
                              default_issuer_slug, default_template,
                              legal_name, company_no, legal_jurisdiction)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            c.slug,
            c.name,
            c.attn,
            c.country,
            c.tax_id,
            addr_to_text(&c.address),
            c.email,
            c.notes,
            c.default_issuer_slug,
            c.default_template,
            c.legal_name,
            c.company_no,
            c.legal_jurisdiction,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn client_list(conn: &Connection) -> Result<Vec<Client>> {
    let mut stmt = conn.prepare(
        "SELECT id, slug, name, attn, country, tax_id, address, email, notes,
                default_issuer_slug, default_template,
                legal_name, company_no, legal_jurisdiction
         FROM clients ORDER BY slug",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Client {
                id: row.get(0)?,
                slug: row.get(1)?,
                name: row.get(2)?,
                attn: row.get(3)?,
                country: row.get(4)?,
                tax_id: row.get(5)?,
                address: text_to_addr(&row.get::<_, String>(6)?),
                email: row.get(7)?,
                notes: row.get(8)?,
                default_issuer_slug: row.get(9)?,
                default_template: row.get(10)?,
                legal_name: row.get(11)?,
                company_no: row.get(12)?,
                legal_jurisdiction: row.get(13)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn client_by_slug(conn: &Connection, slug: &str) -> Result<Client> {
    for c in client_list(conn)? {
        if c.slug == slug {
            return Ok(c);
        }
    }
    let lower = slug.to_lowercase();
    let matches: Vec<Client> = client_list(conn)?
        .into_iter()
        .filter(|c| c.slug.contains(slug) || c.name.to_lowercase().contains(&lower))
        .collect();
    match matches.len() {
        0 => Err(AppError::NotFound(format!("client '{slug}'"))),
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => Err(AppError::Ambiguous(format!(
            "client '{slug}' matches {}",
            matches
                .iter()
                .map(|m| m.slug.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

pub fn client_update(conn: &Connection, c: &Client) -> Result<()> {
    let affected = conn.execute(
        "UPDATE clients SET
             name = ?1, attn = ?2, country = ?3, tax_id = ?4, address = ?5,
             email = ?6, notes = ?7, default_issuer_slug = ?8, default_template = ?9,
             legal_name = ?10, company_no = ?11, legal_jurisdiction = ?12
         WHERE slug = ?13",
        params![
            c.name,
            c.attn,
            c.country,
            c.tax_id,
            addr_to_text(&c.address),
            c.email,
            c.notes,
            c.default_issuer_slug,
            c.default_template,
            c.legal_name,
            c.company_no,
            c.legal_jurisdiction,
            c.slug,
        ],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("client '{}'", c.slug)));
    }
    Ok(())
}

pub fn client_delete(conn: &Connection, slug: &str) -> Result<()> {
    let affected = conn.execute("DELETE FROM clients WHERE slug = ?1", params![slug])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("client '{slug}'")));
    }
    Ok(())
}

// ─── Contracts ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub id: i64,
    pub number: String,
    pub kind: String,
    pub issuer_id: i64,
    pub client_id: i64,
    pub title: String,
    pub effective_date: String,
    pub end_date: Option<String>,
    pub term_months: Option<i64>,
    pub governing_law: String,
    pub venue: Option<String>,
    pub status: String,
    pub sent_at: Option<String>,
    pub signed_at: Option<String>,
    pub terminated_at: Option<String>,
    pub notes: Option<String>,
    pub fee_type: Option<String>,
    pub fee_amount_minor: Option<i64>,
    pub fee_currency: Option<String>,
    pub fee_schedule: Option<String>,
    pub terms_json: String,
    pub clause_pack: String,
    pub clause_pack_version: String,
    pub default_template: Option<String>,
    pub signed_by_us_name: Option<String>,
    pub signed_by_us_title: Option<String>,
    pub signed_by_us_at: Option<String>,
    pub signed_by_them_name: Option<String>,
    pub signed_by_them_title: Option<String>,
    pub signed_by_them_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractClauseRow {
    pub id: i64,
    pub contract_id: i64,
    pub position: i64,
    pub slug: String,
    pub heading: Option<String>,
    pub body: Option<String>,
}

const SELECT_CONTRACT: &str = "SELECT c.id, c.number, c.kind, c.issuer_id, c.client_id, c.title,
        c.effective_date, c.end_date, c.term_months, c.governing_law, c.venue,
        c.status, c.sent_at, c.signed_at, c.terminated_at, c.notes,
        c.fee_type, c.fee_amount_minor, c.fee_currency, c.fee_schedule,
        c.terms_json, c.clause_pack, c.clause_pack_version, c.default_template,
        c.signed_by_us_name, c.signed_by_us_title, c.signed_by_us_at,
        c.signed_by_them_name, c.signed_by_them_title, c.signed_by_them_at,
        c.created_at, c.updated_at";

fn row_to_contract(row: &rusqlite::Row) -> rusqlite::Result<Contract> {
    Ok(Contract {
        id: row.get(0)?,
        number: row.get(1)?,
        kind: row.get(2)?,
        issuer_id: row.get(3)?,
        client_id: row.get(4)?,
        title: row.get(5)?,
        effective_date: row.get(6)?,
        end_date: row.get(7)?,
        term_months: row.get(8)?,
        governing_law: row.get(9)?,
        venue: row.get(10)?,
        status: row.get(11)?,
        sent_at: row.get(12)?,
        signed_at: row.get(13)?,
        terminated_at: row.get(14)?,
        notes: row.get(15)?,
        fee_type: row.get(16)?,
        fee_amount_minor: row.get(17)?,
        fee_currency: row.get(18)?,
        fee_schedule: row.get(19)?,
        terms_json: row.get(20)?,
        clause_pack: row.get(21)?,
        clause_pack_version: row.get(22)?,
        default_template: row.get(23)?,
        signed_by_us_name: row.get(24)?,
        signed_by_us_title: row.get(25)?,
        signed_by_us_at: row.get(26)?,
        signed_by_them_name: row.get(27)?,
        signed_by_them_title: row.get(28)?,
        signed_by_them_at: row.get(29)?,
        created_at: row.get(30)?,
        updated_at: row.get(31)?,
    })
}

pub fn contract_create(conn: &mut Connection, c: &Contract, clauses: &[ContractClauseRow]) -> Result<i64> {
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO contracts (number, kind, issuer_id, client_id, title,
                                effective_date, end_date, term_months,
                                governing_law, venue, status, notes,
                                fee_type, fee_amount_minor, fee_currency, fee_schedule,
                                terms_json, clause_pack, clause_pack_version, default_template)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
        params![
            c.number,
            c.kind,
            c.issuer_id,
            c.client_id,
            c.title,
            c.effective_date,
            c.end_date,
            c.term_months,
            c.governing_law,
            c.venue,
            c.status,
            c.notes,
            c.fee_type,
            c.fee_amount_minor,
            c.fee_currency,
            c.fee_schedule,
            c.terms_json,
            c.clause_pack,
            c.clause_pack_version,
            c.default_template,
        ],
    )?;
    let id = tx.last_insert_rowid();
    for cl in clauses {
        tx.execute(
            "INSERT INTO contract_clauses (contract_id, position, slug, heading, body)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, cl.position, cl.slug, cl.heading, cl.body],
        )?;
    }
    tx.commit()?;
    Ok(id)
}

pub fn contract_get(conn: &Connection, number: &str) -> Result<Contract> {
    let sql = format!("{SELECT_CONTRACT} FROM contracts c WHERE c.number = ?1");
    Ok(conn.query_row(&sql, params![number], row_to_contract)?)
}

pub fn contract_get_or_404(conn: &Connection, number: &str) -> Result<Contract> {
    let sql = format!("{SELECT_CONTRACT} FROM contracts c WHERE c.number = ?1");
    conn.query_row(&sql, params![number], row_to_contract)
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("contract '{number}'")))
}

pub fn contract_list(
    conn: &Connection,
    kind: Option<&str>,
    status: Option<&str>,
    issuer_slug: Option<&str>,
) -> Result<Vec<Contract>> {
    let mut sql = format!(
        "{SELECT_CONTRACT} FROM contracts c JOIN issuers s ON s.id = c.issuer_id WHERE 1=1"
    );
    let mut p: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(k) = kind {
        sql.push_str(" AND c.kind = ?");
        p.push(Box::new(k.to_string()));
    }
    if let Some(st) = status {
        sql.push_str(" AND c.status = ?");
        p.push(Box::new(st.to_string()));
    }
    if let Some(sl) = issuer_slug {
        sql.push_str(" AND s.slug = ?");
        p.push(Box::new(sl.to_string()));
    }
    sql.push_str(" ORDER BY c.effective_date DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(
            rusqlite::params_from_iter(p.iter().map(|b| b.as_ref())),
            row_to_contract,
        )?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn contract_update_draft(conn: &Connection, c: &Contract) -> Result<()> {
    let status: Option<String> = conn
        .query_row(
            "SELECT status FROM contracts WHERE number = ?1",
            params![c.number],
            |r| r.get(0),
        )
        .optional()?;
    let status = status.ok_or_else(|| AppError::NotFound(format!("contract '{}'", c.number)))?;
    if status != "draft" {
        return Err(AppError::InvalidInput(format!(
            "contract '{}' is {status}, not draft — sent/signed contracts are immutable.",
            c.number
        )));
    }
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE contracts SET
             client_id = ?1, title = ?2, effective_date = ?3, end_date = ?4,
             term_months = ?5, governing_law = ?6, venue = ?7, notes = ?8,
             fee_type = ?9, fee_amount_minor = ?10, fee_currency = ?11, fee_schedule = ?12,
             terms_json = ?13, default_template = ?14, updated_at = ?15
         WHERE number = ?16",
        params![
            c.client_id,
            c.title,
            c.effective_date,
            c.end_date,
            c.term_months,
            c.governing_law,
            c.venue,
            c.notes,
            c.fee_type,
            c.fee_amount_minor,
            c.fee_currency,
            c.fee_schedule,
            c.terms_json,
            c.default_template,
            now,
            c.number,
        ],
    )?;
    Ok(())
}

pub fn contract_set_status(conn: &Connection, number: &str, status: &str) -> Result<()> {
    let valid = ["draft", "sent", "signed", "active", "expired", "terminated"];
    if !valid.contains(&status) {
        return Err(AppError::InvalidInput(format!(
            "invalid status '{status}'. Expected one of: {}",
            valid.join(", ")
        )));
    }
    let now = chrono::Utc::now().to_rfc3339();
    let affected = match status {
        "sent" => conn.execute(
            "UPDATE contracts SET status = ?1, sent_at = COALESCE(sent_at, ?2), updated_at = ?2 WHERE number = ?3",
            params![status, now, number],
        )?,
        "signed" | "active" => conn.execute(
            "UPDATE contracts SET status = ?1, signed_at = COALESCE(signed_at, ?2), updated_at = ?2 WHERE number = ?3",
            params![status, now, number],
        )?,
        "terminated" => conn.execute(
            "UPDATE contracts SET status = ?1, terminated_at = COALESCE(terminated_at, ?2), updated_at = ?2 WHERE number = ?3",
            params![status, now, number],
        )?,
        _ => conn.execute(
            "UPDATE contracts SET status = ?1, updated_at = ?2 WHERE number = ?3",
            params![status, now, number],
        )?,
    };
    if affected == 0 {
        return Err(AppError::NotFound(format!("contract '{number}'")));
    }
    Ok(())
}

pub fn contract_record_signature(
    conn: &Connection,
    number: &str,
    side: &str,
    name: &str,
    title: Option<&str>,
    date_iso: &str,
) -> Result<Contract> {
    let now = chrono::Utc::now().to_rfc3339();
    let affected = match side {
        "us" => conn.execute(
            "UPDATE contracts SET signed_by_us_name = ?1, signed_by_us_title = ?2, signed_by_us_at = ?3, updated_at = ?4 WHERE number = ?5",
            params![name, title, date_iso, now, number],
        )?,
        "them" => conn.execute(
            "UPDATE contracts SET signed_by_them_name = ?1, signed_by_them_title = ?2, signed_by_them_at = ?3, updated_at = ?4 WHERE number = ?5",
            params![name, title, date_iso, now, number],
        )?,
        _ => {
            return Err(AppError::InvalidInput(format!(
                "side must be 'us' or 'them', got '{side}'"
            )))
        }
    };
    if affected == 0 {
        return Err(AppError::NotFound(format!("contract '{number}'")));
    }
    // If both sides have signatures, auto-bump status to 'signed' (idempotent)
    let c = contract_get(conn, number)?;
    if c.signed_by_us_at.is_some()
        && c.signed_by_them_at.is_some()
        && (c.status == "draft" || c.status == "sent")
    {
        contract_set_status(conn, number, "signed")?;
        return contract_get(conn, number);
    }
    Ok(c)
}

pub fn contract_delete(conn: &Connection, number: &str, force: bool) -> Result<()> {
    let status: Option<String> = conn
        .query_row(
            "SELECT status FROM contracts WHERE number = ?1",
            params![number],
            |r| r.get(0),
        )
        .optional()?;
    let status = status.ok_or_else(|| AppError::NotFound(format!("contract '{number}'")))?;
    if status != "draft" && !force {
        return Err(AppError::InvalidInput(format!(
            "refusing to delete non-draft contract '{number}' (status='{status}') — pass --force"
        )));
    }
    conn.execute("DELETE FROM contracts WHERE number = ?1", params![number])?;
    Ok(())
}

// ─── Contract clauses CRUD ───────────────────────────────────────────────

pub fn clauses_for(conn: &Connection, contract_id: i64) -> Result<Vec<ContractClauseRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, contract_id, position, slug, heading, body
         FROM contract_clauses WHERE contract_id = ?1 ORDER BY position",
    )?;
    let rows = stmt
        .query_map(params![contract_id], |r| {
            Ok(ContractClauseRow {
                id: r.get(0)?,
                contract_id: r.get(1)?,
                position: r.get(2)?,
                slug: r.get(3)?,
                heading: r.get(4)?,
                body: r.get(5)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn require_mutable(conn: &Connection, number: &str) -> Result<Contract> {
    let c = contract_get_or_404(conn, number)?;
    if c.status != "draft" {
        return Err(AppError::InvalidInput(format!(
            "contract '{number}' is {} — clauses can only be edited on draft.",
            c.status
        )));
    }
    Ok(c)
}

pub fn clause_add(
    conn: &mut Connection,
    number: &str,
    slug: &str,
    heading: Option<&str>,
    body: Option<&str>,
    position: Option<i64>,
) -> Result<ContractClauseRow> {
    let c = require_mutable(conn, number)?;
    let tx = conn.transaction()?;
    // disallow duplicates
    let exists: Option<i64> = tx
        .query_row(
            "SELECT id FROM contract_clauses WHERE contract_id = ?1 AND slug = ?2",
            params![c.id, slug],
            |r| r.get(0),
        )
        .optional()?;
    if exists.is_some() {
        return Err(AppError::InvalidInput(format!(
            "clause '{slug}' already on contract '{}'",
            number
        )));
    }
    let count: i64 = tx.query_row(
        "SELECT COUNT(*) FROM contract_clauses WHERE contract_id = ?1",
        params![c.id],
        |r| r.get(0),
    )?;
    let insert_pos = position.unwrap_or(count).clamp(0, count);
    // shift existing
    tx.execute(
        "UPDATE contract_clauses SET position = position + 1
           WHERE contract_id = ?1 AND position >= ?2",
        params![c.id, insert_pos],
    )?;
    tx.execute(
        "INSERT INTO contract_clauses (contract_id, position, slug, heading, body)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![c.id, insert_pos, slug, heading, body],
    )?;
    let id = tx.last_insert_rowid();
    tx.commit()?;
    Ok(ContractClauseRow {
        id,
        contract_id: c.id,
        position: insert_pos,
        slug: slug.to_string(),
        heading: heading.map(str::to_string),
        body: body.map(str::to_string),
    })
}

pub fn clause_remove(conn: &mut Connection, number: &str, slug: &str) -> Result<()> {
    let c = require_mutable(conn, number)?;
    let tx = conn.transaction()?;
    let pos: Option<i64> = tx
        .query_row(
            "SELECT position FROM contract_clauses WHERE contract_id = ?1 AND slug = ?2",
            params![c.id, slug],
            |r| r.get(0),
        )
        .optional()?;
    let pos = pos.ok_or_else(|| AppError::NotFound(format!("clause '{slug}' on '{number}'")))?;
    tx.execute(
        "DELETE FROM contract_clauses WHERE contract_id = ?1 AND slug = ?2",
        params![c.id, slug],
    )?;
    tx.execute(
        "UPDATE contract_clauses SET position = position - 1
           WHERE contract_id = ?1 AND position > ?2",
        params![c.id, pos],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn clause_edit(
    conn: &Connection,
    number: &str,
    slug: &str,
    heading: Option<&str>,
    body: Option<&str>,
) -> Result<()> {
    let c = require_mutable(conn, number)?;
    let affected = conn.execute(
        "UPDATE contract_clauses SET heading = COALESCE(?1, heading), body = COALESCE(?2, body)
         WHERE contract_id = ?3 AND slug = ?4",
        params![heading, body, c.id, slug],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!(
            "clause '{slug}' on contract '{number}'"
        )));
    }
    Ok(())
}

pub fn clause_move(conn: &mut Connection, number: &str, slug: &str, new_pos: i64) -> Result<()> {
    let c = require_mutable(conn, number)?;
    let tx = conn.transaction()?;
    let cur_pos: i64 = tx
        .query_row(
            "SELECT position FROM contract_clauses WHERE contract_id = ?1 AND slug = ?2",
            params![c.id, slug],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("clause '{slug}' on '{number}'")))?;
    let count: i64 = tx.query_row(
        "SELECT COUNT(*) FROM contract_clauses WHERE contract_id = ?1",
        params![c.id],
        |r| r.get(0),
    )?;
    let target = new_pos.clamp(0, count - 1);
    if target == cur_pos {
        tx.commit()?;
        return Ok(());
    }
    if target > cur_pos {
        tx.execute(
            "UPDATE contract_clauses SET position = position - 1
               WHERE contract_id = ?1 AND position > ?2 AND position <= ?3",
            params![c.id, cur_pos, target],
        )?;
    } else {
        tx.execute(
            "UPDATE contract_clauses SET position = position + 1
               WHERE contract_id = ?1 AND position >= ?2 AND position < ?3",
            params![c.id, target, cur_pos],
        )?;
    }
    tx.execute(
        "UPDATE contract_clauses SET position = ?1 WHERE contract_id = ?2 AND slug = ?3",
        params![target, c.id, slug],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn clauses_reset(
    conn: &mut Connection,
    number: &str,
    fresh: &[ContractClauseRow],
) -> Result<()> {
    let c = require_mutable(conn, number)?;
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM contract_clauses WHERE contract_id = ?1",
        params![c.id],
    )?;
    for cl in fresh {
        tx.execute(
            "INSERT INTO contract_clauses (contract_id, position, slug, heading, body)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![c.id, cl.position, cl.slug, cl.heading, cl.body],
        )?;
    }
    tx.commit()?;
    Ok(())
}

// ─── Numbering ────────────────────────────────────────────────────────────

/// Generate the next contract number for an issuer/year/kind triple. Uses the
/// shared `number_series` table with `kind = "contract.<kind>"` so contract
/// sequences live separately from invoice/credit-note sequences.
pub fn next_contract_number(
    conn: &Connection,
    issuer: &Issuer,
    year: i32,
    kind: &str,
) -> Result<String> {
    let series_kind = format!("contract.{kind}");
    let seq: i64 = conn.query_row(
        "INSERT INTO number_series (issuer_id, year, kind, next_seq)
         VALUES (?1, ?2, ?3, 2)
         ON CONFLICT(issuer_id, year, kind) DO UPDATE SET next_seq = next_seq + 1
         RETURNING next_seq - 1",
        params![issuer.id, year, series_kind],
        |r| r.get(0),
    )?;
    let prefix = match kind {
        "consulting" => "CTR",
        "nda" => "NDA",
        "msa" => "MSA",
        "sow" => "SOW",
        "service" => "SVC",
        _ => "DOC",
    };
    Ok(format!("{prefix}-{}-{}-{:04}", issuer.slug, year, seq))
}
