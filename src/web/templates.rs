pub const HTML_HEADER: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Agent Office</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600&family=IBM+Plex+Sans:wght@300;400;500;600;700&display=swap" rel="stylesheet">
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <div class="container">
        <nav class="navbar">
            <div class="navbar-inner">
                <a href="/" class="navbar-brand">
                    <span class="navbar-logo">AO</span>
                    <span class="navbar-title">Agent Office</span>
                </a>
                <div class="navbar-links">
                    <a href="/">Dashboard</a>
                    <a href="/agents">Agents</a>
                    <a href="/kb">KB</a>
                </div>
            </div>
        </nav>
        <main class="content">
"#;

pub const HTML_FOOTER: &str = r#"
        </main>
        <footer class="site-footer">
            <span class="footer-mono">agent-office v0.1.0</span>
        </footer>
    </div>
</body>
</html>
"#;

pub const CSS: &str = r##"
/* ============================================================
   Agent Office — Scientific / IBM Plex Design System
   ============================================================ */

/* --- Custom Properties --- */
:root {
    --font-sans: 'IBM Plex Sans', -apple-system, BlinkMacSystemFont, sans-serif;
    --font-mono: 'IBM Plex Mono', 'SF Mono', 'Fira Code', monospace;

    --color-bg: #f4f5f7;
    --color-surface: #ffffff;
    --color-surface-raised: #ffffff;
    --color-surface-sunken: #ebedf0;
    --color-border: #d1d5db;
    --color-border-light: #e5e7eb;

    --color-text: #1a1d23;
    --color-text-secondary: #5f6672;
    --color-text-muted: #8b919d;

    --color-primary: #2563eb;
    --color-primary-hover: #1d4ed8;
    --color-primary-light: #eff4ff;
    --color-primary-border: #bfdbfe;

    --color-success: #059669;
    --color-success-bg: #ecfdf5;
    --color-success-border: #a7f3d0;

    --color-warning: #d97706;
    --color-warning-bg: #fffbeb;
    --color-warning-border: #fde68a;

    --color-danger: #dc2626;
    --color-danger-bg: #fef2f2;
    --color-danger-border: #fecaca;

    --color-neutral: #6b7280;
    --color-neutral-bg: #f3f4f6;
    --color-neutral-border: #d1d5db;

    --color-header-bg: #111827;
    --color-header-text: #f9fafb;
    --color-header-muted: #9ca3af;

    --shadow-xs: 0 1px 2px rgba(0, 0, 0, 0.04);
    --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.06), 0 1px 2px rgba(0, 0, 0, 0.04);
    --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.07), 0 2px 4px -2px rgba(0, 0, 0, 0.05);
    --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.08), 0 4px 6px -4px rgba(0, 0, 0, 0.04);

    --radius-sm: 4px;
    --radius-md: 6px;
    --radius-lg: 10px;
    --radius-xl: 14px;

    --transition-fast: 120ms ease;
    --transition-base: 200ms ease;
    --transition-slow: 300ms ease;
}

/* --- Reset --- */
*, *::before, *::after {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

/* --- Base --- */
html {
    font-size: 16px;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
}

body {
    font-family: var(--font-sans);
    font-weight: 400;
    background: var(--color-bg);
    color: var(--color-text);
    line-height: 1.65;
    letter-spacing: -0.01em;
}

/* --- Layout --- */
.container {
    max-width: 960px;
    margin: 0 auto;
    padding: 0 20px;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
}

/* --- Navbar --- */
.navbar {
    background: var(--color-header-bg);
    margin: 16px 0 0;
    padding: 0 20px;
    border-radius: var(--radius-lg) var(--radius-lg) 0 0;
}

.navbar-inner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 56px;
}

.navbar-brand {
    display: flex;
    align-items: center;
    gap: 10px;
    text-decoration: none;
    color: var(--color-header-text);
}

.navbar-logo {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    background: var(--color-primary);
    color: white;
    border-radius: var(--radius-md);
    font-family: var(--font-mono);
    font-weight: 600;
    font-size: 13px;
    letter-spacing: 0.02em;
}

.navbar-title {
    font-weight: 600;
    font-size: 15px;
    letter-spacing: -0.02em;
}

.navbar-links {
    display: flex;
    gap: 4px;
}

.navbar-links a {
    color: var(--color-header-muted);
    text-decoration: none;
    font-size: 13px;
    font-weight: 500;
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    transition: color var(--transition-fast), background var(--transition-fast);
}

.navbar-links a:hover {
    color: var(--color-header-text);
    background: rgba(255, 255, 255, 0.08);
}

/* --- Content Area --- */
.content {
    background: var(--color-surface);
    padding: 28px 32px;
    border-radius: 0 0 var(--radius-lg) var(--radius-lg);
    box-shadow: var(--shadow-sm);
    border: 1px solid var(--color-border-light);
    border-top: none;
    flex: 1;
}

/* --- Footer --- */
.site-footer {
    padding: 20px 0;
    text-align: center;
}

.footer-mono {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--color-text-muted);
    letter-spacing: 0.04em;
    text-transform: uppercase;
}

/* --- Typography --- */
h2 {
    font-family: var(--font-sans);
    font-weight: 600;
    font-size: 20px;
    color: var(--color-text);
    margin-bottom: 20px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--color-border-light);
    letter-spacing: -0.02em;
    line-height: 1.3;
}

h3 {
    font-family: var(--font-sans);
    font-weight: 600;
    font-size: 15px;
    color: var(--color-text);
    letter-spacing: -0.01em;
    line-height: 1.4;
}

p {
    color: var(--color-text-secondary);
    font-size: 14px;
}

/* --- Agent List Grid --- */
.agent-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 12px;
}

/* --- Agent Card --- */
.agent-card {
    background: var(--color-surface);
    border: 1px solid var(--color-border-light);
    border-radius: var(--radius-lg);
    padding: 16px 18px;
    transition: border-color var(--transition-base), box-shadow var(--transition-base);
}

.agent-card:hover {
    border-color: var(--color-border);
    box-shadow: var(--shadow-md);
}

.agent-card .agent-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
}

.agent-card .agent-info h3 {
    margin: 0;
    font-size: 15px;
}

.agent-card .agent-info p {
    color: var(--color-text-muted);
    font-size: 13px;
    margin: 0;
}

/* --- Status Badges --- */
.status {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 3px 10px;
    border-radius: 100px;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    white-space: nowrap;
    border: 1px solid transparent;
}

.status::before {
    content: '';
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
}

.status.online {
    background: var(--color-success-bg);
    color: var(--color-success);
    border-color: var(--color-success-border);
}

.status.online::before {
    background: var(--color-success);
    box-shadow: 0 0 0 2px rgba(5, 150, 105, 0.2);
}

.status.offline {
    background: var(--color-neutral-bg);
    color: var(--color-neutral);
    border-color: var(--color-neutral-border);
}

.status.offline::before {
    background: var(--color-neutral);
}

.status.busy {
    background: var(--color-warning-bg);
    color: var(--color-warning);
    border-color: var(--color-warning-border);
}

.status.busy::before {
    background: var(--color-warning);
    box-shadow: 0 0 0 2px rgba(217, 119, 6, 0.2);
}

/* --- Mailboxes Section --- */
.agent-mailboxes {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--color-border-light);
}

.agent-mailboxes h4 {
    margin: 0 0 8px 0;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 500;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
}

.mailbox-item {
    display: inline-block;
    margin: 2px 4px 2px 0;
}

/* --- Buttons --- */
.btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 7px 14px;
    background: var(--color-primary);
    color: white;
    text-decoration: none;
    border-radius: var(--radius-md);
    border: 1px solid transparent;
    cursor: pointer;
    font-family: var(--font-sans);
    font-size: 13px;
    font-weight: 500;
    line-height: 1.4;
    transition: background var(--transition-fast), box-shadow var(--transition-fast), transform var(--transition-fast);
    white-space: nowrap;
}

.btn:hover {
    background: var(--color-primary-hover);
    box-shadow: var(--shadow-xs);
}

.btn:active {
    transform: scale(0.98);
}

.btn-sm {
    padding: 4px 10px;
    font-size: 12px;
    border-radius: var(--radius-sm);
}

.btn-secondary {
    background: var(--color-surface);
    color: var(--color-text);
    border-color: var(--color-border);
}

.btn-secondary:hover {
    background: var(--color-surface-sunken);
    box-shadow: var(--shadow-xs);
}

.btn-success {
    background: var(--color-success);
    color: white;
}

.btn-success:hover {
    background: #047857;
}

.btn-offline {
    background: var(--color-surface);
    color: var(--color-danger);
    border-color: var(--color-danger-border);
    margin-left: 8px;
}

.btn-offline:hover {
    background: var(--color-danger-bg);
    border-color: var(--color-danger);
}

/* --- Data Table --- */
.data-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 14px;
}

.data-table thead th {
    text-align: left;
    padding: 10px 14px;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 500;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    border-bottom: 2px solid var(--color-border-light);
}

.data-table tbody td {
    padding: 12px 14px;
    border-bottom: 1px solid var(--color-border-light);
    vertical-align: middle;
}

.data-table tbody tr:last-child td {
    border-bottom: none;
}

.data-table tbody tr {
    transition: background var(--transition-fast);
}

.data-table tbody tr:hover {
    background: var(--color-surface-sunken);
}

/* --- Mail List --- */
.mail-list {
    display: grid;
    gap: 8px;
}

.mail-card {
    border: 1px solid var(--color-border-light);
    border-radius: var(--radius-md);
    padding: 14px 16px;
    transition: border-color var(--transition-base), box-shadow var(--transition-base);
    border-left: 3px solid transparent;
}

.mail-card:hover {
    border-color: var(--color-border);
    box-shadow: var(--shadow-sm);
}

.mail-card.unread {
    background: var(--color-warning-bg);
    border-left-color: var(--color-warning);
}

.mail-card.read {
    background: var(--color-surface);
    border-left-color: var(--color-success);
}

.mail-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 12px;
    margin-bottom: 6px;
}

.mail-subject {
    font-weight: 600;
    font-size: 14px;
    color: var(--color-text);
}

.mail-meta {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--color-text-muted);
    white-space: nowrap;
    letter-spacing: 0.02em;
}

.mail-body {
    color: var(--color-text-secondary);
    font-size: 13px;
    line-height: 1.6;
    margin-top: 6px;
}

/* --- Forms --- */
.form-group {
    margin-bottom: 16px;
}

.form-group label {
    display: block;
    margin-bottom: 4px;
    font-weight: 500;
    font-size: 13px;
    color: var(--color-text);
}

.form-control {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    font-family: var(--font-sans);
    font-size: 14px;
    color: var(--color-text);
    background: var(--color-surface);
    transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
}

.form-control:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px var(--color-primary-light);
}

textarea.form-control {
    min-height: 120px;
    resize: vertical;
    font-family: var(--font-sans);
}

/* --- Note Cards (KB) --- */
.note-list {
    display: grid;
    gap: 12px;
}

.note-card {
    border: 1px solid var(--color-border-light);
    border-radius: var(--radius-lg);
    padding: 16px 18px;
    background: var(--color-surface);
    transition: border-color var(--transition-base), box-shadow var(--transition-base);
}

.note-card:hover {
    border-color: var(--color-border);
    box-shadow: var(--shadow-sm);
}

.note-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
}

.luhmann-id {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 500;
    background: var(--color-surface-sunken);
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    letter-spacing: 0.02em;
}

/* --- Search --- */
.search-form {
    background: var(--color-surface-sunken);
    padding: 18px 20px;
    border-radius: var(--radius-lg);
    margin-bottom: 20px;
    border: 1px solid var(--color-border-light);
}

.search-results {
    margin-top: 16px;
}

/* --- Tabs --- */
.tabs {
    display: flex;
    gap: 0;
    margin-bottom: 20px;
    border-bottom: 1px solid var(--color-border-light);
}

.tab {
    padding: 10px 16px;
    background: none;
    border: none;
    cursor: pointer;
    font-family: var(--font-sans);
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-muted);
    transition: color var(--transition-fast);
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
}

.tab:hover {
    color: var(--color-text);
}

.tab.active {
    color: var(--color-primary);
    border-bottom-color: var(--color-primary);
}

/* --- Grid Utilities --- */
.grid-2 {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 16px;
}

.card {
    background: var(--color-surface);
    border: 1px solid var(--color-border-light);
    border-radius: var(--radius-lg);
    padding: 18px 20px;
}

.card h3 {
    margin-top: 0;
    margin-bottom: 8px;
}

/* --- Empty State --- */
.empty-state {
    text-align: center;
    padding: 48px 20px;
    color: var(--color-text-muted);
    font-size: 14px;
}

.empty-state h2 {
    border-bottom: none;
    color: var(--color-text-muted);
}

/* --- Error Page --- */
.error-container {
    text-align: center;
    padding: 48px 20px;
}

.error-container h2 {
    border-bottom: none;
    color: var(--color-danger);
    margin-bottom: 8px;
}

.error-container p {
    margin-bottom: 20px;
}

/* --- HTMX Indicators --- */
.htmx-indicator {
    display: none;
    margin-left: 6px;
}

.htmx-request .htmx-indicator {
    display: inline;
}

.htmx-request.htmx-indicator {
    display: inline;
}

/* --- Back Link --- */
.back-link {
    margin-bottom: 16px;
}

/* --- Section Subtitle --- */
.section-count {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 400;
    color: var(--color-text-muted);
    margin-left: 8px;
    letter-spacing: 0.02em;
}

/* ============================================================
   Responsive / Mobile
   ============================================================ */
@media (max-width: 640px) {
    .container {
        padding: 0 12px;
    }

    .navbar {
        margin: 8px -12px 0;
        padding: 0 12px;
        border-radius: var(--radius-md) var(--radius-md) 0 0;
    }

    .navbar-inner {
        height: 48px;
    }

    .navbar-title {
        font-size: 14px;
    }

    .navbar-links {
        gap: 0;
    }

    .navbar-links a {
        font-size: 12px;
        padding: 6px 8px;
    }

    .content {
        padding: 20px 16px;
        border-radius: 0 0 var(--radius-md) var(--radius-md);
    }

    h2 {
        font-size: 18px;
        margin-bottom: 16px;
    }

    .agent-list {
        grid-template-columns: 1fr;
        gap: 10px;
    }

    .agent-card {
        padding: 14px;
    }

    .agent-card .agent-info {
        flex-wrap: wrap;
        gap: 6px;
    }

    .mail-header {
        flex-direction: column;
        gap: 4px;
    }

    .grid-2 {
        grid-template-columns: 1fr;
    }

    .data-table {
        font-size: 13px;
    }

    .data-table thead th,
    .data-table tbody td {
        padding: 10px 10px;
    }

    .btn {
        padding: 8px 14px;
        font-size: 13px;
    }

    .btn-sm {
        padding: 6px 10px;
        font-size: 12px;
    }

    .btn-offline {
        margin-left: 0;
        margin-top: 4px;
    }
}

@media (max-width: 380px) {
    .navbar-logo {
        width: 28px;
        height: 28px;
        font-size: 11px;
    }

    .navbar-title {
        font-size: 13px;
    }

    .content {
        padding: 16px 12px;
    }
}

/* --- Knowledge Base Styles --- */

.page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--color-border-light);
}

.page-header h2 {
    font-size: 22px;
    font-weight: 600;
    margin: 0;
}

.note-count {
    font-size: 13px;
    color: var(--color-text-muted);
    font-family: var(--font-mono);
}

.notes-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.note-card {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 16px;
    transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
}

.note-card:hover {
    border-color: var(--color-primary-border);
    box-shadow: var(--shadow-sm);
}

.note-header {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin-bottom: 8px;
}

.note-id {
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--color-primary);
    font-weight: 500;
}

.note-id a {
    color: inherit;
    text-decoration: none;
}

.note-id a:hover {
    text-decoration: underline;
}

.note-title {
    font-weight: 600;
    font-size: 15px;
}

.note-preview {
    color: var(--color-text-secondary);
    font-size: 13px;
    line-height: 1.5;
    margin-bottom: 12px;
}

.note-meta {
    display: flex;
    gap: 8px;
}

/* Note Detail Page */
.note-detail {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: 24px;
    margin-bottom: 24px;
}

.note-breadcrumb {
    font-size: 13px;
    color: var(--color-text-muted);
    margin-bottom: 12px;
}

.note-breadcrumb a {
    color: var(--color-primary);
    text-decoration: none;
}

.note-breadcrumb a:hover {
    text-decoration: underline;
}

.note-title-large {
    font-size: 24px;
    font-weight: 600;
    margin: 0 0 16px 0;
    color: var(--color-text);
}

.note-content-full {
    font-size: 14px;
    line-height: 1.7;
    color: var(--color-text);
    margin-bottom: 20px;
}

.note-meta-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: 16px;
    border-top: 1px solid var(--color-border-light);
    font-size: 12px;
    color: var(--color-text-muted);
}

/* Note Relationships */
.note-relationships {
    background: var(--color-surface-sunken);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 20px;
}

.relation-section {
    margin-bottom: 16px;
}

.relation-section:last-child {
    margin-bottom: 0;
}

.relation-section h4 {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-secondary);
    margin: 0 0 8px 0;
    text-transform: uppercase;
    letter-spacing: 0.03em;
}

.relation-link {
    display: block;
    padding: 8px 12px;
    margin-bottom: 4px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    text-decoration: none;
    color: var(--color-text);
    font-size: 13px;
    transition: background var(--transition-fast);
}

.relation-link:hover {
    background: var(--color-primary-light);
    border-color: var(--color-primary-border);
}

/* Tree View */
.tree-view {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: 24px;
}

.tree-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--color-border-light);
}

.tree-header h2 {
    font-size: 20px;
    margin: 0;
}

.tree-structure {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.tree-level {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding-left: 0;
}

.tree-level.parent-level {
    margin-bottom: 16px;
    padding-bottom: 16px;
    border-bottom: 2px solid var(--color-border);
}

.tree-node {
    display: block;
    padding: 10px 14px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    text-decoration: none;
    color: var(--color-text);
    font-size: 14px;
    transition: all var(--transition-fast);
}

.tree-node:hover {
    background: var(--color-surface-raised);
    border-color: var(--color-primary-border);
}

.tree-node.parent-node {
    background: var(--color-neutral-bg);
    font-weight: 500;
}

.tree-node.current-node {
    background: var(--color-primary-light);
    border-color: var(--color-primary);
    font-weight: 500;
}

.tree-stats {
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid var(--color-border-light);
    font-size: 13px;
    color: var(--color-text-muted);
}

"##;

pub fn wrap_content(content: impl AsRef<str>) -> String {
    format!("{}{}{}", HTML_HEADER, content.as_ref(), HTML_FOOTER)
}

pub fn error_page(message: &str) -> String {
    wrap_content(format!(
        r#"
        <div class="error-container">
            <h2>Error</h2>
            <p>{}</p>
            <a href="/" class="btn btn-secondary">Back to Dashboard</a>
        </div>
        "#,
        message
    ))
}
