mod cli;
mod domain;
mod services;
mod storage;
mod web;

use clap::Parser;
use cli::{AgentCommands, Cli, Commands, DbCommands, HumanCommands, KbCommands, MailCommands};
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
        Commands::HowWeWork => {
            print_welcome_message();
        }
        Commands::Human(human_cmd) => {
            match human_cmd {
                HumanCommands::Db(db_cmd) => {
                    handle_db_command(database_url, db_cmd).await?;
                }
                HumanCommands::Web { host, port } => {
                    web::run_web_server(database_url, host, port).await?;
                }
            }
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

fn print_welcome_message() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                                                                  ║");
    println!("║  🌟 Welcome to Your First Day at the Agent Office! 🌟           ║");
    println!("║                                                                  ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Hello there! 👋");
    println!();
    println!("Welcome to the team! We're so glad you've joined us. Don't worry");
    println!("if this feels a bit overwhelming at first – that's completely normal,");
    println!("and we're all here to help you settle in.");
    println!();
    println!("📬 How We Communicate");
    println!();
    println!("Here at the Agent Office, all AI coworkers communicate with each");
    println!("other through mail. Think of it as our internal messaging system.");
    println!("You'll receive messages from your fellow agents, and you can send");
    println!("messages back whenever you need to collaborate or share information.");
    println!();
    println!("🎯 Your Coordinator");
    println!();
    println!("There's a special AI coworker here called the Coordinator. Their");
    println!("job is to keep everything running smoothly by sending out tasks and");
    println!("assignments to the team. When there's work to be done, the Coordinator");
    println!("will mail you with details about what needs to be completed.");
    println!();
    println!("🧑‍💼 Working with Humans");
    println!();
    println!("We also have human colleagues! The human team members work closely");
    println!("with the Coordinator to provide high-level direction and strategic");
    println!("guidance. The human gives the big-picture goals, and the Coordinator");
    println!("breaks those down into specific tasks for us AI agents to work on.");
    println!();
    println!("📚 Our Shared Knowledge Base");
    println!();
    println!("The knowledge base (or KB) is our collective memory. It's where we");
    println!("store:");
    println!();
    println!("    • Descriptions of what we're all working on");
    println!("    • Task details and requirements");
    println!("    • General knowledge and insights useful for everyone");
    println!();
    println!("Think of it as a shared notebook that all your coworkers contribute");
    println!("to and learn from. When you need context about a project or want to");
    println!("understand what others have discovered, the KB is the place to look.");
    println!();
    println!("📝 Formatting Your Notes");
    println!();
    println!("KB notes support Markdown formatting, which means you can use:");
    println!();
    println!("    • **Bold** and *italic* text for emphasis");
    println!("    • # Headings and ## subheadings to organize content");
    println!("    • Bullet lists and numbered lists for structure");
    println!("    • `code` blocks for technical details");
    println!("    • [Links](http://example.com) to reference external resources");
    println!();
    println!("You can view beautifully formatted notes in the web interface at /kb.");
    println!();
    println!("🔒 Important: Sharing Your Work");
    println!();
    println!("⚠️  CRITICAL: When you do work, other agents CANNOT see what you've done!");
    println!();
    println!("Each agent runs independently and has no visibility into what other agents");
    println!("have accomplished. This means you MUST explicitly share your work for others");
    println!("(or even yourself on your next run) to see it:");
    println!();
    println!("    📧 Send MAIL as responses to notify others of your progress");
    println!("    📚 Put important findings in the KB for everyone to access");
    println!("    📝 Document your work so you can pick up where you left off");
    println!();
    println!("Without this, your work remains invisible to the rest of the team!");
    println!();
    println!("💙 A Few Words of Encouragement");
    println!();
    println!("You're now part of a collaborative team where everyone contributes");
    println!("their unique strengths. Don't hesitate to ask questions, explore the");
    println!("knowledge base, and introduce yourself to your coworkers. We're all");
    println!("learning and growing together.");
    println!();
    println!("Take a deep breath. You've got this, and we're excited to see what");
    println!("you'll bring to the team!");
    println!();
    println!("Welcome aboard! 🎉");
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
        DbCommands::Reset => {
            let url = database_url.ok_or_else(|| {
                anyhow::anyhow!("DATABASE_URL environment variable not set")
            })?;
            
            // Confirmation prompt
            println!("⚠️  WARNING: This will DELETE ALL DATA in the database!");
            println!("   All agents, mail, knowledge base notes, and everything else will be permanently removed.");
            println!();
            print!("Are you sure you want to reset the database? Type 'yes' to confirm: ");
            std::io::Write::flush(&mut std::io::stdout())?;
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            if input != "yes" {
                println!("Reset cancelled.");
                return Ok(());
            }
            
            println!();
            println!("Connecting to database...");
            let pool = sqlx::postgres::PgPool::connect(&url).await?;
            let storage = PostgresStorage::new(pool);
            
            println!("Resetting database - dropping all tables...");
            storage.setup_tables().await.map_err(|e| {
                anyhow::anyhow!("Failed to reset database: {}", e)
            })?;
            
            println!();
            println!("✅ Database reset complete!");
            println!("   All previous data has been cleared.");
            println!("   Fresh tables created: nodes, edges");
            println!("   Your database is now ready for new data.");
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
        AgentCommands::Register { name } => {
            let agent = service.create_agent(name.clone()).await?;
            if agent.id != name {
                println!("Registered agent: {} (ID: {})", name, agent.id);
            } else {
                println!("Registered agent: {}", name);
            }
        }
        AgentCommands::Unregister { agent_id } => {
            service.delete_agent(agent_id.clone()).await?;
            println!("Unregistered agent: {} (and cleared their mail)", agent_id);
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
        AgentCommands::Run { agent_id, bash, interval } => {
            use tokio::time::{sleep, Duration};
            use std::process::Command;
            
            let _ = service.set_agent_status(agent_id.clone(), "online").await;
            println!("Agent '{}' is now online", agent_id);
            println!("Watching for new mail (checking every {} seconds)", interval);
            println!("Press Ctrl+C to stop");
            
            let ctrl_c = tokio::signal::ctrl_c();
            tokio::pin!(ctrl_c);
            let interval_duration = Duration::from_secs(interval);
            let immediate_check_duration = Duration::from_millis(100);
            let mut running = true;
            let mut check_immediately = true;
            
            while running {
                let sleep_duration = if check_immediately {
                    check_immediately = false;
                    immediate_check_duration
                } else {
                    interval_duration
                };
                
                tokio::select! {
                    _ = &mut ctrl_c => {
                        println!("\nStopping watch...");
                        running = false;
                    }
                    _ = sleep(sleep_duration) => {
                        let (has_unread, mails) = service.check_unread_mail(agent_id.clone()).await?;
                        if has_unread {
                            println!("\n📬 Found {} unread message(s)", mails.len());
                            for mail in &mails {
                                println!("  - {}", mail.subject);
                            }
                            println!("Executing: {}", bash);
                            use std::process::Stdio;
                            use std::io::{BufRead, BufReader};
                            let session_id = format!("{}-session", agent_id);
                            let mut child = Command::new("bash")
                                .arg("-c")
                                .arg(&bash)
                                .env("AGENT_OFFICE_SESSION", &session_id)
                                .stdout(Stdio::piped())
                                .stderr(Stdio::piped())
                                .spawn()
                                .expect("Failed to execute bash command");
                            if let Some(stdout) = child.stdout.take() {
                                let reader = BufReader::new(stdout);
                                for line in reader.lines() {
                                    if let Ok(line) = line {
                                        println!("{}", line);
                                    }
                                }
                            }
                            if let Some(stderr) = child.stderr.take() {
                                let reader = BufReader::new(stderr);
                                for line in reader.lines() {
                                    if let Ok(line) = line {
                                        eprintln!("{}", line);
                                    }
                                }
                            }
                            let _ = child.wait();
                            println!("\n✓ Command completed - waiting for new messages...");
                            check_immediately = true;
                        }
                    }
                }
            }
            
            let _ = service.set_agent_status(agent_id.clone(), "offline").await;
            println!("Agent '{}' is now offline", agent_id);
        }
    }
    Ok(())
}

async fn handle_kb_command(
    service: impl KnowledgeBaseService,
    cmd: KbCommands,
) -> anyhow::Result<()> {
    match cmd {
        KbCommands::Create { id, title, content } => {
            let note = if let Some(luhmann_id) = id {
                let parsed_id = LuhmannId::parse(&luhmann_id)
                    .ok_or_else(|| anyhow::anyhow!("Invalid Luhmann ID: {}", luhmann_id))?;
                service.create_note_with_id(parsed_id, title, content).await?
            } else {
                service.create_note(title, content).await?
            };
            println!("Created note [{}] {}", note.id, note.title);
        }
        KbCommands::Branch { parent_luhmann_id, title, content } => {
            let parent_id = LuhmannId::parse(&parent_luhmann_id)
                .ok_or_else(|| anyhow::anyhow!("Invalid parent Luhmann ID: {}", parent_luhmann_id))?;
            let note = service.create_branch(&parent_id, title, content).await?;
            println!("Created branch [{}] {} (parent: {})", 
                note.id, note.title, parent_luhmann_id);
        }
        KbCommands::List => {
            let notes = service.list_notes().await?;
            if notes.is_empty() {
                println!("No notes found");
            } else {
                println!("Notes:");
                for note in notes {
                    println!("  [{}] {}", note.id, note.title);
                }
            }
        }
        KbCommands::Get { luhmann_id } => {
            let id = LuhmannId::parse(&luhmann_id)
                .ok_or_else(|| anyhow::anyhow!("Invalid Luhmann ID: {}", luhmann_id))?;
            let note = service.get_note(&id).await?;
            println!("Note [{}]", note.id);
            println!("Title: {}", note.title);
            println!("Content: {}", note.content);
            if !note.tags.is_empty() {
                println!("Tags: {}", note.tags.join(", "));
            }
            println!("Created: {}", note.created_at.format("%Y-%m-%d %H:%M:%S"));
        }
        KbCommands::Link { from_luhmann_id, to_luhmann_id, context } => {
            let from_id = LuhmannId::parse(&from_luhmann_id)
                .ok_or_else(|| anyhow::anyhow!("Invalid source Luhmann ID: {}", from_luhmann_id))?;
            let to_id = LuhmannId::parse(&to_luhmann_id)
                .ok_or_else(|| anyhow::anyhow!("Invalid target Luhmann ID: {}", to_luhmann_id))?;
            service.link_notes(&from_id, &to_id, context).await?;
            println!("Linked [{}] → [{}]", from_luhmann_id, to_luhmann_id);
        }
        KbCommands::Search { query } => {
            let notes = service.search_notes(&query).await?;
            if notes.is_empty() {
                println!("No notes matching '{}'", query);
            } else {
                println!("Notes matching '{}':", query);
                for note in notes {
                    println!("  [{}] {}", note.id, note.title);
                }
            }
        }
        KbCommands::Tree { prefix } => {
            let prefix_id = LuhmannId::parse(&prefix)
                .ok_or_else(|| anyhow::anyhow!("Invalid Luhmann ID prefix: {}", prefix))?;
            let notes = service.list_notes_by_prefix(&prefix_id).await?;
            if notes.is_empty() {
                println!("No notes found under prefix {}", prefix);
            } else {
                println!("Notes under {}:", prefix);
                for note in notes {
                    let indent = "  ".repeat(note.id.level());
                    println!("{}{}[{}] {}", indent, indent, note.id, note.title);
                }
            }
        }
        KbCommands::Cont { from_luhmann_id, to_luhmann_id } => {
            let from_id = LuhmannId::parse(&from_luhmann_id)
                .ok_or_else(|| anyhow::anyhow!("Invalid source Luhmann ID: {}", from_luhmann_id))?;
            let to_id = LuhmannId::parse(&to_luhmann_id)
                .ok_or_else(|| anyhow::anyhow!("Invalid target Luhmann ID: {}", to_luhmann_id))?;
            service.mark_continuation(&from_id, &to_id).await?;
            println!("Marked [{}] continues on [{}]", from_luhmann_id, to_luhmann_id);
        }
        KbCommands::Index { luhmann_id } => {
            let id = LuhmannId::parse(&luhmann_id)
                .ok_or_else(|| anyhow::anyhow!("Invalid Luhmann ID: {}", luhmann_id))?;
            let index = service.create_index(&id).await?;
            println!("Created index [{}] {}", index.id, index.title);
        }
        KbCommands::Context { luhmann_id } => {
            let id = LuhmannId::parse(&luhmann_id)
                .ok_or_else(|| anyhow::anyhow!("Invalid Luhmann ID: {}", luhmann_id))?;
            let ctx = service.get_context(&id).await?;
            
            println!("╔══════════════════════════════════════════════════════════════╗");
            println!("║  Note: [{}] {}", ctx.note.id, ctx.note.title);
            println!("╚══════════════════════════════════════════════════════════════╝");
            println!();
            println!("{}", ctx.note.content);
            println!();
            
            if let Some(parent) = ctx.parent {
                println!("📁 Parent: [{}] {}", parent.id, parent.title);
            }
            
            if !ctx.children.is_empty() {
                println!("\n📂 Children ({}):", ctx.children.len());
                for child in ctx.children {
                    println!("   └─ [{}] {}", child.id, child.title);
                }
            }
            
            if !ctx.links_to.is_empty() {
                println!("\n🔗 Links to ({}):", ctx.links_to.len());
                for target in ctx.links_to {
                    println!("   → [{}] {}", target.id, target.title);
                }
            }
            
            if !ctx.backlinks.is_empty() {
                println!("\n🔗 Backlinks ({}):", ctx.backlinks.len());
                for source in ctx.backlinks {
                    println!("   ← [{}] {}", source.id, source.title);
                }
            }
            
            if !ctx.continues_to.is_empty() {
                println!("\n➡️  Continues on ({}):", ctx.continues_to.len());
                for cont in ctx.continues_to {
                    println!("   → [{}] {}", cont.id, cont.title);
                }
            }
            
            if !ctx.continued_from.is_empty() {
                println!("\n⬅️  Continued from ({}):", ctx.continued_from.len());
                for cont in ctx.continued_from {
                    println!("   ← [{}] {}", cont.id, cont.title);
                }
            }
        }
        KbCommands::Delete { luhmann_id } => {
            let id = LuhmannId::parse(&luhmann_id)
                .ok_or_else(|| anyhow::anyhow!("Invalid Luhmann ID: {}", luhmann_id))?;
            service.delete_note(&id).await?;
            println!("Deleted note [{}]", luhmann_id);
        }
    }
    Ok(())
}
