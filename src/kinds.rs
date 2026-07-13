// Registry of contract kinds: one place for the kind's identity — number
// prefix, party role labels, description, and the trigger tags used by
// `contract kinds find` / `--kind-like` matching. Adding a kind here plus a
// clause pack under clauses/<kind>/ is all it takes to teach the CLI a new
// contract type.

pub struct KindSpec {
    pub kind: &'static str,
    pub prefix: &'static str,
    /// (our role, their role) as printed on the contract
    pub roles: (&'static str, &'static str),
    pub description: &'static str,
    pub tags: &'static [&'static str],
}

pub const KINDS: &[KindSpec] = &[
    KindSpec {
        kind: "nda",
        prefix: "NDA",
        roles: ("Disclosing Party", "Receiving Party"),
        description: "Confidentiality agreement (mutual or one-way) — protects information exchanged while exploring a deal.",
        tags: &[
            "confidential", "confidentiality", "secret", "nondisclosure", "non-disclosure",
            "disclosure", "privacy", "protect", "information", "evaluate", "exploring",
        ],
    },
    KindSpec {
        kind: "ncnda",
        prefix: "NCNDA",
        roles: ("Party", "Party"),
        description: "Non-circumvention + non-disclosure — stops a counterparty going around you to deal directly with contacts you introduce, and protects the information shared.",
        tags: &[
            "circumvent", "circumvention", "non-circumvention", "go-around", "bypass",
            "introducer", "introduction", "intermediary", "broker", "middleman",
            "commission", "protect-contacts", "steal", "poach", "direct-deal", "cut-out",
        ],
    },
    KindSpec {
        kind: "consulting",
        prefix: "CTR",
        roles: ("Consultant", "Client"),
        description: "One engagement with defined deliverables and a fee — a self-contained services contract.",
        tags: &[
            "freelance", "project", "deliverables", "engagement", "gig", "advisory",
            "consultant", "services", "scope", "fee",
        ],
    },
    KindSpec {
        kind: "msa",
        prefix: "MSA",
        roles: ("Provider", "Customer"),
        description: "Master services agreement — umbrella terms with no fee or scope; SOWs hang off it per piece of work.",
        tags: &[
            "master", "framework", "umbrella", "long-term", "terms", "relationship",
        ],
    },
    KindSpec {
        kind: "sow",
        prefix: "SOW",
        roles: ("Provider", "Customer"),
        description: "Statement of work under an MSA — scope, deliverables, milestones, and price for one project.",
        tags: &[
            "scope", "milestones", "work-order", "statement", "under-msa", "project",
        ],
    },
    KindSpec {
        kind: "service",
        prefix: "SVC",
        roles: ("Provider", "Customer"),
        description: "Ongoing or recurring services — retainers, hosting, support, monthly reviews.",
        tags: &[
            "retainer", "ongoing", "recurring", "monthly", "hosting", "support",
            "subscription", "maintenance", "managed",
        ],
    },
    KindSpec {
        kind: "loan",
        prefix: "LOAN",
        roles: ("Lender", "Borrower"),
        description: "Loan agreement between two parties — principal, interest (or interest-free), repayment date, default, and optional security.",
        tags: &[
            "loan", "lend", "lending", "borrow", "money", "principal", "interest",
            "repayment", "repay", "credit", "advance", "bridge", "facility",
        ],
    },
];

pub fn get(kind: &str) -> Option<&'static KindSpec> {
    KINDS.iter().find(|k| k.kind == kind)
}

pub fn names() -> Vec<&'static str> {
    KINDS.iter().map(|k| k.kind).collect()
}

pub fn prefix_for(kind: &str) -> &'static str {
    get(kind).map(|k| k.prefix).unwrap_or("DOC")
}

/// Score a free-text description against a kind's tags + description.
/// Deterministic local token matching — no guessing, callers decide.
pub fn score(query: &str, description: &str, tags: &[&str]) -> f64 {
    let q = query.to_lowercase();
    let q_tokens: Vec<&str> = q
        .split(|c: char| !c.is_alphanumeric() && c != '-')
        .filter(|t| t.len() > 2)
        .collect();
    if q_tokens.is_empty() {
        return 0.0;
    }
    let desc = description.to_lowercase();
    let mut hits = 0.0;
    for t in &q_tokens {
        // exact tag hit is strongest; tag prefix and description hits count less
        if tags.iter().any(|tag| tag == t) {
            hits += 3.0;
        } else if tags.iter().any(|tag| tag.starts_with(*t) || t.starts_with(*tag)) {
            hits += 1.5;
        } else if desc.contains(*t) {
            hits += 1.0;
        }
    }
    hits / q_tokens.len() as f64
}
