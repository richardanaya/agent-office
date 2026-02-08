use axum::{
    extract::Path,
    response::Html,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;

pub mod templates;

use crate::services::mail::{MailService, MailServiceImpl};
use crate::services::kb::{KnowledgeBaseService, KnowledgeBaseServiceImpl, LuhmannId};
use crate::storage::{memory::InMemoryStorage, postgres::PostgresStorage};

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
    let db_url3 = Arc::new(database_url);
    
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
        
        // Update agent status
        .route("/agents/{agent_id}/status", post({
            let db = db_url3.clone();
            move |Path(agent_id): Path<String>| set_agent_status((*db).clone(), agent_id)
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
        
        // For now, just show inbox link (agents auto-create inbox)
        let mailbox_list = format!(
            r#"<div class="mailbox-item">
                <a href="/mail/inbox/{}" class="btn btn-sm">📧 Inbox</a>
            </div>"#,
            agent.id
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
        r#"
        <h2>Dashboard <span class="section-count">{} agents</span></h2>
        <div class="agent-list">
            {}
        </div>
        "#,
        agents.len(),
        if agent_cards.is_empty() {
            "<p class='empty-state'>No agents registered yet</p>".to_string()
        } else {
            agent_cards
        }
    );
    
    Html(templates::wrap_content(content))
}

// List all agents (separate page)
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
            </tr>"#,
            agent.name, status_class, agent.status
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
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>
        "#,
        agents.len(),
        if agent_rows.is_empty() {
            "<tr><td colspan=\"2\" class=\"empty-state\">No agents registered</td></tr>".to_string()
        } else {
            agent_rows
        }
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
    
    let mail_html = inbox_mail.iter()
        .map(|m| {
            let status_class = if m.read { "read" } else { "unread" };
            format!(
                r#"<div class="mail-card {}">
                    <div class="mail-header">
                        <span class="mail-subject">{}</span>
                        <span class="mail-meta">{}</span>
                    </div>
                    <div class="mail-body">{}</div>
                </div>"#,
                status_class, m.subject, m.created_at.format("%Y-%m-%d %H:%M"), m.body
            )
        })
        .collect::<String>();
    
    let content = format!(
        r#"
        <div class="back-link">
            <a href="/" class="btn btn-secondary btn-sm">&larr; Back to Dashboard</a>
        </div>
        <h2>Inbox: {} <span class="section-count">{} messages</span></h2>
        <div class="mail-list">
            {}
        </div>
        "#,
        agent_name,
        inbox_mail.len(),
        if mail_html.is_empty() {
            "<p class='empty-state'>No mail in inbox</p>".to_string()
        } else {
            mail_html
        }
    );
    
    Html(templates::wrap_content(content))
}
