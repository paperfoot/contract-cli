//! Explicit legal-profile selection and validation shared by creation, editing and rendering.
use crate::error::{AppError, Result};
use serde_json::{Map, Value};

fn invalid(message: impl Into<String>) -> AppError {
    AppError::InvalidInput(message.into())
}

pub fn validate_dates(effective: &str, end: Option<&str>) -> Result<()> {
    let parse = |s: &str| {
        chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|_| invalid(format!("invalid date {s}; expected YYYY-MM-DD")))
    };
    let start = parse(effective)?;
    if let Some(end) = end
        && parse(end)? < start
    {
        return Err(invalid("end date must not precede the effective date"));
    }
    Ok(())
}

pub fn validate_law(law: &str, venue: Option<&str>) -> Result<()> {
    let l = law.trim().to_lowercase();
    if l.is_empty()
        || matches!(
            l.as_str(),
            "uk" | "united kingdom"
                | "us"
                | "usa"
                | "united states"
                | "global"
                | "international"
                | "eu"
                | "european union"
        )
    {
        return Err(invalid(
            "choose a specific governing law, such as England and Wales, Scotland, Delaware or Singapore; global is a profile, not a legal system",
        ));
    }
    if venue.is_some_and(|v| v.trim().is_empty()) {
        return Err(invalid("court venue cannot be blank"));
    }
    Ok(())
}

pub fn select_profile(
    profile: Option<&str>,
    state: Option<&str>,
    law: Option<&str>,
    venue: Option<&str>,
) -> Result<Option<String>> {
    if state.is_some() && profile != Some("us") {
        return Err(invalid("--us-state requires --legal-profile us"));
    }
    let selected = match profile {
        Some("uk") => {
            let law = law.unwrap_or("England and Wales");
            if !["england and wales", "scotland", "northern ireland"]
                .contains(&law.trim().to_lowercase().as_str())
            {
                return Err(invalid(
                    "UK profile requires England and Wales, Scotland or Northern Ireland",
                ));
            }
            Some(law.to_string())
        }
        Some("singapore") => {
            if law.is_some_and(|l| !l.trim().eq_ignore_ascii_case("singapore")) {
                return Err(invalid(
                    "Singapore profile requires Singapore governing law",
                ));
            }
            Some("Singapore".into())
        }
        Some("us") => {
            let state = state
                .ok_or_else(|| invalid("US profile requires --us-state, e.g. Delaware"))?
                .trim();
            let states = [
                "Alabama",
                "Alaska",
                "Arizona",
                "Arkansas",
                "California",
                "Colorado",
                "Connecticut",
                "Delaware",
                "Florida",
                "Georgia",
                "Hawaii",
                "Idaho",
                "Illinois",
                "Indiana",
                "Iowa",
                "Kansas",
                "Kentucky",
                "Louisiana",
                "Maine",
                "Maryland",
                "Massachusetts",
                "Michigan",
                "Minnesota",
                "Mississippi",
                "Missouri",
                "Montana",
                "Nebraska",
                "Nevada",
                "New Hampshire",
                "New Jersey",
                "New Mexico",
                "New York",
                "North Carolina",
                "North Dakota",
                "Ohio",
                "Oklahoma",
                "Oregon",
                "Pennsylvania",
                "Rhode Island",
                "South Carolina",
                "South Dakota",
                "Tennessee",
                "Texas",
                "Utah",
                "Vermont",
                "Virginia",
                "Washington",
                "West Virginia",
                "Wisconsin",
                "Wyoming",
                "District of Columbia",
            ];
            let state = states
                .iter()
                .find(|s| s.eq_ignore_ascii_case(state))
                .ok_or_else(|| {
                    invalid("use the full name of a US state or District of Columbia")
                })?;
            if law.is_some_and(|l| !l.trim().eq_ignore_ascii_case(state)) {
                return Err(invalid("--governing-law must match --us-state"));
            }
            Some(state.to_string())
        }
        Some("global") => {
            let law =
                law.ok_or_else(|| invalid("global profile requires --governing-law and --venue"))?;
            if venue.is_none() {
                return Err(invalid("global profile requires an explicit --venue"));
            }
            Some(law.to_string())
        }
        Some(other) => return Err(invalid(format!("unknown legal profile {other}"))),
        None => law.map(str::to_string),
    };
    if let Some(law) = &selected {
        validate_law(law, venue)?;
    }
    Ok(selected)
}

pub fn validate_terms(terms: &mut Map<String, Value>) -> Result<()> {
    for (key, min, max) in [
        ("confidentiality_years", 1, 100),
        ("termination_notice_days", 0, 3650),
        ("non_circumvention_months", 1, 120),
        ("acceptance_days", 1, 365),
        ("revision_rounds", 0, 100),
    ] {
        if let Some(value) = terms.get(key) {
            let n = value
                .as_i64()
                .or_else(|| value.as_str().and_then(|s| s.parse::<i64>().ok()))
                .ok_or_else(|| invalid(format!("{key} must be a whole number")))?;
            if n < min || n > max {
                return Err(invalid(format!("{key} must be between {min} and {max}")));
            }
            terms.insert(key.into(), Value::from(n));
        }
    }
    for (key, choices) in [
        ("mutuality", &["mutual", "unilateral"][..]),
        ("disclosing_side", &["us", "them", "both"][..]),
        (
            "ip_assignment",
            &["client", "consultant", "provider", "shared"][..],
        ),
        ("legal_profile", &["global", "uk", "us", "singapore"][..]),
    ] {
        if let Some(value) = terms.get(key)
            && !value.as_str().is_some_and(|s| choices.contains(&s))
        {
            return Err(invalid(format!(
                "invalid {key}; expected {}",
                choices.join(" | ")
            )));
        }
    }
    let unilateral = terms.get("mutuality").and_then(Value::as_str) == Some("unilateral");
    if let Some(side) = terms.get("disclosing_side").and_then(Value::as_str)
        && ((unilateral && side == "both") || (!unilateral && side != "both"))
    {
        return Err(invalid(
            "mutual NDAs require disclosing_side=both; unilateral NDAs require us or them",
        ));
    }
    if let Some(v) = terms.get("deliverables")
        && !v.as_array().is_some_and(|a| {
            a.iter()
                .all(|v| v.as_str().is_some_and(|s| !s.trim().is_empty()))
        })
    {
        return Err(invalid(
            "deliverables must be a list of non-empty strings; use --deliverable",
        ));
    }
    Ok(())
}

pub fn venue_phrase(law: &str, venue: Option<&str>, profile: Option<&str>) -> String {
    if let Some(venue) = venue {
        return venue.trim().to_string();
    }
    if profile == Some("us") {
        return format!(
            "the state courts of {law} and, where federal subject-matter jurisdiction exists, the federal courts located there"
        );
    }
    format!("the courts of {}", law.trim())
}

/// DTSA immunity notice. Only inserted for an explicitly selected US profile.
pub const US_IMMUNITY: &str = "Under 18 U.S.C. § 1833(b), an individual is immune from liability under federal or state trade-secret law for disclosing a trade secret in confidence, directly or indirectly, to a government official or attorney solely to report or investigate a suspected violation of law, or in a complaint or other document filed under seal in a proceeding. An individual bringing a retaliation claim for reporting suspected illegality may disclose the trade secret to their attorney and use it in that proceeding if documents containing it are filed under seal and it is not otherwise disclosed except by court order. These protections include qualifying contractors and consultants. Nothing in this agreement restricts those rights.";
