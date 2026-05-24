use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "contract",
    version,
    about = "Beautiful contracts from the CLI — NDA, consulting, MSA, SOW, service"
)]
pub struct Cli {
    /// Emit JSON envelope on stdout (auto-detected when piped)
    #[arg(long, global = true)]
    pub json: bool,
    /// Suppress human output
    #[arg(long, global = true)]
    pub quiet: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage issuers (companies you contract AS)
    #[command(visible_alias = "issuers", subcommand)]
    Issuer(IssuerCmd),

    /// Manage clients (counterparties)
    #[command(subcommand)]
    Clients(ClientCmd),

    /// Manage contracts (new/list/show/render/mark/sign/edit/duplicate/delete)
    #[command(subcommand)]
    Contracts(ContractCmd),

    /// Shorthand: `contract new …` (= `contract contracts new …`)
    #[command(name = "new")]
    New(ContractNewArgs),

    /// Shorthand: `contract list` (= `contract contracts list`)
    #[command(name = "list", alias = "ls")]
    List(ContractListArgs),

    /// Shorthand: `contract show <number>`
    #[command(name = "show", alias = "get")]
    Show { number: String },

    /// Shorthand: `contract render <number>`
    #[command(name = "render")]
    Render(ContractRenderArgs),

    /// Shorthand: `contract mark <number> <status>`
    #[command(name = "mark")]
    Mark { number: String, status: String },

    /// Shorthand: `contract sign <number> --side us|them --name "..."`
    #[command(name = "sign")]
    Sign(SignArgs),

    /// Browse clause packs (the building blocks per contract kind)
    #[command(subcommand)]
    Pack(PackCmd),

    /// Inspect / render templates
    #[command(subcommand)]
    Template(TemplateCmd),

    /// Show / edit config
    #[command(subcommand)]
    Config(ConfigCmd),

    /// Self-describing JSON manifest for agents
    #[command(alias = "info")]
    AgentInfo,

    /// Install the embedded skill into ~/.claude, ~/.codex, ~/.gemini
    #[command(subcommand)]
    Skill(SkillCmd),

    /// Run dependency & config diagnostics
    Doctor,

    /// Self-update from GitHub Releases
    Update {
        #[arg(long)]
        check: bool,
    },
}

// ─── Issuers ─────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum IssuerCmd {
    /// Add a new issuer
    #[command(alias = "new")]
    Add {
        slug: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        legal_name: Option<String>,
        #[arg(long, default_value = "sg")]
        jurisdiction: String,
        #[arg(long)]
        tax_id: Option<String>,
        #[arg(long)]
        company_no: Option<String>,
        #[arg(long)]
        address: String,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        phone: Option<String>,
        /// Path to logo image (PNG/SVG/JPG) — used in contract header
        #[arg(long)]
        logo: Option<String>,
        /// Default output dir for `contract render`
        #[arg(long)]
        output_dir: Option<String>,
    },
    Edit {
        slug: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        legal_name: Option<String>,
        #[arg(long)]
        jurisdiction: Option<String>,
        #[arg(long)]
        tax_id: Option<String>,
        #[arg(long)]
        company_no: Option<String>,
        #[arg(long)]
        address: Option<String>,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        phone: Option<String>,
        #[arg(long)]
        logo: Option<String>,
        #[arg(long)]
        logo_clear: bool,
        #[arg(long)]
        output_dir: Option<String>,
    },
    #[command(alias = "ls")]
    List,
    #[command(alias = "get")]
    Show { slug: String },
    #[command(alias = "rm")]
    Delete { slug: String },
}

// ─── Clients ─────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum ClientCmd {
    #[command(alias = "new")]
    Add {
        slug: String,
        #[arg(long)]
        name: String,
        /// Legal entity name on the contract (e.g. "Meridian & Co. Pty Ltd")
        #[arg(long)]
        legal_name: Option<String>,
        /// Registration / company number
        #[arg(long)]
        company_no: Option<String>,
        /// Legal jurisdiction of incorporation (e.g. "Singapore", "Delaware")
        #[arg(long)]
        jurisdiction: Option<String>,
        #[arg(long)]
        attn: Option<String>,
        #[arg(long)]
        country: Option<String>,
        #[arg(long)]
        address: String,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        notes: Option<String>,
    },
    Edit {
        slug: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        legal_name: Option<String>,
        #[arg(long)]
        company_no: Option<String>,
        #[arg(long)]
        jurisdiction: Option<String>,
        #[arg(long)]
        attn: Option<String>,
        #[arg(long)]
        country: Option<String>,
        #[arg(long)]
        address: Option<String>,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        notes: Option<String>,
    },
    #[command(alias = "ls")]
    List,
    #[command(alias = "get")]
    Show { slug: String },
    #[command(alias = "rm")]
    Delete { slug: String },
}

// ─── Contracts ───────────────────────────────────────────────────────────

#[derive(clap::Args, Debug, Clone)]
pub struct ContractNewArgs {
    /// Contract kind: nda | consulting | msa | sow | service
    #[arg(long)]
    pub kind: String,
    /// Issuer slug (your side)
    #[arg(long = "as")]
    pub r#as: Option<String>,
    /// Client slug (counterparty)
    #[arg(long)]
    pub client: String,
    /// Title — defaults to a sensible per-kind label
    #[arg(long)]
    pub title: Option<String>,
    /// Effective date (YYYY-MM-DD; defaults to today)
    #[arg(long)]
    pub effective: Option<String>,
    /// Explicit end date (YYYY-MM-DD). Mutually exclusive with --term-months.
    #[arg(long, conflicts_with = "term_months")]
    pub end: Option<String>,
    /// Term length in months. Mutually exclusive with --end.
    #[arg(long)]
    pub term_months: Option<i64>,
    /// Term length in years (sugar for --term-months N*12)
    #[arg(long, conflicts_with_all = ["end", "term_months"])]
    pub term_years: Option<i64>,
    /// Governing law (e.g. "Singapore", "England and Wales", "Delaware")
    #[arg(long)]
    pub governing_law: Option<String>,
    /// Court venue (e.g. "Courts of Singapore")
    #[arg(long)]
    pub venue: Option<String>,
    /// Fee spec for consulting / msa / sow / service.
    /// Formats: "fixed:8400:SGD" | "hourly:200:SGD" | "daily:1500:SGD"
    ///        | "retainer:5000:SGD/month".
    #[arg(long)]
    pub fee: Option<String>,
    /// Payment schedule: on-completion | monthly | on-milestone | upon-invoice
    #[arg(long)]
    pub fee_schedule: Option<String>,
    /// NDA: unilateral | mutual (default mutual)
    #[arg(long)]
    pub mutuality: Option<String>,
    /// NDA: disclosing side — us | them | both (default both for mutual)
    #[arg(long)]
    pub disclosing_side: Option<String>,
    /// NDA / consulting: purpose / scope summary
    #[arg(long)]
    pub purpose: Option<String>,
    /// Consulting: deliverable lines (repeat). e.g. --deliverable "Discovery"
    #[arg(long = "deliverable")]
    pub deliverables: Vec<String>,
    /// Consulting: who owns work-product IP (client | consultant | shared)
    #[arg(long)]
    pub ip_assignment: Option<String>,
    /// Days of notice required for termination for convenience
    #[arg(long)]
    pub termination_notice_days: Option<i64>,
    /// Pack slug (default = "standard"). Picks a different clause library.
    #[arg(long)]
    pub pack: Option<String>,
    /// Append a clause slug from the pack that isn't included by default
    #[arg(long = "include")]
    pub include: Vec<String>,
    /// Remove a clause slug that the pack would include by default
    #[arg(long = "exclude")]
    pub exclude: Vec<String>,
    /// Free-form notes (not rendered on the contract body, kept for reference)
    #[arg(long)]
    pub notes: Option<String>,
    /// Override the rendered template (else config default)
    #[arg(long)]
    pub template: Option<String>,
}

#[derive(clap::Args, Debug, Clone)]
pub struct ContractListArgs {
    #[arg(long)]
    pub kind: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
    #[arg(long = "as")]
    pub issuer: Option<String>,
}

#[derive(clap::Args, Debug, Clone)]
pub struct ContractRenderArgs {
    pub number: String,
    /// Template override
    #[arg(long)]
    pub template: Option<String>,
    /// Output path (defaults to issuer default_output_dir / ./contract-<number>.pdf)
    #[arg(long, short)]
    pub out: Option<String>,
    /// Open the PDF after rendering
    #[arg(long)]
    pub open: bool,
    /// Render with DRAFT watermark — implicit when status != signed/active
    #[arg(long)]
    pub draft: bool,
    /// Force render without DRAFT watermark even if status != signed/active.
    /// Use when you genuinely want a clean copy for review before signing.
    #[arg(long = "final", conflicts_with = "draft")]
    pub final_render: bool,
}

#[derive(clap::Args, Debug, Clone)]
pub struct SignArgs {
    pub number: String,
    /// us | them
    #[arg(long)]
    pub side: String,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub title: Option<String>,
    /// Signature date (YYYY-MM-DD; defaults to today)
    #[arg(long)]
    pub date: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum ContractCmd {
    /// Create a new contract
    #[command(alias = "new")]
    New(ContractNewArgs),
    /// List contracts
    #[command(alias = "ls")]
    List(ContractListArgs),
    /// Show one contract's metadata + clause list
    #[command(alias = "get")]
    Show { number: String },
    /// Edit DRAFT contract metadata (status != draft is immutable)
    Edit {
        number: String,
        #[arg(long)]
        client: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        effective: Option<String>,
        #[arg(long)]
        end: Option<String>,
        #[arg(long)]
        term_months: Option<i64>,
        #[arg(long)]
        governing_law: Option<String>,
        #[arg(long)]
        venue: Option<String>,
        #[arg(long)]
        fee: Option<String>,
        #[arg(long)]
        fee_schedule: Option<String>,
        #[arg(long)]
        notes: Option<String>,
        #[arg(long)]
        template: Option<String>,
    },
    /// Render contract to PDF
    Render(ContractRenderArgs),
    /// Update status: draft | sent | signed | active | expired | terminated
    Mark { number: String, status: String },
    /// Record a signature (auto-bumps status to "signed" when both sides done)
    Sign(SignArgs),
    /// Manage the clause set for a contract
    #[command(subcommand)]
    Clauses(ClauseCmd),
    /// Clone an existing contract as a new draft
    Duplicate {
        number: String,
        #[arg(long)]
        client: Option<String>,
        #[arg(long = "as")]
        r#as: Option<String>,
    },
    /// Delete a contract (draft only unless --force)
    #[command(alias = "rm")]
    Delete {
        number: String,
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ClauseCmd {
    /// List clauses currently attached to a contract
    #[command(alias = "ls")]
    List { number: String },
    /// Add a clause to a contract (pack default or custom from --from-file)
    Add {
        number: String,
        slug: String,
        /// Override heading
        #[arg(long)]
        heading: Option<String>,
        /// Custom body markdown — direct string
        #[arg(long)]
        body: Option<String>,
        /// Custom body markdown — read from file
        #[arg(long = "from-file")]
        from_file: Option<String>,
        /// Insert at zero-indexed position (default: end)
        #[arg(long)]
        position: Option<i64>,
    },
    /// Edit a clause that is currently attached
    Edit {
        number: String,
        slug: String,
        #[arg(long)]
        heading: Option<String>,
        #[arg(long)]
        body: Option<String>,
        #[arg(long = "from-file")]
        from_file: Option<String>,
    },
    /// Remove a clause
    #[command(alias = "rm")]
    Remove { number: String, slug: String },
    /// Move a clause to a new position
    Move {
        number: String,
        slug: String,
        position: i64,
    },
    /// Reset the contract's clause set to the pack default
    Reset { number: String },
}

// ─── Packs / Templates / Config / Skill ───────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum PackCmd {
    /// List available kind/pack combinations
    #[command(alias = "ls")]
    List,
    /// Show available clauses (slug + heading) for a kind + pack
    Show {
        kind: String,
        #[arg(long, default_value = "standard")]
        pack: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum TemplateCmd {
    #[command(alias = "ls")]
    List,
    /// Render a preview contract with synthetic data
    Preview {
        name: String,
        /// Which contract kind to preview (default consulting)
        #[arg(long, default_value = "consulting")]
        kind: String,
        #[arg(long, short)]
        out: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCmd {
    Show,
    Path,
    Set { key: String, value: String },
}

#[derive(Subcommand, Debug)]
pub enum SkillCmd {
    Install,
}
