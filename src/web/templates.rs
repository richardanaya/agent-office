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
                    <a href="/agents">⏰ Schedules</a>
                </div>
            </div>
        </nav>
        <main class="content">
"#;

pub const HTML_FOOTER: &str = r#"
        </main>
        <footer class="site-footer">
            <span class="footer-mono">agent-office v0.1.14</span>
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

/* --- Session Editor --- */
.agentcard .agentsession {
    margin: 12px 0;
    padding: 10px;
    background: var(--color-bg-tertiary);
    border-radius: 6px;
    border: 1px solid var(--color-border-light);
}

.sessioneditor {
    margin: 12px 0;
    padding: 10px;
    background: var(--color-bg-tertiary);
    border-radius: 6px;
    border: 1px solid var(--color-border-light);
}

.sessionform {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.sessionform .formgroup {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
}

.sessionform label {
    font-weight: 500;
    font-size: 12px;
    color: var(--color-text-muted);
    min-width: 70px;
}

.sessioninput {
    flex: 1;
    min-width: 150px;
    padding: 4px 8px;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    font-family: var(--font-mono);
    font-size: 12px;
    background: var(--color-bg);
}

.sessionhint {
    font-size: 11px;
    color: var(--color-text-muted);
    font-style: italic;
}

.sessiondisplay {
    font-size: 12px;
    margin-top: 8px;
    font-family: var(--font-mono);
}

.sessiondisplay .textmuted {
    color: var(--color-text-muted);
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

/* ============================================================
   Knowledge Base — Zettelkasten Reading Experience
   ============================================================ */

/* --- KB List Page --- */

.page-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 20px;
    padding-bottom: 14px;
    border-bottom: 1px solid var(--color-border-light);
}

.page-header h2 {
    font-size: 20px;
    font-weight: 600;
    margin: 0;
    border: none;
    padding: 0;
}

.note-count {
    font-size: 12px;
    color: var(--color-text-muted);
    font-family: var(--font-mono);
}

.notes-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

/* Note cards are now <a> tags — full-card tap targets for mobile */
.note-card {
    display: block;
    text-decoration: none;
    color: inherit;
    background: var(--color-surface);
    border-left: 3px solid var(--color-border-light);
    padding: 14px 16px;
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    transition: border-color var(--transition-fast), background var(--transition-fast);
}

.note-card:hover, .note-card:focus {
    border-left-color: var(--color-primary);
    background: var(--color-primary-light);
    outline: none;
}

.note-card:active {
    background: var(--color-primary-light);
}

.note-header {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin-bottom: 4px;
    flex-wrap: wrap;
}

.note-id {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--color-primary);
    font-weight: 500;
    flex-shrink: 0;
}

.note-title {
    font-weight: 600;
    font-size: 15px;
    line-height: 1.35;
    flex: 1;
    min-width: 0;
}

.note-date {
    font-size: 11px;
    color: var(--color-text-muted);
    font-family: var(--font-mono);
    flex-shrink: 0;
    margin-left: auto;
}

.note-preview {
    color: var(--color-text-secondary);
    font-size: 13px;
    line-height: 1.55;
    margin-bottom: 6px;
    /* Clamp to 3 lines on mobile */
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
}

.note-tags, .note-tags-detail {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-bottom: 6px;
}

.tag-badge {
    display: inline-block;
    font-size: 11px;
    font-family: var(--font-mono);
    padding: 1px 8px;
    background: var(--color-neutral-bg);
    color: var(--color-text-secondary);
    border-radius: 100px;
    border: 1px solid var(--color-border-light);
}

.note-meta {
    display: flex;
    gap: 12px;
    font-size: 11px;
    color: var(--color-text-muted);
    font-family: var(--font-mono);
}

.note-depth {
    opacity: 0.6;
}

.note-tree-link {
    cursor: pointer;
    color: var(--color-primary);
}

.note-tree-link:hover {
    text-decoration: underline;
}

.empty-state {
    text-align: center;
    padding: 40px 20px;
    color: var(--color-text-muted);
    font-size: 14px;
}

.empty-state code {
    font-family: var(--font-mono);
    background: var(--color-neutral-bg);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-size: 13px;
}

/* --- Note Detail Page --- */

.note-detail {
    background: var(--color-surface);
    padding: 0;
    margin-bottom: 24px;
}

.note-breadcrumb {
    font-size: 13px;
    color: var(--color-text-muted);
    margin-bottom: 16px;
    line-height: 1.6;
    position: sticky;
    top: 0;
    background: var(--color-surface);
    padding: 10px 0;
    z-index: 10;
    border-bottom: 1px solid var(--color-border-light);
}

.note-breadcrumb a {
    color: var(--color-primary);
    text-decoration: none;
}

.note-breadcrumb a:hover {
    text-decoration: underline;
}

.bc-sep {
    color: var(--color-border);
    margin: 0 2px;
}

.note-title-large {
    font-size: 26px;
    font-weight: 700;
    margin: 0 0 8px 0;
    color: var(--color-text);
    line-height: 1.3;
    letter-spacing: -0.02em;
    border: none;
    padding: 0;
}

.note-meta-bar-top {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
    margin-bottom: 24px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--color-border-light);
}

.note-id-detail {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--color-primary);
    font-weight: 500;
    background: var(--color-primary-light);
    padding: 2px 8px;
    border-radius: var(--radius-sm);
}

.note-date-detail {
    font-size: 12px;
    color: var(--color-text-muted);
    font-family: var(--font-mono);
}

.note-tags-detail {
    margin-bottom: 0;
}

/* --- Markdown Prose Styles --- */

.prose {
    font-size: 16px;
    line-height: 1.75;
    color: var(--color-text);
    margin-bottom: 24px;
    overflow-wrap: break-word;
    word-break: break-word;
}

.prose h1, .prose h2, .prose h3, .prose h4, .prose h5, .prose h6 {
    font-family: var(--font-sans);
    font-weight: 600;
    color: var(--color-text);
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    line-height: 1.3;
    letter-spacing: -0.01em;
    border: none;
    padding: 0;
}

.prose h1 { font-size: 1.5em; }
.prose h2 { font-size: 1.3em; }
.prose h3 { font-size: 1.15em; }
.prose h4 { font-size: 1em; }

.prose p {
    margin: 0 0 1em 0;
    font-size: inherit;
    color: inherit;
}

.prose ul, .prose ol {
    margin: 0 0 1em 0;
    padding-left: 1.5em;
}

.prose li {
    margin-bottom: 0.35em;
    line-height: 1.65;
}

.prose li > ul, .prose li > ol {
    margin-top: 0.35em;
    margin-bottom: 0;
}

.prose blockquote {
    margin: 0 0 1em 0;
    padding: 0.75em 1em;
    border-left: 3px solid var(--color-primary-border);
    background: var(--color-primary-light);
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    color: var(--color-text-secondary);
    font-style: italic;
}

.prose blockquote p:last-child {
    margin-bottom: 0;
}

.prose code {
    font-family: var(--font-mono);
    font-size: 0.88em;
    background: var(--color-neutral-bg);
    padding: 0.15em 0.4em;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border-light);
}

.prose pre {
    margin: 0 0 1em 0;
    padding: 14px 16px;
    background: #1e293b;
    color: #e2e8f0;
    border-radius: var(--radius-md);
    overflow-x: auto;
    font-size: 0.85em;
    line-height: 1.6;
    -webkit-overflow-scrolling: touch;
}

.prose pre code {
    font-family: var(--font-mono);
    background: none;
    padding: 0;
    border: none;
    border-radius: 0;
    font-size: inherit;
    color: inherit;
}

.prose a {
    color: var(--color-primary);
    text-decoration: underline;
    text-decoration-color: var(--color-primary-border);
    text-underline-offset: 2px;
    transition: text-decoration-color var(--transition-fast);
}

.prose a:hover {
    text-decoration-color: var(--color-primary);
}

.prose strong {
    font-weight: 600;
    color: var(--color-text);
}

.prose em {
    font-style: italic;
}

.prose hr {
    border: none;
    border-top: 1px solid var(--color-border-light);
    margin: 1.5em 0;
}

.prose table {
    width: 100%;
    border-collapse: collapse;
    margin: 0 0 1em 0;
    font-size: 0.9em;
}

.prose th, .prose td {
    padding: 8px 12px;
    border: 1px solid var(--color-border);
    text-align: left;
}

.prose th {
    background: var(--color-neutral-bg);
    font-weight: 600;
    font-size: 0.85em;
    text-transform: uppercase;
    letter-spacing: 0.03em;
}

.prose img {
    max-width: 100%;
    height: auto;
    border-radius: var(--radius-md);
}

/* Task lists (checkbox lists) */
.prose ul li input[type="checkbox"] {
    margin-right: 6px;
}

.note-content-full {
    font-size: 16px;
    line-height: 1.75;
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
    flex-wrap: wrap;
    gap: 8px;
}

/* --- Note Relationships --- */

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
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text-muted);
    margin: 0 0 8px 0;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-family: var(--font-mono);
}

.relation-link {
    display: block;
    padding: 10px 14px;
    margin-bottom: 4px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    text-decoration: none;
    color: var(--color-text);
    font-size: 14px;
    transition: background var(--transition-fast);
    /* Good touch target */
    min-height: 44px;
    display: flex;
    align-items: center;
}

.relation-link:hover, .relation-link:active {
    background: var(--color-primary-light);
    border-color: var(--color-primary-border);
}

/* --- Tree View --- */

.tree-view {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: 20px;
}

.tree-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--color-border-light);
    flex-wrap: wrap;
    gap: 8px;
}

.tree-header h2 {
    font-size: 18px;
    margin: 0;
    border: none;
    padding: 0;
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
    padding: 12px 14px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    text-decoration: none;
    color: var(--color-text);
    font-size: 14px;
    transition: all var(--transition-fast);
    /* 44px minimum touch target */
    min-height: 44px;
    display: flex;
    align-items: center;
}

.tree-node:hover, .tree-node:active {
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

/* ============================================================
   Mobile Responsive — KB Focus
   ============================================================ */

@media (max-width: 768px) {
    /* Global layout */
    .container {
        padding: 0;
    }

    .navbar {
        margin: 0;
        border-radius: 0;
        padding: 0 14px;
    }

    .navbar-inner {
        height: 48px;
    }

    .navbar-title {
        font-size: 14px;
    }

    .navbar-links a {
        font-size: 12px;
        padding: 5px 8px;
    }

    .content {
        padding: 16px;
        border-radius: 0;
        border-left: none;
        border-right: none;
    }

    /* KB list */
    .page-header {
        margin-bottom: 14px;
        padding-bottom: 10px;
    }

    .page-header h2 {
        font-size: 18px;
    }

    .notes-list {
        gap: 1px;
    }

    .note-card {
        padding: 12px 14px;
        /* On mobile, remove the Luhmann indent so notes don't get squished */
        margin-left: 0 !important;
        border-left-width: 3px;
    }

    .note-header {
        gap: 6px;
    }

    .note-title {
        font-size: 14px;
    }

    .note-preview {
        font-size: 13px;
        -webkit-line-clamp: 2;
    }

    .note-date {
        display: none; /* save space; date shown on detail */
    }

    .note-meta {
        font-size: 10px;
    }

    /* Note detail */
    .note-breadcrumb {
        font-size: 12px;
        padding: 8px 0;
    }

    .note-title-large {
        font-size: 22px;
    }

    .note-meta-bar-top {
        margin-bottom: 16px;
        padding-bottom: 12px;
    }

    .prose {
        font-size: 15px;
        line-height: 1.7;
    }

    .prose h1 { font-size: 1.35em; }
    .prose h2 { font-size: 1.2em; }
    .prose h3 { font-size: 1.1em; }

    .prose pre {
        padding: 12px;
        font-size: 0.8em;
        border-radius: var(--radius-sm);
    }

    .prose table {
        font-size: 0.85em;
    }

    .prose th, .prose td {
        padding: 6px 8px;
    }

    /* Relationships */
    .note-relationships {
        padding: 14px;
    }

    .relation-link {
        padding: 12px 14px;
        font-size: 13px;
    }

    /* Tree view */
    .tree-view {
        padding: 14px;
    }

    .tree-header h2 {
        font-size: 16px;
    }

    .tree-node {
        padding: 12px;
        font-size: 13px;
    }
}

/* Small phones */
@media (max-width: 400px) {
    .content {
        padding: 12px;
    }

    .note-card {
        padding: 10px 12px;
    }

    .note-title {
        font-size: 13px;
    }

    .note-title-large {
        font-size: 20px;
    }

    .prose {
        font-size: 14px;
    }

    .tag-badge {
        font-size: 10px;
        padding: 1px 6px;
    }
}

/* --- Send Message Form --- */
.send-message-card {
    background: var(--color-surface);
    border: 1px solid var(--color-border-light);
    border-radius: var(--radius-lg);
    padding: 20px 24px;
    margin-bottom: 24px;
    box-shadow: var(--shadow-sm);
}

.send-message-card h3 {
    margin-top: 0;
    margin-bottom: 16px;
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text);
}

.form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-bottom: 16px;
}

.form-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
}

.form-group label {
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.03em;
}

.form-group input,
.form-group textarea {
    padding: 10px 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    font-family: var(--font-sans);
    font-size: 14px;
    background: var(--color-surface);
    transition: border-color 0.15s ease;
}

.form-group input:focus,
.form-group textarea:focus {
    outline: none;
    border-color: var(--color-accent);
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}

.form-group textarea {
    resize: vertical;
    min-height: 80px;
}

.form-actions {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 16px;
}

.send-result {
    font-size: 13px;
    padding: 8px 12px;
    border-radius: var(--radius-md);
}

.send-result.success {
    background: var(--color-success-bg);
    color: var(--color-success);
}

.send-result.error {
    background: var(--color-danger-bg);
    color: var(--color-danger);
}

@media (max-width: 640px) {
    .form-row {
        grid-template-columns: 1fr;
    }
    
    .send-message-card {
        padding: 16px;
    }
}

/* --- Schedule Management --- */
.schedules-list {
    display: grid;
    gap: 16px;
    margin-bottom: 32px;
}

.schedule-item {
    background: white;
    border: 1px solid var(--color-border);
    border-radius: 8px;
    padding: 16px;
    box-shadow: var(--shadow-sm);
}

.schedule-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 8px;
    flex-wrap: wrap;
}

.schedule-cron {
    font-family: var(--font-mono);
    font-size: 14px;
    font-weight: 500;
    color: var(--color-primary);
    background: var(--color-bg);
    padding: 4px 8px;
    border-radius: 4px;
}

.schedule-action {
    font-size: 14px;
    color: var(--color-text);
    margin: 8px 0;
    line-height: 1.5;
}

.schedule-meta {
    font-size: 12px;
    color: var(--color-text-muted);
    font-family: var(--font-mono);
}

.schedule-form {
    background: white;
    border: 1px solid var(--color-border);
    border-radius: 8px;
    padding: 24px;
    max-width: 600px;
}

.schedule-form .form-group {
    margin-bottom: 16px;
}

.schedule-form label {
    display: block;
    font-size: 14px;
    font-weight: 500;
    margin-bottom: 6px;
    color: var(--color-text);
}

.schedule-form input,
.schedule-form textarea {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    font-size: 14px;
    font-family: inherit;
}

.schedule-form input:focus,
.schedule-form textarea:focus {
    outline: none;
    border-color: var(--color-primary);
}

.schedule-form small {
    display: block;
    margin-top: 4px;
    font-size: 12px;
    color: var(--color-text-muted);
}

.badge {
    display: inline-block;
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 500;
}

.badge-success {
    background: rgba(16, 185, 129, 0.1);
    color: var(--color-success);
}

.badge-secondary {
    background: var(--color-bg);
    color: var(--color-text-muted);
}

.empty-state {
    color: var(--color-text-muted);
    font-style: italic;
    padding: 24px;
    text-align: center;
    background: var(--color-bg);
    border-radius: 8px;
}

/* --- Enhanced Schedule Cards --- */
.schedules-container {
    display: grid;
    gap: 20px;
    margin-bottom: 32px;
}

.schedule-card {
    background: white;
    border: 1px solid var(--color-border);
    border-radius: 12px;
    padding: 20px;
    box-shadow: var(--shadow-sm);
}

.schedule-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--color-border);
}

.schedule-title {
    display: flex;
    align-items: center;
    gap: 12px;
}

.schedule-actions {
    display: flex;
    gap: 8px;
}

.schedule-body {
    display: grid;
    gap: 12px;
}

.schedule-detail {
    font-size: 14px;
}

.schedule-detail strong {
    display: block;
    margin-bottom: 6px;
    color: var(--color-text-muted);
}

.schedule-action-text {
    background: var(--color-bg);
    padding: 12px;
    border-radius: 6px;
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
    margin: 0;
    max-height: 200px;
    overflow-y: auto;
}

.schedule-edit-form {
    margin-top: 16px;
    padding: 16px;
    background: var(--color-bg);
    border-radius: 8px;
    border: 1px solid var(--color-border);
}

.schedule-edit-form .form-group {
    margin-bottom: 12px;
}

.schedule-edit-form label {
    display: block;
    font-size: 13px;
    font-weight: 500;
    margin-bottom: 4px;
    color: var(--color-text);
}

.schedule-edit-form input,
.schedule-edit-form textarea {
    width: 100%;
    padding: 8px 10px;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    font-size: 13px;
    font-family: inherit;
}

.btn-danger {
    background: #dc3545;
    color: white;
    border: none;
}

.btn-danger:hover {
    background: #c82333;
}

/* --- Inbox Styles --- */
.inbox-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    flex-wrap: wrap;
    gap: 12px;
}

.inbox-header h2 {
    margin: 0;
}

.mail-actions {
    margin-top: 12px;
    display: flex;
    gap: 8px;
}

.mail-card .badge {
    margin-left: 8px;
}

.badge-error {
    background: rgba(220, 53, 69, 0.1);
    color: #dc3545;
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
