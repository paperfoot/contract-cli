use crate::cli::ClauseCmd;
use crate::clauses;
use crate::db::{self, ContractClauseRow};
use crate::error::{AppError, Result};
use crate::output::{print_success, Ctx};

pub fn run(cmd: ClauseCmd, ctx: Ctx) -> Result<()> {
    match cmd {
        ClauseCmd::List { number } => list(&number, ctx),
        ClauseCmd::Add {
            number,
            slug,
            heading,
            body,
            from_file,
            position,
        } => add(&number, &slug, heading, body, from_file, position, ctx),
        ClauseCmd::Edit {
            number,
            slug,
            heading,
            body,
            from_file,
        } => edit(&number, &slug, heading, body, from_file, ctx),
        ClauseCmd::Remove { number, slug } => remove(&number, &slug, ctx),
        ClauseCmd::Move {
            number,
            slug,
            position,
        } => move_clause(&number, &slug, position, ctx),
        ClauseCmd::Reset { number } => reset(&number, ctx),
    }
}

fn body_text(body: Option<String>, from_file: Option<String>) -> Result<Option<String>> {
    match (body, from_file) {
        (Some(_), Some(_)) => Err(AppError::InvalidInput(
            "pass either --body or --from-file, not both".into(),
        )),
        (Some(b), None) => Ok(Some(b)),
        (None, Some(p)) => Ok(Some(std::fs::read_to_string(&p).map_err(|e| {
            AppError::InvalidInput(format!("could not read {p}: {e}"))
        })?)),
        (None, None) => Ok(None),
    }
}

fn list(number: &str, ctx: Ctx) -> Result<()> {
    let conn = db::open()?;
    let c = db::contract_get_or_404(&conn, number)?;
    let rows = db::clauses_for(&conn, c.id)?;
    print_success(ctx, &rows, |rs| {
        if rs.is_empty() {
            println!("(no clauses)");
        } else {
            for r in rs {
                let custom = if r.heading.is_some() || r.body.is_some() {
                    " [custom]"
                } else {
                    ""
                };
                println!(
                    "  {:>2}. {:<22} {}",
                    r.position + 1,
                    r.slug,
                    r.heading.clone().unwrap_or_default() + custom
                );
            }
        }
    });
    Ok(())
}

fn add(
    number: &str,
    slug: &str,
    heading: Option<String>,
    body: Option<String>,
    from_file: Option<String>,
    position: Option<i64>,
    ctx: Ctx,
) -> Result<()> {
    let mut conn = db::open()?;
    let body = body_text(body, from_file)?;
    // Validate slug exists in the pack OR a custom body was provided.
    let c = db::contract_get_or_404(&conn, number)?;
    let pack = clauses::load_pack(&c.kind, &c.clause_pack)?;
    if body.is_none() && !pack.clauses.contains_key(slug) {
        return Err(AppError::NotFound(format!(
            "clause '{slug}' is not in pack '{}/{}'. Either pick a known slug or pass --body / --from-file to define a custom clause.",
            c.kind, c.clause_pack
        )));
    }
    // If using a custom clause that's not in the pack, also require a heading.
    let heading = if body.is_some() && !pack.clauses.contains_key(slug) && heading.is_none() {
        Some(humanise_slug(slug))
    } else {
        heading
    };
    let row = db::clause_add(&mut conn, number, slug, heading.as_deref(), body.as_deref(), position)?;
    print_success(ctx, &row, |r| {
        println!("added clause '{}' at position {}", r.slug, r.position + 1);
    });
    Ok(())
}

fn humanise_slug(slug: &str) -> String {
    slug.replace('_', " ")
        .split(' ')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn edit(
    number: &str,
    slug: &str,
    heading: Option<String>,
    body: Option<String>,
    from_file: Option<String>,
    ctx: Ctx,
) -> Result<()> {
    let conn = db::open()?;
    let body = body_text(body, from_file)?;
    if heading.is_none() && body.is_none() {
        return Err(AppError::InvalidInput(
            "pass at least one of --heading / --body / --from-file".into(),
        ));
    }
    db::clause_edit(&conn, number, slug, heading.as_deref(), body.as_deref())?;
    print_success(ctx, &slug, |s| println!("edited clause '{s}'"));
    Ok(())
}

fn remove(number: &str, slug: &str, ctx: Ctx) -> Result<()> {
    let mut conn = db::open()?;
    db::clause_remove(&mut conn, number, slug)?;
    print_success(ctx, &slug, |s| println!("removed clause '{s}'"));
    Ok(())
}

fn move_clause(number: &str, slug: &str, position: i64, ctx: Ctx) -> Result<()> {
    let mut conn = db::open()?;
    db::clause_move(&mut conn, number, slug, position)?;
    print_success(ctx, &slug, |s| {
        println!("moved clause '{s}' to position {}", position + 1)
    });
    Ok(())
}

fn reset(number: &str, ctx: Ctx) -> Result<()> {
    let mut conn = db::open()?;
    let c = db::contract_get_or_404(&conn, number)?;
    let pack = clauses::load_pack(&c.kind, &c.clause_pack)?;
    let fresh: Vec<ContractClauseRow> = pack
        .pack
        .default_clauses
        .iter()
        .enumerate()
        .map(|(i, slug)| ContractClauseRow {
            id: 0,
            contract_id: 0,
            position: i as i64,
            slug: slug.clone(),
            heading: None,
            body: None,
        })
        .collect();
    db::clauses_reset(&mut conn, number, &fresh)?;
    print_success(ctx, &fresh, |f| {
        println!("reset to pack default ({} clauses)", f.len());
    });
    Ok(())
}
