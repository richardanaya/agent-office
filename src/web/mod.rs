use axum::{
    extract::Path,
    response::Html,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;

pub mod templates;
mod schedules;
use schedules::{agent_schedule_view, create_schedule, update_schedule, delete_schedule, toggle_schedule};

use crate::services::mail::{MailService, MailServiceImpl};
use crate::services::kb::{KnowledgeBaseService, KnowledgeBaseServiceImpl};
use crate::services::kb::domain::LuhmannId;
// Schedule handlers are in schedules module
use crate::storage::{memory::InMemoryStorage, postgres::PostgresStorage};

/// Render markdown content to HTML using pulldown-cmark.
fn render_markdown(content: &str) -> String {
    use pulldown_cmark::{Parser, Options, html};
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(content, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

/// Strip markdown formatting to produce a plain-text preview, truncated at
/// a word boundary near `max_chars`.
fn plain_text_preview(content: &str, max_chars: usize) -> String {
    use pulldown_cmark::{Parser, Event, Tag, TagEnd};
    let parser = Parser::new(content);
    let mut text = String::new();
    for event in parser {
        match event {
            Event::Text(t) | Event::Code(t) => text.push_str(&t),
            Event::SoftBreak | Event::HardBreak => text.push(' '),
            Event::Start(Tag::Paragraph) => {
                if !text.is_empty() {
                    text.push(' ');
                }
            }
            Event::End(TagEnd::Paragraph) => text.push(' '),
            _ => {}
        }
        if text.len() > max_chars + 50 {
            break; // we have enough
        }
    }
    // Truncate at word boundary
    let trimmed = text.trim();
    if trimmed.len() <= max_chars {
        return trimmed.to_string();
    }
    // Find the last space before max_chars
    match trimmed[..max_chars].rfind(' ') {
        Some(pos) => format!("{}...", &trimmed[..pos]),
        None => format!("{}...", &trimmed[..max_chars]),
    }
}

pub async fn run_web_server(
    database_url: Option<String>,
    host: String,
    port: u16,
) -> anyhow::Result<()> {
    let app = create_router(database_url);
    
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    println!("🌐 Starting web server on http://{}", addr);
    println!("📱 Open your browser and navigate to http://{}", addr);
    println!("Press Ctrl+C to stop");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

fn create_router(database_url: Option<String>) -> Router {
    use std::sync::Arc;
    let db_url = Arc::new(database_url.clone());
    let db_url2 = Arc::new(database_url.clone());
    let db_url3 = Arc::new(database_url.clone());
    let db_url4 = Arc::new(database_url.clone());
    let db_url5 = Arc::new(database_url.clone());
    let db_url6 = Arc::new(database_url.clone());
    let db_url7 = Arc::new(database_url.clone());
    let db_url8 = Arc::new(database_url.clone());
    let db_url9 = Arc::new(database_url.clone());
    let db_url10 = Arc::new(database_url.clone());
    let db_url11 = Arc::new(database_url.clone());
    
    Router::new()
        // Dashboard / Home
        .route("/", get({
            let db = db_url.clone();
            move || dashboard((*db).clone())
        }))
        
        // Agents
        .route("/agents", get({
            let db = db_url.clone();
            move || list_agents((*db).clone())
        }))
        
        // Inbox view
        .route("/mail/inbox/{agent_id}", get({
            let db = db_url2.clone();
            move |Path(agent_id): Path<String>| inbox_view((*db).clone(), agent_id)
        }))
        .route("/mail/{mail_id}/read", post({
            let db = db_url2.clone();
            move |Path(mail_id): Path<String>| mark_mail_read((*db).clone(), mail_id)
        }))
        .route("/mail/inbox/{agent_id}/read-all", post({
            let db = db_url2.clone();
            move |Path(agent_id): Path<String>| mark_all_mail_read((*db).clone(), agent_id)
        }))
        
        // Outbox view
        .route("/mail/outbox/{agent_id}", get({
            let db = db_url8.clone();
            move |Path(agent_id): Path<String>| outbox_view((*db).clone(), agent_id)
        }))
        
        // Update agent status
        .route("/agents/{agent_id}/status", post({
            let db = db_url3.clone();
            move |Path(agent_id): Path<String>| set_agent_status((*db).clone(), agent_id)
        }))
        
        // Send mail to agent
        .route("/mail/send", post({
            let db = db_url7.clone();
            move |body: axum::body::Bytes| send_mail((*db).clone(), body)
        }))
        
        // Schedule management
        .route("/agents/{agent_id}/schedule", get({
            let db = db_url9.clone();
            move |Path(agent_id): Path<String>| agent_schedule_view((*db).clone(), agent_id)
        }))
        .route("/agents/{agent_id}/schedule", post({
            let db = db_url10.clone();
            move |Path(agent_id): Path<String>, body: axum::body::Bytes| create_schedule((*db).clone(), agent_id, body)
        }))
        .route("/schedules/{schedule_id}/toggle", post({
            let db = db_url11.clone();
            move |Path(schedule_id): Path<String>| toggle_schedule((*db).clone(), schedule_id)
        }))
        .route("/schedules/{schedule_id}/update", post({
            let db = db_url11.clone();
            move |Path(schedule_id): Path<String>, body: axum::body::Bytes| update_schedule((*db).clone(), schedule_id, body)
        }))
        .route("/schedules/{schedule_id}/delete", post({
            let db = db_url11.clone();
            move |Path(schedule_id): Path<String>| delete_schedule((*db).clone(), schedule_id)
        }))
        
        // KB - Knowledge Base
        .route("/kb", get({
            let db = db_url4.clone();
            move || kb_list_notes((*db).clone())
        }))
        
        // KB - View specific note
        .route("/kb/note/{note_id}", get({
            let db = db_url5.clone();
            move |Path(note_id): Path<String>| kb_view_note((*db).clone(), note_id)
        }))
        
        // KB - Tree view by prefix
        .route("/kb/tree/{prefix}", get({
            let db = db_url6.clone();
            move |Path(prefix): Path<String>| kb_tree_view((*db).clone(), prefix)
        }))
        
        // Static assets
        .route("/static/style.css", get(|| async {
            ([("content-type", "text/css")], templates::CSS)
        }))
}

// Dashboard / Home - Show agents with their mailboxes
async fn dashboard(database_url: Option<String>) -> Html<String> {
    let agents = if let Some(url) = database_url {
        let pool = match sqlx::postgres::PgPool::connect(&url).await {
            Ok(p) => p,
            Err(_) => return Html(templates::error_page("Failed to connect to database")),
        };
        let storage = PostgresStorage::new(pool);
        let service = MailServiceImpl::new(storage);
        
        match service.list_agents().await {
            Ok(agents) => agents,
            Err(_) => return Html(templates::error_page("Failed to load agents")),
        }
    } else {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        match service.list_agents().await {
            Ok(agents) => agents,
            Err(_) => return Html(templates::error_page("Failed to load agents")),
        }
    };
    
    let mut agent_cards = String::new();
    for agent in &agents {
        let status_class = match agent.status.as_str() {
            "online" => "online",
            "busy" => "busy",
            _ => "offline",
        };
        
        // Show inbox, outbox, and schedule links
        let mailbox_list = format!(
            r#"<div class="mailbox-item">
                <a href="/mail/inbox/{}" class="btn btn-sm">📥 Inbox</a>
                <a href="/mail/outbox/{}" class="btn btn-sm">📤 Outbox</a>
                <a href="/agents/{}/schedule" class="btn btn-sm">⏰ Schedules</a>
            </div>"#,
            agent.id, agent.id, agent.id
        );
        
        // Quick status toggle button (only show if not already offline)
        let status_button = if agent.status != "offline" {
            format!(
                "<button class=\"btn btn-sm btn-offline\" \
                    hx-post=\"/agents/{}/status\" \
                    hx-target=\"#agent-status-{}\" \
                    hx-swap=\"outerHTML\"> \
                    Set Offline \
                </button>",
                agent.id, agent.id
            )
        } else {
            String::new()
        };
        
        agent_cards.push_str(&format!(
            r#"<div class="agent-card">
                <div class="agent-info">
                    <h3>{}</h3>
                    <span class="status {}" id="agent-status-{}">{}</span>
                    {}
                </div>
                <div class="agent-mailboxes">
                    <h4>Mailboxes</h4>
                    {}
                </div>
            </div>"#,
            agent.name, status_class, agent.id, agent.status, status_button, mailbox_list
        ));
    }
    
    let content = format!(
        r##"
        <!-- Send Message Form -->
        <div class="send-message-card">
            <h3>Send Message to Agent</h3>
            <form id="send-mail-form" 
                  hx-post="/mail/send" 
                  hx-target="#send-result"
                  hx-swap="innerHTML"
                  onsubmit="saveFormFields()">
                <div class="form-row">
                    <div class="form-group">
                        <label for="send-to">To (Agent ID)</label>
                        <input type="text" id="send-to" name="to" placeholder="agent-name" required>
                    </div>
                    <div class="form-group">
                        <label for="send-from">From (Your ID)</label>
                        <input type="text" id="send-from" name="from" placeholder="human">
                    </div>
                </div>
                <div class="form-group">
                    <label for="send-subject">Subject</label>
                    <input type="text" id="send-subject" name="subject" placeholder="Message subject (optional)">
                </div>
                <div class="form-group">
                    <label for="send-body">Message</label>
                    <textarea id="send-body" name="body" rows="3" placeholder="Enter your message..." required></textarea>
                </div>
                <div class="form-actions">
                    <button type="submit" class="btn btn-primary">Send Message</button>
                    <span id="send-result"></span>
                </div>
            </form>
        </div>
        
        <script>
            // Load saved form fields from localStorage
            function loadFormFields() {{
                const savedTo = localStorage.getItem('send-mail-to');
                const savedFrom = localStorage.getItem('send-mail-from');
                if (savedTo) document.getElementById('send-to').value = savedTo;
                if (savedFrom) document.getElementById('send-from').value = savedFrom;
            }}
            
            // Save form fields to localStorage
            function saveFormFields() {{
                const to = document.getElementById('send-to').value;
                const from = document.getElementById('send-from').value;
                localStorage.setItem('send-mail-to', to);
                localStorage.setItem('send-mail-from', from);
            }}
            
            // Load on page load
            document.addEventListener('DOMContentLoaded', loadFormFields);
        </script>
        
        <h2>Dashboard <span class="section-count">{} agents</span></h2>
        <div class="agent-list">
            {}
        </div>
        "##,
        agents.len(),
        if agent_cards.is_empty() {
            "<p class='empty-state'>No agents registered yet</p>".to_string()
        } else {
            agent_cards
        }
    );
    
    Html(templates::wrap_content(content))
}

// List all agents
async fn list_agents(database_url: Option<String>) -> Html<String> {
    let agents = if let Some(url) = database_url {
        let pool = match sqlx::postgres::PgPool::connect(&url).await {
            Ok(p) => p,
            Err(_) => return Html(templates::error_page("Failed to connect to database")),
        };
        let storage = PostgresStorage::new(pool);
        let service = MailServiceImpl::new(storage);
        
        match service.list_agents().await {
            Ok(agents) => agents,
            Err(_) => return Html(templates::error_page("Failed to load agents")),
        }
    } else {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        match service.list_agents().await {
            Ok(agents) => agents,
            Err(_) => return Html(templates::error_page("Failed to load agents")),
        }
    };
    
    let mut agent_rows = String::new();
    for agent in &agents {
        let status_class = match agent.status.as_str() {
            "online" => "online",
            "busy" => "busy",
            _ => "offline",
        };
        
        agent_rows.push_str(&format!(
            r#"<tr>
                <td><strong>{}</strong></td>
                <td><span class="status {}">{}</span></td>
                <td><a href="/agents/{}/schedule" class="btn btn-sm">⏰ Schedules</a></td>
            </tr>"#,
            agent.name, status_class, agent.status, agent.id
        ));
    }
    
    let content = format!(
        r#"
        <h2>Agents <span class="section-count">{} total</span></h2>
        <table class="data-table">
            <thead>
                <tr>
                    <th>Name</th>
                    <th>Status</th>
                    <th>Actions</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>
        "#,
        agents.len(),
        if agent_rows.is_empty() {
            "<tr><td colspan=\"3\" class=\"empty-state\">No agents registered</td></tr>".to_string()
        } else {
            agent_rows
        }
    );
    
    Html(templates::wrap_content(content))
}

// KB - List all notes
async fn kb_list_notes(database_url: Option<String>) -> Html<String> {
    let notes = if let Some(url) = database_url {
        let pool = match sqlx::postgres::PgPool::connect(&url).await {
            Ok(p) => p,
            Err(_) => return Html(templates::error_page("Failed to connect to database")),
        };
        let storage = PostgresStorage::new(pool);
        let service = KnowledgeBaseServiceImpl::new(storage);
        
        match service.list_notes().await {
            Ok(notes) => notes,
            Err(_) => return Html(templates::error_page("Failed to load notes")),
        }
    } else {
        let storage = InMemoryStorage::new();
        let service = KnowledgeBaseServiceImpl::new(storage);
        
        match service.list_notes().await {
            Ok(notes) => notes,
            Err(_) => return Html(templates::error_page("Failed to load notes")),
        }
    };
    
    let mut notes_html = String::new();
    for note in &notes {
        let depth = note.id.level().saturating_sub(1); // 0 for root notes
        let indent_px = depth * 24; // indent per level

        // Build tags HTML
        let tags_html = if note.tags.is_empty() {
            String::new()
        } else {
            let badges: Vec<String> = note.tags.iter()
                .map(|t| format!(r#"<span class="tag-badge">{}</span>"#, t))
                .collect();
            format!(r#"<div class="note-tags">{}</div>"#, badges.join(""))
        };

        // Smart plain-text preview from markdown content
        let preview = plain_text_preview(&note.content, 240);

        // Relative date
        let date_str = note.created_at.format("%b %d").to_string();

        notes_html.push_str(&format!(
            r#"<a href="/kb/note/{id}" class="note-card" style="margin-left: {indent}px;">
                <div class="note-header">
                    <span class="note-id">{id}</span>
                    <span class="note-title">{title}</span>
                    <span class="note-date">{date}</span>
                </div>
                <div class="note-preview">{preview}</div>
                {tags}
                <div class="note-meta">
                    <span class="note-depth">depth {depth}</span>
                    <span class="note-tree-link" onclick="event.preventDefault(); window.location='/kb/tree/{id}';">tree</span>
                </div>
            </a>"#,
            id = note.id,
            indent = indent_px,
            title = note.title,
            date = date_str,
            preview = preview,
            tags = tags_html,
            depth = note.id.level(),
        ));
    }
    
    let content = format!(
        r#"
        <div class="page-header">
            <h2>Knowledge Base</h2>
            <div class="header-actions">
                <span class="note-count">{count} {noun}</span>
            </div>
        </div>
        <div class="notes-list">
            {notes}
        </div>
        "#,
        count = notes.len(),
        noun = if notes.len() == 1 { "note" } else { "notes" },
        notes = if notes_html.is_empty() {
            "<p class='empty-state'>No notes yet. Use <code>kb create</code> to add notes.</p>".to_string()
        } else {
            notes_html
        }
    );
    
    Html(templates::wrap_content(content))
}

// KB - View specific note with full context
async fn kb_view_note(database_url: Option<String>, note_id: String) -> Html<String> {
    let id = match LuhmannId::parse(&note_id) {
        Some(id) => id,
        None => return Html(templates::error_page(&format!("Invalid Luhmann ID: {}", note_id))),
    };
    
    let (note, children, parent, links, backlinks) = if let Some(url) = database_url {
        let pool = match sqlx::postgres::PgPool::connect(&url).await {
            Ok(p) => p,
            Err(_) => return Html(templates::error_page("Failed to connect to database")),
        };
        let storage = PostgresStorage::new(pool);
        let service = KnowledgeBaseServiceImpl::new(storage);
        
        let note = match service.get_note(&id).await {
            Ok(n) => n,
            Err(_) => return Html(templates::error_page(&format!("Note '{}' not found", note_id))),
        };
        
        // Get children
        let all_notes = match service.list_notes().await {
            Ok(n) => n,
            Err(_) => vec![],
        };
        let children: Vec<_> = all_notes.iter()
            .filter(|n| n.id.parent().as_ref() == Some(&id))
            .cloned()
            .collect();
        
        // Get parent
        let parent = if let Some(parent_id) = id.parent() {
            service.get_note(&parent_id).await.ok()
        } else {
            None
        };
        
        // Get links
        let links = match service.get_links(&id).await {
            Ok(l) => {
                let mut linked_notes = vec![];
                for link in l {
                    if let Ok(target) = service.get_note(&link.to_note_id).await {
                        linked_notes.push(target);
                    }
                }
                linked_notes
            },
            Err(_) => vec![],
        };
        
        // Get backlinks via context
        let ctx = match service.get_context(&id).await {
            Ok(c) => c.backlinks,
            Err(_) => vec![],
        };
        
        (note, children, parent, links, ctx)
    } else {
        let storage = InMemoryStorage::new();
        let service = KnowledgeBaseServiceImpl::new(storage);
        
        let note = match service.get_note(&id).await {
            Ok(n) => n,
            Err(_) => return Html(templates::error_page(&format!("Note '{}' not found", note_id))),
        };
        
        // Get children
        let all_notes = match service.list_notes().await {
            Ok(n) => n,
            Err(_) => vec![],
        };
        let children: Vec<_> = all_notes.iter()
            .filter(|n| n.id.parent().as_ref() == Some(&id))
            .cloned()
            .collect();
        
        // Get parent
        let parent = if let Some(parent_id) = id.parent() {
            service.get_note(&parent_id).await.ok()
        } else {
            None
        };
        
        // Get links
        let links = match service.get_links(&id).await {
            Ok(l) => {
                let mut linked_notes = vec![];
                for link in l {
                    if let Ok(target) = service.get_note(&link.to_note_id).await {
                        linked_notes.push(target);
                    }
                }
                linked_notes
            },
            Err(_) => vec![],
        };
        
        // Get backlinks via context
        let ctx = match service.get_context(&id).await {
            Ok(c) => c.backlinks,
            Err(_) => vec![],
        };
        
        (note, children, parent, links, ctx)
    };
    
    // Build relationships HTML
    let mut relations_html = String::new();
    
    if let Some(p) = parent {
        relations_html.push_str(&format!(
            r#"<div class="relation-section">
                <h4>📁 Parent</h4>
                <a href="/kb/note/{}" class="relation-link">[{}] {}</a>
            </div>"#,
            p.id, p.id, p.title
        ));
    }
    
    if !children.is_empty() {
        relations_html.push_str(r#"<div class="relation-section"><h4>📂 Children</h4>"#);
        for child in &children {
            relations_html.push_str(&format!(
                r#"<a href="/kb/note/{}" class="relation-link">└─ [{}] {}</a>"#,
                child.id, child.id, child.title
            ));
        }
        relations_html.push_str("</div>");
    }
    
    if !links.is_empty() {
        relations_html.push_str(r#"<div class="relation-section"><h4>🔗 Links To</h4>"#);
        for link in &links {
            relations_html.push_str(&format!(
                r#"<a href="/kb/note/{}" class="relation-link">→ [{}] {}</a>"#,
                link.id, link.id, link.title
            ));
        }
        relations_html.push_str("</div>");
    }
    
    if !backlinks.is_empty() {
        relations_html.push_str(r#"<div class="relation-section"><h4>🔗 Backlinks</h4>"#);
        for backlink in &backlinks {
            relations_html.push_str(&format!(
                r#"<a href="/kb/note/{}" class="relation-link">← [{}] {}</a>"#,
                backlink.id, backlink.id, backlink.title
            ));
        }
        relations_html.push_str("</div>");
    }
    
    // Build tags for detail page
    let tags_html = if note.tags.is_empty() {
        String::new()
    } else {
        let badges: Vec<String> = note.tags.iter()
            .map(|t| format!(r#"<span class="tag-badge">{}</span>"#, t))
            .collect();
        format!(r#"<div class="note-tags-detail">{}</div>"#, badges.join(""))
    };

    // Build breadcrumb with full ancestry
    let mut breadcrumb_parts = vec![r#"<a href="/kb">KB</a>"#.to_string()];
    {
        // Walk up the ancestors and push each one
        let mut ancestors = Vec::new();
        let mut current = id.clone();
        while let Some(pid) = current.parent() {
            ancestors.push(pid.clone());
            current = pid;
        }
        ancestors.reverse();
        for anc in &ancestors {
            breadcrumb_parts.push(format!(
                r#"<a href="/kb/note/{}">{}</a>"#,
                anc, anc
            ));
        }
    }
    breadcrumb_parts.push(format!("<span>{}</span>", note_id));
    let breadcrumb = breadcrumb_parts.join(r#" <span class="bc-sep">/</span> "#);

    // Render markdown content
    let rendered_content = render_markdown(&note.content);

    let content = format!(
        r#"
        <div class="note-detail">
            <div class="note-breadcrumb">
                {breadcrumb}
            </div>
            <h1 class="note-title-large">{title}</h1>
            <div class="note-meta-bar-top">
                <span class="note-id-detail">{note_id}</span>
                <span class="note-date-detail">{date}</span>
                {tags}
            </div>
            <article class="note-content-full prose">
                {content}
            </article>
            <div class="note-meta-bar">
                <span>Last updated {updated}</span>
                <a href="/kb/tree/{note_id}" class="btn btn-sm">View in tree</a>
            </div>
        </div>
        <div class="note-relationships">
            {relations}
        </div>
        "#,
        breadcrumb = breadcrumb,
        title = note.title,
        note_id = note_id,
        date = note.created_at.format("%b %d, %Y"),
        tags = tags_html,
        content = rendered_content,
        updated = note.updated_at.format("%b %d, %Y at %H:%M"),
        relations = if relations_html.is_empty() {
            "<p class='empty-state'>No relationships yet.</p>".to_string()
        } else {
            relations_html
        }
    );
    
    Html(templates::wrap_content(content))
}

// KB - Tree view by prefix
async fn kb_tree_view(database_url: Option<String>, prefix: String) -> Html<String> {
    let prefix_id = match LuhmannId::parse(&prefix) {
        Some(id) => id,
        None => return Html(templates::error_page(&format!("Invalid prefix: {}", prefix))),
    };
    
    let (notes_in_tree, parent_note) = if let Some(url) = database_url {
        let pool = match sqlx::postgres::PgPool::connect(&url).await {
            Ok(p) => p,
            Err(_) => return Html(templates::error_page("Failed to connect to database")),
        };
        let storage = PostgresStorage::new(pool);
        let service = KnowledgeBaseServiceImpl::new(storage);
        
        let all_notes = match service.list_notes().await {
            Ok(n) => n,
            Err(_) => return Html(templates::error_page("Failed to load notes")),
        };
        
        // Filter notes that are in this tree
        let notes_in_tree: Vec<_> = all_notes.iter()
            .filter(|n| n.id.to_string().starts_with(&prefix))
            .cloned()
            .collect();
        
        // Get parent note if exists
        let parent = if let Some(parent_id) = prefix_id.parent() {
            service.get_note(&parent_id).await.ok()
        } else {
            None
        };
        
        (notes_in_tree, parent)
    } else {
        let storage = InMemoryStorage::new();
        let service = KnowledgeBaseServiceImpl::new(storage);
        
        let all_notes = match service.list_notes().await {
            Ok(n) => n,
            Err(_) => return Html(templates::error_page("Failed to load notes")),
        };
        
        let notes_in_tree: Vec<_> = all_notes.iter()
            .filter(|n| n.id.to_string().starts_with(&prefix))
            .cloned()
            .collect();
        
        let parent = if let Some(parent_id) = prefix_id.parent() {
            service.get_note(&parent_id).await.ok()
        } else {
            None
        };
        
        (notes_in_tree, parent)
    };
    
    // Build tree visualization
    let mut tree_html = String::new();
    
    if let Some(parent) = parent_note {
        tree_html.push_str(&format!(
            r#"<div class="tree-level parent-level">
                <a href="/kb/note/{}" class="tree-node parent-node">📁 [{}] {}</a>
            </div>"#,
            parent.id, parent.id, parent.title
        ));
    }
    
    tree_html.push_str(r#"<div class="tree-level current-level">"#);
    for note in &notes_in_tree {
        let is_current = note.id.to_string() == prefix;
        let node_class = if is_current { "tree-node current-node" } else { "tree-node" };
        let icon = if note.id.to_string().len() > prefix.len() { "📄" } else { "📂" };
        tree_html.push_str(&format!(
            r#"<a href="/kb/note/{}" class="{}">{} [{}] {}</a>"#,
            note.id, node_class, icon, note.id, note.title
        ));
    }
    tree_html.push_str("</div>");
    
    let content = format!(
        r#"
        <div class="tree-view">
            <div class="tree-header">
                <h2>🌳 Tree View: {}</h2>
                <a href="/kb" class="btn btn-sm">← Back to All Notes</a>
            </div>
            <div class="tree-structure">
                {}
            </div>
            <div class="tree-stats">
                <span>{} notes in this branch</span>
            </div>
        </div>
        "#,
        prefix,
        tree_html,
        notes_in_tree.len()
    );
    
    Html(templates::wrap_content(content))
}

// Set agent status to offline
async fn set_agent_status(database_url: Option<String>, agent_id: String) -> Html<String> {
    let result = if let Some(url) = database_url {
        let pool = match sqlx::postgres::PgPool::connect(&url).await {
            Ok(p) => p,
            Err(_) => return Html(templates::error_page("Failed to connect to database")),
        };
        let storage = PostgresStorage::new(pool);
        let service = MailServiceImpl::new(storage);
        
        service.set_agent_status(agent_id, "offline").await
    } else {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        service.set_agent_status(agent_id, "offline").await
    };
    
    match result {
        Ok(agent) => {
            // Return a small HTML fragment for HTMX to swap
            let status_class = "offline";
            Html(format!(
                r#"<span class="status {}" id="agent-status-{}">{}</span>"#,
                status_class, agent.id, agent.status
            ))
        }
        Err(_) => Html(templates::error_page("Failed to update agent status")),
    }
}

// Inbox view - Show mail for an agent
async fn inbox_view(database_url: Option<String>, agent_id: String) -> Html<String> {
    let (inbox_mail, agent_name) = if let Some(url) = database_url {
        let pool = match sqlx::postgres::PgPool::connect(&url).await {
            Ok(p) => p,
            Err(_) => return Html(templates::error_page("Failed to connect to database")),
        };
        let storage = PostgresStorage::new(pool);
        let service = MailServiceImpl::new(storage);
        
        let agent = match service.get_agent(agent_id.clone()).await {
            Ok(a) => a,
            Err(_) => return Html(templates::error_page(&format!("Agent '{}' not found", agent_id))),
        };
        
        let mailbox = match service.get_agent_mailbox(agent_id.clone()).await {
            Ok(m) => m,
            Err(_) => return Html(templates::error_page("Failed to get mailbox")),
        };
        
        let mail = match service.get_mailbox_inbox(mailbox.id).await {
            Ok(m) => m,
            Err(_) => vec![],
        };
        
        (mail, agent.name)
    } else {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        let agent = match service.get_agent(agent_id.clone()).await {
            Ok(a) => a,
            Err(_) => return Html(templates::error_page(&format!("Agent '{}' not found", agent_id))),
        };
        
        let mailbox = match service.get_agent_mailbox(agent_id.clone()).await {
            Ok(m) => m,
            Err(_) => return Html(templates::error_page("Failed to get mailbox")),
        };
        
        let mail = match service.get_mailbox_inbox(mailbox.id).await {
            Ok(m) => m,
            Err(_) => vec![],
        };
        
        (mail, agent.name)
    };
    
    // Count unread messages
    let unread_count = inbox_mail.iter().filter(|m| !m.read).count();
    
    let mail_html = inbox_mail.iter()
        .map(|m| {
            let status_class = if m.read { "read" } else { "unread" };
            let mail_id_short = &m.id.to_string()[..8];
            
            let mark_read_button = if !m.read {
                format!(
                    r##"<button class="btn btn-sm btn-secondary" hx-post="/mail/{}/read" hx-target="#mail-{}" hx-swap="outerHTML">Mark as Read</button>"##,
                    m.id, mail_id_short
                )
            } else {
                String::new()
            };
            
            let read_badge = if m.read { 
                r#"<span class="badge badge-secondary">Read</span>"# 
            } else { 
                r#"<span class="badge badge-success">Unread</span>"# 
            };
            
            format!(
                r##"<div id="mail-{}" class="mail-card {}">
                    <div class="mail-header">
                        <span class="mail-subject">{}</span>
                        <span class="mail-meta">{} {}</span>
                    </div>
                    <div class="mail-body">{}</div>
                    <div class="mail-actions">{}</div>
                </div>"##,
                mail_id_short, status_class, m.subject, m.created_at.format("%Y-%m-%d %H:%M"), 
                read_badge, m.body, mark_read_button
            )
        })
        .collect::<String>();
    
    // Mark All as Read button (only show if there are unread messages)
    let mark_all_button = if unread_count > 0 {
        format!(
            r##"<button class="btn btn-sm btn-success" hx-post="/mail/inbox/{}/read-all" hx-target="#inbox-content" hx-swap="innerHTML">✓ Mark All as Read ({} unread)</button>"##,
            agent_id, unread_count
        )
    } else {
        String::new()
    };
    
    let content = format!(
        r##"
        <div id="inbox-content">
        <div class="back-link">
            <a href="/" class="btn btn-secondary btn-sm">&larr; Back to Dashboard</a>
        </div>
        <div class="inbox-header">
            <h2>Inbox: {} <span class="section-count">{} messages</span></h2>
            {}
        </div>
        <div class="mail-list">
            {}
        </div>
        </div>
        "##,
        agent_name,
        inbox_mail.len(),
        mark_all_button,
        if mail_html.is_empty() {
            "<p class='empty-state'>No mail in inbox</p>".to_string()
        } else {
            mail_html
        }
    );
    
    Html(templates::wrap_content(content))
}

// Outbox view - Show sent messages for an agent
async fn outbox_view(database_url: Option<String>, agent_id: String) -> Html<String> {
    let (outbox_mail, agent_name) = if let Some(url) = database_url {
        let pool = match sqlx::postgres::PgPool::connect(&url).await {
            Ok(p) => p,
            Err(_) => return Html(templates::error_page("Failed to connect to database")),
        };
        let storage = PostgresStorage::new(pool);
        let service = MailServiceImpl::new(storage);
        
        let agent = match service.get_agent(agent_id.clone()).await {
            Ok(a) => a,
            Err(_) => return Html(templates::error_page(&format!("Agent '{}' not found", agent_id))),
        };
        
        let mailbox = match service.get_agent_mailbox(agent_id.clone()).await {
            Ok(m) => m,
            Err(_) => return Html(templates::error_page("Failed to get mailbox")),
        };
        
        let mail = match service.get_mailbox_outbox(mailbox.id).await {
            Ok(m) => m,
            Err(_) => vec![],
        };
        
        (mail, agent.name)
    } else {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        let agent = match service.get_agent(agent_id.clone()).await {
            Ok(a) => a,
            Err(_) => return Html(templates::error_page(&format!("Agent '{}' not found", agent_id))),
        };
        
        let mailbox = match service.get_agent_mailbox(agent_id.clone()).await {
            Ok(m) => m,
            Err(_) => return Html(templates::error_page("Failed to get mailbox")),
        };
        
        let mail = match service.get_mailbox_outbox(mailbox.id).await {
            Ok(m) => m,
            Err(_) => vec![],
        };
        
        (mail, agent.name)
    };
    
    let mail_html = outbox_mail.iter()
        .map(|m| {
            format!(
                r#"<div class="mail-card sent">
                    <div class="mail-header">
                        <span class="mail-subject">{}</span>
                        <span class="mail-meta">To: {} • {}</span>
                    </div>
                    <div class="mail-body">{}</div>
                </div>"#,
                m.subject, m.to_mailbox_id, m.created_at.format("%Y-%m-%d %H:%M"), m.body
            )
        })
        .collect::<String>();
    
    let content = format!(
        r#"
        <div class="back-link">
            <a href="/" class="btn btn-secondary btn-sm">&larr; Back to Dashboard</a>
        </div>
        <h2>Outbox: {} <span class="section-count">{} messages</span></h2>
        <div class="mail-list">
            {}
        </div>
        "#,
        agent_name,
        outbox_mail.len(),
        if mail_html.is_empty() {
            "<p class='empty-state'>No sent messages</p>".to_string()
        } else {
            mail_html
        }
    );
    
    Html(templates::wrap_content(content))
}

// Send mail to agent from human
async fn send_mail(database_url: Option<String>, body: axum::body::Bytes) -> Html<String> {
    // Parse form data from body
    let body_str = String::from_utf8_lossy(&body);
    let params: std::collections::HashMap<String, String> = body_str
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?.to_string();
            let value = parts.next().unwrap_or("").to_string();
            // Simple URL decode: replace + with space and %XX with actual character
            let decoded = value.replace('+', " ")
                .replace("%20", " ")
                .replace("%21", "!")
                .replace("%22", "\"")
                .replace("%23", "#")
                .replace("%24", "$")
                .replace("%25", "%")
                .replace("%26", "&")
                .replace("%27", "'")
                .replace("%28", "(")
                .replace("%29", ")")
                .replace("%2C", ",")
                .replace("%2F", "/")
                .replace("%3A", ":")
                .replace("%3B", ";")
                .replace("%3D", "=")
                .replace("%3F", "?")
                .replace("%40", "@")
                .replace("%5B", "[")
                .replace("%5D", "]");
            Some((key, decoded))
        })
        .collect();
    
    let to_agent = params.get("to").cloned().unwrap_or_default();
    let from_human = params.get("from").cloned().unwrap_or_default();
    let subject = params.get("subject").cloned().unwrap_or_default();
    let body_text = params.get("body").cloned().unwrap_or_default();
    
    if to_agent.is_empty() || body_text.is_empty() {
        return Html(format!(
            r#"<div class="send-result error">Error: To and body are required</div>"#
        ));
    }
    
    // Use provided subject or default to "Message from {sender}"
    let subject = if subject.is_empty() {
        format!("Message from {}", from_human)
    } else {
        subject
    };
    
    let result = if let Some(url) = database_url {
        let pool = match sqlx::postgres::PgPool::connect(&url).await {
            Ok(p) => p,
            Err(_) => return Html(templates::error_page("Failed to connect to database")),
        };
        let storage = PostgresStorage::new(pool);
        let service = MailServiceImpl::new(storage);
        
        service.send_agent_to_agent(
            from_human.clone(),
            to_agent.clone(),
            subject,
            body_text.clone(),
        ).await
    } else {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        service.send_agent_to_agent(
            from_human.clone(),
            to_agent.clone(),
            subject,
            body_text.clone(),
        ).await
    };
    
    match result {
        Ok(_) => Html(format!(
            r#"<div class="send-result success">✓ Message sent to {}</div>"#,
            to_agent
        )),
        Err(_) => Html(format!(
            r#"<div class="send-result error">✗ Failed to send message</div>"#
        )),
    }
}

// Mark a single mail as read
async fn mark_mail_read(database_url: Option<String>, mail_id: String) -> Html<String> {
    let result = if let Some(url) = database_url {
        let pool = match sqlx::postgres::PgPool::connect(&url).await {
            Ok(p) => p,
            Err(_) => return Html("<div class='error'>Database connection failed</div>".to_string()),
        };
        let storage = PostgresStorage::new(pool);
        let service = MailServiceImpl::new(storage);
        
        // Try to parse as UUID first
        if let Ok(id) = uuid::Uuid::parse_str(&mail_id) {
            service.mark_mail_as_read(id).await
        } else {
            // Try as short ID
            service.mark_mail_as_read_by_short_id(&mail_id).await
        }
    } else {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        if let Ok(id) = uuid::Uuid::parse_str(&mail_id) {
            service.mark_mail_as_read(id).await
        } else {
            service.mark_mail_as_read_by_short_id(&mail_id).await
        }
    };
    
    match result {
        Ok(_) => Html(r#"<span class="badge badge-success">✓ Read</span>"#.to_string()),
        Err(_) => Html(r#"<span class="badge badge-error">✗ Failed</span>"#.to_string()),
    }
}

// Mark all mail in inbox as read
async fn mark_all_mail_read(database_url: Option<String>, agent_id: String) -> Html<String> {
    let result = if let Some(url) = database_url.clone() {
        let pool = match sqlx::postgres::PgPool::connect(&url).await {
            Ok(p) => p,
            Err(_) => return Html("<div class='error'>Database connection failed</div>".to_string()),
        };
        let storage = PostgresStorage::new(pool);
        let service = MailServiceImpl::new(storage);
        
        // Get mailbox and mark all unread mail as read
        match service.get_agent_mailbox(agent_id.clone()).await {
            Ok(mailbox) => {
                match service.get_mailbox_inbox(mailbox.id).await {
                    Ok(mail) => {
                        let mut marked_count = 0;
                        for m in mail {
                            if !m.read {
                                if let Ok(_) = service.mark_mail_as_read(m.id).await {
                                    marked_count += 1;
                                }
                            }
                        }
                        Ok(marked_count)
                    }
                    Err(_) => Err(()),
                }
            }
            Err(_) => Err(()),
        }
    } else {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        match service.get_agent_mailbox(agent_id.clone()).await {
            Ok(mailbox) => {
                match service.get_mailbox_inbox(mailbox.id).await {
                    Ok(mail) => {
                        let mut marked_count = 0;
                        for m in mail {
                            if !m.read {
                                if let Ok(_) = service.mark_mail_as_read(m.id).await {
                                    marked_count += 1;
                                }
                            }
                        }
                        Ok(marked_count)
                    }
                    Err(_) => Err(()),
                }
            }
            Err(_) => Err(()),
        }
    };
    
    match result {
        Ok(_count) => {
            // Return updated inbox view
            inbox_view(database_url, agent_id).await
        }
        Err(_) => Html(templates::error_page("Failed to mark mail as read")),
    }
}
