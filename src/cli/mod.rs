use clap::{Parser, Subcommand};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "agent-office")]
#[command(about = "A graph-based data structure tool for AI agents")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(subcommand)]
    Mail(MailCommands),
    #[command(subcommand)]
    Agent(AgentCommands),
    #[command(subcommand)]
    Db(DbCommands),
    #[command(subcommand)]
    Kb(KbCommands),
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
        #[arg(short, long)]
        agent_id: String,
    },
    /// View outbox (sent items) of an agent
    Outbox {
        #[arg(short, long)]
        agent_id: String,
    },
    /// Mark mail as read by short ID (first 8 chars of UUID)
    Read {
        /// Agent who owns the mail
        #[arg(short, long)]
        agent_id: String,
        /// Short mail ID (first 8 characters of UUID)
        mail_id: String,
    },
    /// Check if agent should look at their mail (has unread messages)
    ShouldLook {
        #[arg(short, long)]
        agent_id: String,
    },
    /// Watch for new mail and execute command when unread mail arrives
    Watch {
        #[arg(short, long)]
        agent_id: String,
        /// Interval in seconds between checks
        #[arg(short, long, default_value = "60")]
        interval: u64,
        /// Bash command to execute when unread mail is found
        #[arg(short, long)]
        bash: String,
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
    /// Create a new agent
    Create {
        #[arg(short, long)]
        name: String,
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
}

#[derive(Subcommand)]
pub enum DbCommands {
    /// Setup database tables (drops existing tables if they exist)
    Setup,
}

#[derive(Subcommand)]
pub enum KbCommands {
    /// Initialize a knowledge base for an agent
    Init {
        #[arg(short, long)]
        agent_id: String,
        #[arg(short, long)]
        name: String,
    },
    /// Create a new note with auto-generated Luhmann ID
    Note {
        #[arg(short, long)]
        agent_id: String,
        #[arg(short, long)]
        title: String,
        #[arg(short, long)]
        content: String,
    },
    /// Create a note with specific Luhmann ID (e.g., 1a2b)
    NoteWithId {
        #[arg(short, long)]
        agent_id: String,
        #[arg(short, long)]
        luhmann_id: String,
        #[arg(short, long)]
        title: String,
        #[arg(short, long)]
        content: String,
    },
    /// Branch from an existing note (create child)
    Branch {
        #[arg(short, long)]
        agent_id: String,
        #[arg(short, long)]
        parent_note_id: Uuid,
        #[arg(short, long)]
        title: String,
        #[arg(short, long)]
        content: String,
    },
    /// List all notes in an agent's knowledge base
    List {
        #[arg(short, long)]
        agent_id: String,
    },
    /// Get a specific note by ID
    Get {
        #[arg(short, long)]
        note_id: Uuid,
    },
    /// Get a note by its Luhmann address
    GetByLuhmann {
        #[arg(short, long)]
        agent_id: String,
        #[arg(short, long)]
        luhmann_id: String,
    },
    /// Link two notes together
    Link {
        #[arg(short, long)]
        from: Uuid,
        #[arg(short, long)]
        to: Uuid,
        #[arg(short, long)]
        context: Option<String>,
    },
    /// Show backlinks (notes that link to this note)
    Backlinks {
        #[arg(short, long)]
        note_id: Uuid,
    },
    /// Show related notes within N hops
    Related {
        #[arg(short, long)]
        note_id: Uuid,
        #[arg(short, long, default_value = "2")]
        depth: usize,
    },
    /// Show notes by Luhmann ID prefix
    Tree {
        #[arg(short, long)]
        agent_id: String,
        #[arg(short, long)]
        prefix: String,
    },
    /// Search notes
    Search {
        #[arg(short, long)]
        agent_id: String,
        #[arg(short, long)]
        query: String,
    },
    /// Add a tag to a note
    Tag {
        #[arg(short, long)]
        note_id: Uuid,
        #[arg(short, long)]
        tag: String,
    },
    /// List all tags for an agent
    Tags {
        #[arg(short, long)]
        agent_id: String,
    },
    /// Show the graph around a note
    Graph {
        #[arg(short, long)]
        note_id: Uuid,
        #[arg(short, long, default_value = "2")]
        depth: usize,
    },
}
