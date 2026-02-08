mod cli;
mod domain;
mod services;
mod storage;
mod web;

use clap::Parser;
use cli::{AgentCommands, Cli, Commands, DbCommands, KbCommands, MailCommands};
use services::kb::{KnowledgeBaseService, KnowledgeBaseServiceImpl};
use services::kb::domain::LuhmannId;
use services::mail::{MailService, MailServiceImpl};
use storage::memory::InMemoryStorage;
use storage::postgres::PostgresStorage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if present (silently ignore errors if file doesn't exist)
    let _ = dotenvy::dotenv();
    
    let cli = Cli::parse();
    
    // Check for AGENT_OFFICE_URL first, then fall back to DATABASE_URL
    // This allows users to use .env files for configuration
    let database_url = std::env::var("AGENT_OFFICE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok();
    
    match cli.command {
        Commands::Db(db_cmd) => {
            handle_db_command(database_url, db_cmd).await?;
        }
        Commands::Kb(kb_cmd) => {
            if let Some(url) = database_url.clone() {
                let pool = sqlx::postgres::PgPool::connect(&url).await?;
                let storage = PostgresStorage::new(pool);
                let kb_service = KnowledgeBaseServiceImpl::new(storage);
                handle_kb_command(kb_service, kb_cmd).await?;
            } else {
                let storage = InMemoryStorage::new();
                let kb_service = KnowledgeBaseServiceImpl::new(storage);
                handle_kb_command(kb_service, kb_cmd).await?;
            }
        }
        Commands::Web { host, port } => {
            // Start the web server
            web::run_web_server(database_url, host, port).await?;
        }
        _ => {
            if let Some(url) = database_url {
                let pool = sqlx::postgres::PgPool::connect(&url).await?;
                let storage = PostgresStorage::new(pool.clone());
                let mail_service = MailServiceImpl::new(storage);
                let kb_storage = PostgresStorage::new(pool);
                let _kb_service = KnowledgeBaseServiceImpl::new(kb_storage);
                
                match cli.command {
                    Commands::Mail(mail_cmd) => handle_mail_command(mail_service, mail_cmd).await?,
                    Commands::Agent(agent_cmd) => handle_agent_command(mail_service, agent_cmd).await?,
                    _ => {}
                }
            } else {
                // Use in-memory storage
                let storage = InMemoryStorage::new();
                let mail_service = MailServiceImpl::new(storage.clone());
                let _kb_service = KnowledgeBaseServiceImpl::new(storage);
                
                match cli.command {
                    Commands::Mail(mail_cmd) => handle_mail_command(mail_service, mail_cmd).await?,
                    Commands::Agent(agent_cmd) => handle_agent_command(mail_service, agent_cmd).await?,
                    _ => {}
                }
            }
        }
    }
    
    Ok(())
}

async fn handle_db_command(
    database_url: Option<String>,
    cmd: DbCommands,
) -> anyhow::Result<()> {
    match cmd {
        DbCommands::Setup => {
            let url = database_url.ok_or_else(|| {
                anyhow::anyhow!("DATABASE_URL environment variable not set")
            })?;
            
            println!("Connecting to database...");
            let pool = sqlx::postgres::PgPool::connect(&url).await?;
            let storage = PostgresStorage::new(pool);
            
            println!("Setting up database tables...");
            storage.setup_tables().await.map_err(|e| {
                anyhow::anyhow!("Failed to setup database: {}", e)
            })?;
            
            println!("Database setup complete!");
            println!("Tables created: nodes, edges");
            println!("Indexes created for performance");
        }
    }
    Ok(())
}

async fn handle_mail_command(
    service: impl MailService,
    cmd: MailCommands,
) -> anyhow::Result<()> {
    match cmd {
        MailCommands::Recent { agent_id } => {
            let mailbox = service.get_agent_mailbox(agent_id.clone()).await?;
            let mails = service.get_recent_mail(mailbox.id, 24, 50).await?;
            if mails.is_empty() {
                println!("No recent mail for agent {}", agent_id);
            } else {
                println!("Recent mail for agent {} (last 24 hours):", agent_id);
                for mail in mails {
                    let status = if mail.read { "[Read]" } else { "[Unread]" };
                    let short_id = &mail.id.to_string()[..8];
                    match service.get_agent_by_mailbox(mail.from_mailbox_id).await {
                        Ok(sender) => println!("  {} [{}] from {}: {}", status, short_id, sender.name, mail.subject),
                        Err(_) => println!("  {} [{}]: {}", status, short_id, mail.subject),
                    }
                }
            }
        }
        MailCommands::Send { from, to, subject, body } => {
            service.send_agent_to_agent(from.clone(), to.clone(), subject.clone(), body).await?;
            println!("✉️  {} -> {}: {}", from, to, subject);
        }
        MailCommands::Inbox { agent_id } => {
            let mailbox = service.get_agent_mailbox(agent_id.clone()).await?;
            let mails = service.get_mailbox_inbox(mailbox.id).await?;
            if mails.is_empty() {
                println!("Inbox is empty for agent {}", agent_id);
            } else {
                println!("Inbox for agent {}:", agent_id);
                for mail in mails {
                    let status = if mail.read { "[Read]" } else { "[Unread]" };
                    let short_id = &mail.id.to_string()[..8];
                    match service.get_agent_by_mailbox(mail.from_mailbox_id).await {
                        Ok(sender) => println!("  {} [{}] from {}: {}", status, short_id, sender.name, mail.subject),
                        Err(_) => println!("  {} [{}]: {}", status, short_id, mail.subject),
                    }
                }
            }
        }
        MailCommands::Outbox { agent_id } => {
            let mailbox = service.get_agent_mailbox(agent_id.clone()).await?;
            let mails = service.get_mailbox_outbox(mailbox.id).await?;
            if mails.is_empty() {
                println!("Outbox is empty for agent {}", agent_id);
            } else {
                println!("Outbox for agent {}:", agent_id);
                for mail in mails {
                    let short_id = &mail.id.to_string()[..8];
                    match service.get_agent_by_mailbox(mail.to_mailbox_id).await {
                        Ok(recipient) => println!("  [{}] to {}: {}", short_id, recipient.name, mail.subject),
                        Err(_) => println!("  [{}]: {}", short_id, mail.subject),
                    }
                }
            }
        }
        MailCommands::Read { mail_id } => {
            let mail = service.mark_mail_as_read_by_short_id(&mail_id).await?;
            let sender = service.get_agent_by_mailbox(mail.from_mailbox_id).await?;
            println!("📧 Mail from {}: {}", sender.name, mail.subject);
            println!("   ID: {}", mail.id);
            println!("   Date: {}", mail.created_at.format("%Y-%m-%d %H:%M:%S"));
            println!();
            println!("{}", mail.body);
        }
        MailCommands::ShouldLook { agent_id } => {
            let (has_unread, mails) = service.check_unread_mail(agent_id.clone()).await?;
            if has_unread {
                println!("📬 Agent '{}' has {} unread message(s)", agent_id, mails.len());
                for mail in &mails {
                    match service.get_agent_by_mailbox(mail.from_mailbox_id).await {
                        Ok(sender) => println!("  [Unread] from {}: {}", sender.name, mail.subject),
                        Err(_) => println!("  [Unread]: {}", mail.subject),
                    }
                }
            } else {
                println!("📭 Agent '{}' has no unread mail", agent_id);
            }
        }
        MailCommands::Watch { agent_id, interval, bash } => {
            use tokio::time::{sleep, Duration};
            use std::process::Command;
            
            let _ = service.set_agent_status(agent_id.clone(), "online").await;
            println!("Agent '{}' is now online", agent_id);
            println!("Watching for new mail (checking every {} seconds)", interval);
            println!("Press Ctrl+C to stop");
            
            let ctrl_c = tokio::signal::ctrl_c();
            tokio::pin!(ctrl_c);
            let interval_duration = Duration::from_secs(interval);
            let mut running = true;
            
            while running {
                tokio::select! {
                    _ = &mut ctrl_c => {
                        println!("\nStopping watch...");
                        running = false;
                    }
                    _ = sleep(interval_duration) => {
                        let (has_unread, mails) = service.check_unread_mail(agent_id.clone()).await?;
                        if has_unread {
                            println!("\n📬 Found {} unread message(s)", mails.len());
                            for mail in &mails {
                                println!("  - {}", mail.subject);
                            }
                            println!("Executing: {}", bash);
                            let _ = Command::new("bash").arg("-c").arg(&bash).output();
                        }
                    }
                }
            }
            
            let _ = service.set_agent_status(agent_id.clone(), "offline").await;
            println!("Agent '{}' is now offline", agent_id);
        }
        MailCommands::Search { agent_id, query } => {
            let mailbox = service.get_agent_mailbox(agent_id.clone()).await?;
            let inbox = service.get_mailbox_inbox(mailbox.id).await?;
            let outbox = service.get_mailbox_outbox(mailbox.id).await?;
            
            let query_lower = query.to_lowercase();
            let mut results: Vec<_> = inbox.iter()
                .chain(outbox.iter())
                .filter(|m| {
                    m.subject.to_lowercase().contains(&query_lower) ||
                    m.body.to_lowercase().contains(&query_lower)
                })
                .collect();
            
            // Sort by date, newest first
            results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            
            if results.is_empty() {
                println!("No mail found matching '{}' for agent {}", query, agent_id);
            } else {
                println!("Found {} mail(s) matching '{}' for agent {}:", results.len(), query, agent_id);
                for mail in results {
                    let direction = if inbox.iter().any(|m| m.id == mail.id) { "📥" } else { "📤" };
                    let status = if mail.read { "Read" } else { "Unread" };
                    let short_id = &mail.id.to_string()[..8];
                    let other_agent = if direction == "📥" {
                        service.get_agent_by_mailbox(mail.from_mailbox_id).await.map(|a| a.name).unwrap_or_else(|_| "Unknown".to_string())
                    } else {
                        service.get_agent_by_mailbox(mail.to_mailbox_id).await.map(|a| a.name).unwrap_or_else(|_| "Unknown".to_string())
                    };
                    println!("  {} [{}] {} - {} (with {})", direction, status, short_id, mail.subject, other_agent);
                }
            }
        }
    }
    Ok(())
}

async fn handle_agent_command(
    service: impl MailService,
    cmd: AgentCommands,
) -> anyhow::Result<()> {
    match cmd {
        AgentCommands::Create { name } => {
            let agent = service.create_agent(name.clone()).await?;
            if agent.id != name {
                println!("Created agent: {} (ID: {})", name, agent.id);
            } else {
                println!("Created agent: {}", name);
            }
        }
        AgentCommands::List => {
            let agents = service.list_agents().await?;
            if agents.is_empty() {
                println!("No agents found");
            } else {
                println!("Agents:");
                for agent in agents {
                    if agent.id != agent.name {
                        println!("  - {} [{}] ({})", agent.name, agent.id, agent.status);
                    } else {
                        println!("  - {} ({})", agent.name, agent.status);
                    }
                }
            }
        }
        AgentCommands::Get { id } => {
            let agent = service.get_agent(id.clone()).await?;
            if agent.id != agent.name {
                println!("Agent: {} (ID: {})", agent.name, agent.id);
            } else {
                println!("Agent: {}", agent.name);
            }
            println!("Status: {}", agent.status);
            
            // Each agent has exactly one mailbox
            let _mailbox = service.get_agent_mailbox(id).await?;
            println!("Mailbox: ✓ (single mailbox per agent)");
        }
        AgentCommands::Status { id, status } => {
            let agent = service.set_agent_status(id.clone(), status.clone()).await?;
            println!("Updated agent '{}' status to: {}", id, agent.status);
        }
    }
    Ok(())
}

async fn handle_kb_command(
    service: impl KnowledgeBaseService,
    cmd: KbCommands,
) -> anyhow::Result<()> {
    match cmd {
        KbCommands::Init { agent_id, name } => {
            let kb = service.create_knowledge_base(agent_id, name).await?;
            println!("Created knowledge base: {}", kb.name);
            println!("Agent ID: {}", kb.agent_id);
        }
        KbCommands::Note { agent_id, title, content } => {
            let note = service.create_note(agent_id, title, content).await?;
            if let Some(ref lid) = note.luhmann_id {
                println!("Created note: [{}] {} (ID: {})", lid, note.title, note.id);
            } else {
                println!("Created note: {} (ID: {})", note.title, note.id);
            }
        }
        KbCommands::NoteWithId { agent_id, luhmann_id, title, content } => {
            let lid = LuhmannId::parse(&luhmann_id)
                .ok_or_else(|| anyhow::anyhow!("Invalid Luhmann ID: {}", luhmann_id))?;
            let note = service.create_note_with_luhmann_id(agent_id, lid, title, content).await?;
            println!("Created note: [{}] {} (ID: {})", 
                note.luhmann_id.as_ref().unwrap(), note.title, note.id);
        }
        KbCommands::Branch { agent_id, parent_note_id, title, content } => {
            let note = service.create_note_branch(agent_id, parent_note_id, title, content).await?;
            println!("Created branch: [{}] {} (ID: {})", 
                note.luhmann_id.as_ref().unwrap(), note.title, note.id);
            println!("Parent: {}", parent_note_id);
        }
        KbCommands::List { agent_id } => {
            let notes = service.list_agent_notes(agent_id.clone()).await?;
            if notes.is_empty() {
                println!("No notes found for agent {}", agent_id);
            } else {
                println!("Notes for agent {}:", agent_id);
                for note in notes {
                    let addr = note.luhmann_id.as_ref()
                        .map(|l| l.to_string())
                        .unwrap_or_else(|| note.id.to_string());
                    println!("  [{}] {} - {}", addr, note.title, note.id);
                }
            }
        }
        KbCommands::Get { note_id } => {
            let note = service.get_note(note_id).await?;
            let addr = note.luhmann_id.as_ref()
                .map(|l| format!("[{}]", l.to_string()))
                .unwrap_or_default();
            println!("Note {} {}", addr, note.id);
            println!("Title: {}", note.title);
            println!("Content: {}", note.content);
            println!("Tags: {:?}", note.tags);
            println!("Created: {}", note.created_at);
        }
        KbCommands::GetByLuhmann { agent_id, luhmann_id } => {
            let lid = LuhmannId::parse(&luhmann_id)
                .ok_or_else(|| anyhow::anyhow!("Invalid Luhmann ID: {}", luhmann_id))?;
            match service.get_note_by_luhmann_id(agent_id, &lid).await? {
                Some(note) => {
                    println!("Note [{}] {}", luhmann_id, note.id);
                    println!("Title: {}", note.title);
                    println!("Content: {}", note.content);
                }
                None => {
                    println!("No note found with Luhmann ID {}", luhmann_id);
                }
            }
        }
        KbCommands::Link { from, to, context, .. } => {
            service.link_notes(from, to, 
                services::kb::domain::LinkType::References, 
                context).await?;
            println!("Linked {} → {}", from, to);
        }
        KbCommands::Backlinks { note_id } => {
            let notes = service.get_backlinks(note_id).await?;
            if notes.is_empty() {
                println!("No backlinks found for {}", note_id);
            } else {
                println!("Notes linking to {}:", note_id);
                for note in notes {
                    let addr = note.luhmann_id.as_ref()
                        .map(|l| l.to_string())
                        .unwrap_or_default();
                    println!("  [{}] {} - {}", addr, note.title, note.id);
                }
            }
        }
        KbCommands::Related { note_id, depth } => {
            let notes = service.get_related_notes(note_id, depth).await?;
            if notes.is_empty() {
                println!("No related notes found within {} hops", depth);
            } else {
                println!("Notes within {} hops of {}:", depth, note_id);
                for note in notes {
                    let addr = note.luhmann_id.as_ref()
                        .map(|l| l.to_string())
                        .unwrap_or_default();
                    println!("  [{}] {}", addr, note.title);
                }
            }
        }
        KbCommands::Tree { agent_id, prefix } => {
            let lid = LuhmannId::parse(&prefix)
                .ok_or_else(|| anyhow::anyhow!("Invalid Luhmann ID prefix: {}", prefix))?;
            let notes = service.list_notes_by_luhmann_prefix(agent_id, &lid).await?;
            if notes.is_empty() {
                println!("No notes found under prefix {}", prefix);
            } else {
                println!("Notes under {}:", prefix);
                for note in notes {
                    let addr = note.luhmann_id.as_ref()
                        .map(|l| l.to_string())
                        .unwrap_or_default();
                    let indent = "  ".repeat(lid.level());
                    println!("{}{}[{}] {}", indent, addr, note.title, note.id);
                }
            }
        }
        KbCommands::Search { agent_id, query } => {
            let notes = service.search_notes(agent_id, &query).await?;
            if notes.is_empty() {
                println!("No notes matching '{}'", query);
            } else {
                println!("Notes matching '{}':", query);
                for note in notes {
                    let addr = note.luhmann_id.as_ref()
                        .map(|l| l.to_string())
                        .unwrap_or_default();
                    println!("  [{}] {}", addr, note.title);
                }
            }
        }
        KbCommands::Tag { note_id, tag } => {
            let note = service.add_tag(note_id, tag).await?;
            println!("Added tag to note {}: {:?}", note.id, note.tags);
        }
        KbCommands::Tags { agent_id } => {
            let tags = service.get_all_tags(agent_id).await?;
            if tags.is_empty() {
                println!("No tags found");
            } else {
                println!("Tags: {}", tags.join(", "));
            }
        }
        KbCommands::Graph { note_id, depth } => {
            let graph = service.get_note_graph(note_id, depth).await?;
            println!("Graph around {} (depth {}):", note_id, depth);
            println!("Notes: {}", graph.notes.len());
            println!("Links: {}", graph.links.len());
            for note in &graph.notes {
                let addr = note.luhmann_id.as_ref()
                    .map(|l| format!("[{}]", l))
                    .unwrap_or_default();
                println!("  {} {} - {}", addr, note.id, note.title);
            }
        }
    }
    Ok(())
}
