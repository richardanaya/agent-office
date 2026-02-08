pub const HTML_HEADER: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Agent Office</title>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <div class="container">
        <nav class="navbar">
            <h1><a href="/">🏢 Agent Office</a></h1>
        </nav>
        <main class="content">
"#;

pub const HTML_FOOTER: &str = r#"
        </main>
    </div>
</body>
</html>
"#;

pub const CSS: &str = r#"
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: #f5f5f5;
    color: #333;
    line-height: 1.6;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
}

.navbar {
    background: #2c3e50;
    color: white;
    padding: 15px 20px;
    border-radius: 8px;
    margin-bottom: 20px;
    text-align: center;
}

.navbar a {
    color: white;
    text-decoration: none;
}

.navbar h1 a:hover {
    opacity: 0.8;
}

.content {
    background: white;
    padding: 20px;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

h2 {
    color: #2c3e50;
    margin-bottom: 20px;
    padding-bottom: 10px;
    border-bottom: 2px solid #ecf0f1;
}

h3 {
    color: #34495e;
    margin: 20px 0 10px 0;
}

.agent-list {
    display: grid;
    gap: 10px;
}

.agent-card {
    background: #ecf0f1;
    padding: 15px;
    border-radius: 6px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    transition: transform 0.2s, box-shadow 0.2s;
}

.agent-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 8px rgba(0,0,0,0.1);
}

.agent-info h3 {
    margin: 0;
    color: #2c3e50;
}

.agent-info p {
    color: #7f8c8d;
    font-size: 0.9em;
}

.status {
    display: inline-block;
    padding: 4px 12px;
    border-radius: 20px;
    font-size: 0.85em;
    font-weight: bold;
}

.status.online {
    background: #2ecc71;
    color: white;
}

.status.offline {
    background: #95a5a6;
    color: white;
}

.status.busy {
    background: #e74c3c;
    color: white;
}

.btn {
    display: inline-block;
    padding: 8px 16px;
    background: #3498db;
    color: white;
    text-decoration: none;
    border-radius: 4px;
    border: none;
    cursor: pointer;
    font-size: 14px;
    transition: background 0.2s;
}

.btn:hover {
    background: #2980b9;
}

.btn-secondary {
    background: #95a5a6;
}

.btn-secondary:hover {
    background: #7f8c8d;
}

.btn-success {
    background: #2ecc71;
}

.btn-success:hover {
    background: #27ae60;
}

.btn-sm {
    padding: 4px 12px;
    font-size: 12px;
}

.btn-offline {
    background: #e74c3c;
    margin-left: 10px;
}

.btn-offline:hover {
    background: #c0392b;
}

.agent-mailboxes {
    margin-top: 10px;
    padding-top: 10px;
    border-top: 1px dashed #bdc3c7;
}

.agent-mailboxes h4 {
    margin: 0 0 8px 0;
    font-size: 14px;
    color: #7f8c8d;
}

.mailbox-item {
    display: inline-block;
    margin: 2px;
}

.agent-card {
    background: #ecf0f1;
    padding: 15px;
    border-radius: 6px;
    transition: transform 0.2s, box-shadow 0.2s;
}

.agent-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 8px rgba(0,0,0,0.1);
}

.agent-card .agent-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
}

.agent-card .agent-info h3 {
    margin: 0;
    color: #2c3e50;
}

.agent-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 15px;
}

.mail-list {
    display: grid;
    gap: 10px;
}

.mail-card {
    border: 1px solid #ecf0f1;
    border-radius: 6px;
    padding: 15px;
    transition: border-color 0.2s;
}

.mail-card:hover {
    border-color: #3498db;
}

.mail-card.unread {
    background: #fff3cd;
    border-left: 4px solid #ffc107;
}

.mail-card.read {
    background: #f8f9fa;
    border-left: 4px solid #28a745;
}

.mail-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
}

.mail-subject {
    font-weight: bold;
    color: #2c3e50;
}

.mail-meta {
    color: #7f8c8d;
    font-size: 0.85em;
}

.mail-body {
    color: #555;
    margin-top: 10px;
}

.form-group {
    margin-bottom: 15px;
}

.form-group label {
    display: block;
    margin-bottom: 5px;
    font-weight: bold;
    color: #2c3e50;
}

.form-control {
    width: 100%;
    padding: 10px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 14px;
}

.form-control:focus {
    outline: none;
    border-color: #3498db;
    box-shadow: 0 0 0 3px rgba(52, 152, 219, 0.1);
}

textarea.form-control {
    min-height: 120px;
    resize: vertical;
}

.note-list {
    display: grid;
    gap: 15px;
}

.note-card {
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    padding: 15px;
    background: white;
}

.note-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
}

.luhmann-id {
    font-family: monospace;
    background: #e8e8e8;
    padding: 2px 8px;
    border-radius: 3px;
    font-size: 0.9em;
    color: #555;
}

.search-form {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 6px;
    margin-bottom: 20px;
}

.search-results {
    margin-top: 20px;
}

.empty-state {
    text-align: center;
    padding: 40px;
    color: #7f8c8d;
}

.tabs {
    display: flex;
    gap: 5px;
    margin-bottom: 20px;
    border-bottom: 2px solid #ecf0f1;
}

.tab {
    padding: 10px 20px;
    background: none;
    border: none;
    cursor: pointer;
    color: #7f8c8d;
    transition: all 0.2s;
}

.tab:hover {
    color: #2c3e50;
}

.tab.active {
    color: #3498db;
    border-bottom: 2px solid #3498db;
}

.grid-2 {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 20px;
}

.card {
    background: white;
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    padding: 20px;
}

.card h3 {
    margin-top: 0;
}

.htmx-indicator {
    display: none;
}

.htmx-request .htmx-indicator {
    display: inline;
}

.htmx-request.htmx-indicator {
    display: inline;
}
"#;

pub fn wrap_content(content: impl AsRef<str>) -> String {
    format!("{}{}{}", HTML_HEADER, content.as_ref(), HTML_FOOTER)
}

pub fn error_page(message: &str) -> String {
    wrap_content(format!(
        r#"
        <div class="empty-state">
            <h2>Error</h2>
            <p>{}</p>
            <a href="/" class="btn">Go Home</a>
        </div>
        "#,
        message
    ))
}
