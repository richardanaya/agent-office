use axum::response::Html;

use crate::services::mail::{MailService, MailServiceImpl};
use crate::services::schedule::{ScheduleService, ScheduleServiceImpl};
use crate::storage::postgres::PostgresStorage;
use crate::web::templates;

// View agent schedules
pub async fn agent_schedule_view(database_url: Option<String>, agent_id: String) -> Html<String> {
    let (agent, schedules, schedule_service_opt) = if let Some(ref url) = database_url {
        let pool = match sqlx::postgres::PgPool::connect(url).await {
            Ok(p) => p,
            Err(_) => return Html(templates::error_page("Failed to connect to database")),
        };
        let mail_service = MailServiceImpl::new(PostgresStorage::new(pool.clone()));
        let schedule_service = ScheduleServiceImpl::new(pool);
        
        let agent = match mail_service.get_agent(agent_id.clone()).await {
            Ok(a) => a,
            Err(_) => return Html(templates::error_page(&format!("Agent '{}' not found", agent_id))),
        };
        
        let schedules = match schedule_service.list_schedules_by_agent(&agent_id).await {
            Ok(s) => s,
            Err(_) => vec![],
        };
        
        (agent, schedules, Some(schedule_service))
    } else {
        return Html(templates::error_page("Database connection required"));
    };
    
    let current_time = chrono::Utc::now();
    
    // Build schedule list HTML with full details
    let mut schedules_html = String::new();
    for schedule in &schedules {
        let status_badge = if schedule.is_active {
            "<span class=\"badge badge-success\">Active</span>"
        } else {
            "<span class=\"badge badge-secondary\">Inactive</span>"
        };
        
        let last_fired = schedule.last_fired_at
            .map(|t| format!("Last fired: {}", t.format("%Y-%m-%d %H:%M UTC")))
            .unwrap_or_else(|| "Never fired".to_string());
        
        // Calculate next run time
        let next_run = if schedule.is_active {
            if let Some(ref service) = schedule_service_opt {
                service.get_next_run(schedule, current_time)
                    .map(|t| format!("Next: {}", t.format("%Y-%m-%d %H:%M UTC")))
                    .unwrap_or_else(|| "No upcoming runs".to_string())
            } else {
                "Unable to calculate".to_string()
            }
        } else {
            "Inactive - no upcoming runs".to_string()
        };
        
        let schedule_id_full = schedule.id.to_string();
        let schedule_id_short = &schedule_id_full[..8];
        
        // Full action text (not truncated)
        let action_escaped = html_escape(&schedule.action);
        
        schedules_html.push_str("<div class=\"schedule-card\">");
        schedules_html.push_str("<div class=\"schedule-card-header\">");
        schedules_html.push_str(&format!(
            "<div class=\"schedule-title\"><span class=\"schedule-cron\">{}</span> {}</div>",
            html_escape(&schedule.cron_expression),
            status_badge
        ));
        schedules_html.push_str("<div class=\"schedule-actions\">");
        schedules_html.push_str(&format!(
            "<button class=\"btn btn-sm btn-secondary\" onclick=\"toggleEditForm('{}')\">Edit</button>",
            schedule_id_short
        ));
        schedules_html.push_str(&format!(
            "<button class=\"btn btn-sm btn-danger\" hx-post=\"/schedules/{}/delete\" hx-confirm=\"Delete this schedule?\" hx-target=\"#schedules-list\" hx-swap=\"innerHTML\">Delete</button>",
            schedule.id
        ));
        schedules_html.push_str(&format!(
            "<button class=\"btn btn-sm btn-secondary\" hx-post=\"/schedules/{}/toggle\" hx-target=\"#schedules-list\" hx-swap=\"innerHTML\">{}</button>",
            schedule.id,
            if schedule.is_active { "Disable" } else { "Enable" }
        ));
        schedules_html.push_str("</div>");
        schedules_html.push_str("</div>");
        
        schedules_html.push_str("<div class=\"schedule-body\">");
        schedules_html.push_str(&format!(
            "<div class=\"schedule-detail\"><strong>Action:</strong><pre class=\"schedule-action-text\">{}</pre></div>",
            action_escaped
        ));
        schedules_html.push_str(&format!(
            "<div class=\"schedule-meta\">{}<br>{}<br>ID: {}</div>",
            last_fired, next_run, schedule_id_short
        ));
        schedules_html.push_str("</div>");
        
        // Hidden edit form
        schedules_html.push_str(&format!(
            "<div id=\"edit-form-{}\" class=\"schedule-edit-form\" style=\"display:none;\">",
            schedule_id_short
        ));
        schedules_html.push_str(&format!(
            "<form hx-post=\"/schedules/{}/update\" hx-target=\"#schedules-list\" hx-swap=\"innerHTML\">",
            schedule.id
        ));
        schedules_html.push_str("<div class=\"form-group\">");
        schedules_html.push_str("<label>CRON Expression</label>");
        schedules_html.push_str(&format!(
            "<input type=\"text\" name=\"cron\" value=\"{}\" required>",
            html_escape(&schedule.cron_expression)
        ));
        schedules_html.push_str("</div>");
        schedules_html.push_str("<div class=\"form-group\">");
        schedules_html.push_str("<label>Action</label>");
        schedules_html.push_str(&format!(
            "<textarea name=\"action\" rows=\"3\" required>{}</textarea>",
            action_escaped
        ));
        schedules_html.push_str("</div>");
        schedules_html.push_str("<button type=\"submit\" class=\"btn btn-success\">Save Changes</button>");
        schedules_html.push_str(&format!(
            "<button type=\"button\" class=\"btn btn-secondary\" onclick=\"toggleEditForm('{}')\">Cancel</button>",
            schedule_id_short
        ));
        schedules_html.push_str("</form>");
        schedules_html.push_str("</div>");
        
        schedules_html.push_str("</div>");
    }
    
    if schedules_html.is_empty() {
        schedules_html = "<p class=\"empty-state\">No schedules configured yet. Create one below.</p>".to_string();
    }
    
    let mut content = String::new();
    content.push_str("<div class=\"back-link\">");
    content.push_str("<a href=\"/\" class=\"btn btn-secondary btn-sm\">&larr; Back to Dashboard</a>");
    content.push_str("</div>");
    content.push_str(&format!(
        "<h2>Schedules for {} <span class=\"section-count\">{} total</span></h2>",
        html_escape(&agent.name),
        schedules.len()
    ));
    content.push_str("<div id=\"schedules-list\" class=\"schedules-container\">");
    content.push_str(&schedules_html);
    content.push_str("</div>");
    
    // Add JavaScript for edit form toggle
    content.push_str(r#"
<script>
function toggleEditForm(id) {
    const form = document.getElementById('edit-form-' + id);
    if (form.style.display === 'none') {
        form.style.display = 'block';
    } else {
        form.style.display = 'none';
    }
}
</script>
"#);
    
    content.push_str("<h3>Create New Schedule</h3>");
    content.push_str(&format!(
        "<form class=\"schedule-form\" hx-post=\"/agents/{}/schedule\" hx-target=\"#schedules-list\" hx-swap=\"innerHTML\">",
        agent_id
    ));
    content.push_str("<div class=\"form-group\">");
    content.push_str("<label>CRON Expression</label>");
    content.push_str("<input type=\"text\" name=\"cron\" placeholder=\"* * * * * (every minute) or 0 9 * * * (daily at 9am)\" required>");
    content.push_str("<small>Format: minute hour day month weekday (5 fields) OR seconds minute hour day month weekday (6 fields)</small>");
    content.push_str("</div>");
    content.push_str("<div class=\"form-group\">");
    content.push_str("<label>Action</label>");
    content.push_str("<textarea name=\"action\" rows=\"3\" placeholder=\"What should the agent do when this fires? (supports markdown)\" required></textarea>");
    content.push_str("</div>");
    content.push_str("<button type=\"submit\" class=\"btn btn-success\">Create Schedule</button>");
    content.push_str("</form>");
    
    Html(templates::wrap_content(content))
}

// Create new schedule via web form
pub async fn create_schedule(database_url: Option<String>, agent_id: String, body: axum::body::Bytes) -> Html<String> {
    let body_str = String::from_utf8_lossy(&body);
    let params: std::collections::HashMap<String, String> = body_str
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?.to_string();
            let value = parts.next().unwrap_or("").to_string();
            Some((key, value))
        })
        .collect();
    
    let cron = params.get("cron").cloned().unwrap_or_default();
    let action = params.get("action").cloned().unwrap_or_default();
    
    let cron = urldecode(&cron);
    let action = urldecode(&action);
    
    if let Some(url) = database_url {
        let pool = match sqlx::postgres::PgPool::connect(&url).await {
            Ok(p) => p,
            Err(_) => return Html("<div class=\"error\">Failed to connect to database</div>".to_string()),
        };
        let schedule_service = ScheduleServiceImpl::new(pool);
        
        match schedule_service.create_schedule(agent_id.clone(), cron, action).await {
            Ok(_) => agent_schedule_view(Some(url), agent_id).await,
            Err(e) => Html(format!("<div class=\"error\">Failed to create schedule: {}</div>", e)),
        }
    } else {
        Html("<div class=\"error\">Database required</div>".to_string())
    }
}

// Update schedule
pub async fn update_schedule(database_url: Option<String>, schedule_id: String, body: axum::body::Bytes) -> Html<String> {
    let id = match uuid::Uuid::parse_str(&schedule_id) {
        Ok(u) => u,
        Err(_) => return Html("<div class=\"error\">Invalid schedule ID</div>".to_string()),
    };
    
    let body_str = String::from_utf8_lossy(&body);
    let params: std::collections::HashMap<String, String> = body_str
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?.to_string();
            let value = parts.next().unwrap_or("").to_string();
            Some((key, value))
        })
        .collect();
    
    let cron = params.get("cron").cloned();
    let action = params.get("action").cloned();
    
    let cron = cron.map(|c| urldecode(&c));
    let action = action.map(|a| urldecode(&a));
    
    if let Some(url) = database_url {
        let pool = match sqlx::postgres::PgPool::connect(&url).await {
            Ok(p) => p,
            Err(_) => return Html("<div class=\"error\">Failed to connect to database</div>".to_string()),
        };
        let schedule_service = ScheduleServiceImpl::new(pool);
        
        // Get agent_id for redirect
        let agent_id = match schedule_service.get_schedule(id).await {
            Ok(s) => s.agent_id,
            Err(_) => return Html("<div class=\"error\">Schedule not found</div>".to_string()),
        };
        
        match schedule_service.update_schedule(id, cron, action).await {
            Ok(_) => agent_schedule_view(Some(url), agent_id).await,
            Err(e) => Html(format!("<div class=\"error\">Failed to update schedule: {}</div>", e)),
        }
    } else {
        Html("<div class=\"error\">Database required</div>".to_string())
    }
}

// Delete schedule
pub async fn delete_schedule(database_url: Option<String>, schedule_id: String) -> Html<String> {
    let id = match uuid::Uuid::parse_str(&schedule_id) {
        Ok(u) => u,
        Err(_) => return Html("<div class=\"error\">Invalid schedule ID</div>".to_string()),
    };
    
    if let Some(url) = database_url {
        let pool = match sqlx::postgres::PgPool::connect(&url).await {
            Ok(p) => p,
            Err(_) => return Html("<div class=\"error\">Failed to connect to database</div>".to_string()),
        };
        let schedule_service = ScheduleServiceImpl::new(pool);
        
        // Get agent_id before deleting
        let agent_id = match schedule_service.get_schedule(id).await {
            Ok(s) => s.agent_id,
            Err(_) => return Html("<div class=\"error\">Schedule not found</div>".to_string()),
        };
        
        match schedule_service.delete_schedule(id).await {
            Ok(_) => agent_schedule_view(Some(url), agent_id).await,
            Err(e) => Html(format!("<div class=\"error\">Failed to delete schedule: {}</div>", e)),
        }
    } else {
        Html("<div class=\"error\">Database required</div>".to_string())
    }
}

// Toggle schedule on/off
pub async fn toggle_schedule(database_url: Option<String>, schedule_id: String) -> Html<String> {
    let id = match uuid::Uuid::parse_str(&schedule_id) {
        Ok(u) => u,
        Err(_) => return Html("<div class=\"error\">Invalid schedule ID</div>".to_string()),
    };
    
    if let Some(url) = database_url {
        let pool = match sqlx::postgres::PgPool::connect(&url).await {
            Ok(p) => p,
            Err(_) => return Html("<div class=\"error\">Failed to connect to database</div>".to_string()),
        };
        let schedule_service = ScheduleServiceImpl::new(pool);
        
        // Get the agent_id from the schedule so we can return the updated list
        let agent_id = match schedule_service.get_schedule(id).await {
            Ok(s) => s.agent_id,
            Err(_) => return Html("<div class=\"error\">Schedule not found</div>".to_string()),
        };
        
        match schedule_service.toggle_schedule(id).await {
            Ok(_) => agent_schedule_view(Some(url), agent_id).await,
            Err(e) => Html(format!("<div class=\"error\">Failed to toggle schedule: {}</div>", e)),
        }
    } else {
        Html("<div class=\"error\">Database required</div>".to_string())
    }
}

// Simple URL decode function
fn urldecode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        if chars[i] == '%' && i + 2 < chars.len() {
            let hex = format!("{}{}", chars[i + 1], chars[i + 2]);
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
                i += 3;
                continue;
            }
        } else if chars[i] == '+' {
            result.push(' ');
            i += 1;
            continue;
        }
        result.push(chars[i]);
        i += 1;
    }
    
    result
}

// Simple HTML escape function
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
