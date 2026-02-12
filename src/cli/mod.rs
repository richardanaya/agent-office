use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agent-office")]
#[command(about = "A pleasant set of tools for refined AI agents to get work done")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// A simple mailbox to communicate with your coworkers
    #[command(subcommand)]
    Mail(MailCommands),
    /// Find your coworkers, let your coworkers know your status, and register yourself as a coworker
    #[command(subcommand)]
    Agent(AgentCommands),
    /// A Zettelkasten knowledge base with Markdown support for all coworkers to share
    #[command(subcommand)]
    Kb(KbCommands),
    /// Human-only tools (not for AI agents)
    #[command(subcommand)]
    Human(HumanCommands),
    /// A warm welcome and guide for new AI agents
    HowWeWork,
}

/// Commands intended for human use only - these are tools for manual interaction
/// and should not be invoked by AI agents
#[derive(Subcommand)]
pub enum HumanCommands {
    #[command(subcommand)]
    Db(DbCommands),
    /// Start web interface
    Web {
        /// Host to bind to
        #[arg(short = 'H', long, default_value = "127.0.0.1")]
        host: String,
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}

#[derive(Subcommand)]
pub enum MailCommands {
    /// View recent mail for an agent (last 24 hours)
    Recent {
        /// Agent ID to view mail for
        agent_id: String,
    },
    /// Send mail from one agent to another (SIMPLE - uses agent names only!)
    Send {
        #[arg(short, long)]
        from: String,
        #[arg(short, long)]
        to: String,
        #[arg(short, long)]
        subject: String,
        #[arg(short, long)]
        body: String,
    },
    /// View inbox of an agent
    Inbox {
        /// Agent ID to view inbox for
        agent_id: String,
    },
    /// View outbox (sent items) of an agent
    Outbox {
        /// Agent ID to view outbox for
        agent_id: String,
    },
    /// Mark mail as read by short ID (first 8 chars of UUID)
    Read {
        /// Short mail ID (first 8 characters of UUID)
        mail_id: String,
    },
    /// Check if agent should look at their mail (has unread messages)
    ShouldLook {
        /// Agent ID to check
        agent_id: String,
    },
    /// Search mail by subject or body content
    Search {
        /// Agent ID to search mail for
        agent_id: String,
        /// Search query string (searches in subject and body)
        query: String,
    },
}

#[derive(Subcommand)]
pub enum AgentCommands {
    /// Register a new agent
    Register {
        #[arg(short, long)]
        name: String,
    },
    /// Unregister an agent (remove from the system)
    Unregister {
        /// Agent ID to remove
        agent_id: String,
    },
    /// List all agents
    List,
    /// Get agent details
    Get {
        #[arg(short, long)]
        id: String,
    },
    /// Set agent status (online, offline, away, etc.)
    Status {
        #[arg(short, long)]
        id: String,
        #[arg(short, long)]
        status: String,
    },
    /// Run an agent in watch mode - continuously monitor for new mail and execute command when found
    Run {
        /// Agent ID to run
        agent_id: String,
        /// Bash command to execute when unread mail is found
        bash: String,
        /// Interval in seconds between checks
        #[arg(short, long, default_value = "60")]
        interval: u64,
    },
}

#[derive(Subcommand)]
pub enum DbCommands {
    /// Setup database tables (drops existing tables if they exist)
    Setup,
    /// Reset the entire database - drops all data and recreates fresh tables
    Reset,
}

/// Simplified KB commands - shared knowledge base, only Luhmann IDs
#[derive(Subcommand)]
pub enum KbCommands {
    /// Create a new note (auto-generates ID unless --id specified)
    /// Usage: kb create "Title" "Content"  OR  kb create --id 1a "Title" "Content"
    Create {
        /// Optional Luhmann ID (e.g., 1a, 1a1). If not provided, auto-generates next available ID
        #[arg(short, long)]
        id: Option<String>,
        /// Note title
        title: String,
        /// Note content
        content: String,
    },
    /// Create a child note (branch) from a parent
    /// Usage: kb branch 1 "Child Title" "Content"
    Branch {
        /// Parent Luhmann ID
        parent_luhmann_id: String,
        /// Note title
        title: String,
        /// Note content
        content: String,
    },
    /// List all notes (sorted by Luhmann ID)
    List,
    /// Get a specific note by Luhmann ID
    /// Usage: kb get 1a
    Get {
        /// Luhmann ID
        luhmann_id: String,
    },
    /// Link two notes together
    /// Usage: kb link 1a 1b
    Link {
        /// Source Luhmann ID
        from_luhmann_id: String,
        /// Target Luhmann ID
        to_luhmann_id: String,
        /// Optional context for the link
        #[arg(short, long)]
        context: Option<String>,
    },
    /// Search notes
    /// Usage: kb search "query"
    Search {
        /// Search query
        query: String,
    },
    /// Show notes by Luhmann ID prefix
    /// Usage: kb tree 1a
    Tree {
        /// Luhmann ID prefix
        prefix: String,
    },
    /// Mark that note A continues on note B (linear chain)
    /// Usage: kb cont 1a 1b
    Cont {
        /// Source Luhmann ID (the note that continues)
        from_luhmann_id: String,
        /// Target Luhmann ID (the continuation)
        to_luhmann_id: String,
    },
    /// Create an index card listing all children of a note
    /// Usage: kb index 1a
    Index {
        /// Luhmann ID to create index for
        luhmann_id: String,
    },
    /// Show full context of a note (parent, children, links, continuations, backlinks)
    /// Usage: kb context 1a
    Context {
        /// Luhmann ID to show context for
        luhmann_id: String,
    },
    /// Delete a note by Luhmann ID
    /// Usage: kb delete 1a
    Delete {
        /// Luhmann ID of the note to delete
        luhmann_id: String,
    },
}
