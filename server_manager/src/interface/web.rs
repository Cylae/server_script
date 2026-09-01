use crate::core::config::Config;
use crate::core::users::{Role, UserManager};
use crate::services;
use axum::{
    extract::{Form, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use sysinfo::{CpuExt, DiskExt, System, SystemExt};
use time::Duration;
use tokio::process::Command;
use tokio::sync::RwLock;
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer};

#[derive(Serialize, Deserialize, Clone)]
struct SessionUser {
    username: String,
    role: Role,
}

const SESSION_KEY: &str = "user";

struct CachedConfig {
    config: Config,
    last_modified: Option<SystemTime>,
}

struct CachedUsers {
    manager: UserManager,
    last_modified: Option<SystemTime>,
}

struct AppState {
    system: Mutex<System>,
    last_system_refresh: Mutex<SystemTime>,
    config_cache: RwLock<CachedConfig>,
    users_cache: RwLock<CachedUsers>,
}

type SharedState = Arc<AppState>;

impl AppState {
    async fn get_config(&self) -> Config {
        // Fast path: check metadata
        let current_mtime = tokio::fs::metadata("config.yaml")
            .await
            .and_then(|m| m.modified())
            .ok();

        {
            let cache = self.config_cache.read().await;
            if cache.last_modified == current_mtime {
                return cache.config.clone();
            }
        }

        // Slow path: reload
        let mut cache = self.config_cache.write().await;

        // Re-check mtime under write lock to avoid race
        let current_mtime_2 = tokio::fs::metadata("config.yaml")
            .await
            .and_then(|m| m.modified())
            .ok();

        if cache.last_modified == current_mtime_2 {
            return cache.config.clone();
        }

        if let Ok(cfg) = Config::load_async().await {
            cache.config = cfg;
            cache.last_modified = current_mtime_2;
        }

        cache.config.clone()
    }

    async fn get_users(&self) -> UserManager {
        // Determine path logic (matches UserManager::load)
        let path = std::path::Path::new("users.yaml");
        let fallback_path = std::path::Path::new("/opt/server_manager/users.yaml");
        let file_path = if path.exists() { path } else { fallback_path };

        // Fast path: check metadata
        let current_mtime = tokio::fs::metadata(file_path)
            .await
            .and_then(|m| m.modified())
            .ok();

        {
            let cache = self.users_cache.read().await;
            // If mtime matches (or both None), return cached
            if cache.last_modified == current_mtime {
                return cache.manager.clone();
            }
        }

        // Slow path: reload
        let mut cache = self.users_cache.write().await;

        // Re-check mtime under write lock
        let current_mtime_2 = tokio::fs::metadata(file_path)
            .await
            .and_then(|m| m.modified())
            .ok();

        if cache.last_modified == current_mtime_2 {
            return cache.manager.clone();
        }

        if let Ok(mgr) = UserManager::load_async().await {
            cache.manager = mgr;
            cache.last_modified = current_mtime_2;
        }

        cache.manager.clone()
    }
}

pub async fn start_server(port: u16) -> anyhow::Result<()> {
    // Session setup
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Localhost/LAN, http usually
        .with_expiry(Expiry::OnInactivity(Duration::hours(24)));

    // Initialize System once
    let mut sys = System::new_all();
    sys.refresh_all();

    let initial_config = Config::load().unwrap_or_default();
    let initial_config_mtime = std::fs::metadata("config.yaml")
        .ok()
        .and_then(|m| m.modified().ok());

    let initial_users = UserManager::load().unwrap_or_default();
    let initial_users_mtime = std::fs::metadata("users.yaml")
        .ok()
        .and_then(|m| m.modified().ok())
        .or_else(|| {
            std::fs::metadata("/opt/server_manager/users.yaml")
                .ok()
                .and_then(|m| m.modified().ok())
        });

    let app_state = Arc::new(AppState {
        system: Mutex::new(sys),
        last_system_refresh: Mutex::new(SystemTime::now()),
        config_cache: RwLock::new(CachedConfig {
            config: initial_config,
            last_modified: initial_config_mtime,
        }),
        users_cache: RwLock::new(CachedUsers {
            manager: initial_users,
            last_modified: initial_users_mtime,
        }),
    });

    let app = Router::new()
        .route("/", get(dashboard))
        .route("/users", get(users_page))
        .route("/users/add", post(add_user_handler))
        .route("/users/delete/:username", post(delete_user_handler))
        .route("/updates", get(updates_page))
        .route("/user/apps/:name/install", post(user_install_app_handler))
        .route(
            "/user/apps/:name/uninstall",
            post(user_uninstall_app_handler),
        )
        .route(
            "/user/profile",
            get(user_profile_page).post(user_passwd_handler),
        )
        .route("/api/services/:name/enable", post(enable_service))
        .route("/api/services/:name/disable", post(disable_service))
        .route("/api/system/update", post(trigger_system_update))
        .route("/logout", post(logout))
        .route("/login", get(login_page).post(login_handler))
        .layer(session_layer)
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Starting Web UI on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn login_page(session: Session) -> impl IntoResponse {
    if let Some(_user) = session
        .get::<SessionUser>(SESSION_KEY)
        .await
        .unwrap_or_default()
    {
        return Redirect::to("/").into_response();
    }

    let html = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Login - Server Manager</title>
        <style>
            :root {
                --bg-main: #0f172a;
                --bg-card: #1e293b;
                --border-color: #334155;
                --text-main: #f8fafc;
                --text-muted: #94a3b8;
                --accent-blue: #38bdf8;
                --accent-hover: #0284c7;
            }
            * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
            body { background: var(--bg-main); color: var(--text-main); display: flex; justify-content: center; align-items: center; min-height: 100vh; padding: 20px; }
            .login-box { background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 12px; padding: 32px; width: 100%; max-width: 380px; box-shadow: 0 10px 25px -5px rgba(0,0,0,0.3); }
            .login-title { font-size: 1.5rem; font-weight: 700; text-align: center; margin-bottom: 24px; color: var(--text-main); }
            .form-group { margin-bottom: 16px; }
            label { display: block; font-size: 0.875rem; color: var(--text-muted); margin-bottom: 6px; }
            input { width: 100%; padding: 12px; border-radius: 8px; border: 1px solid var(--border-color); background: #0f172a; color: var(--text-main); font-size: 1rem; transition: border-color 0.2s; }
            input:focus { outline: none; border-color: var(--accent-blue); }
            button { width: 100%; padding: 12px; background: var(--accent-blue); color: #0f172a; font-weight: 600; border: none; border-radius: 8px; cursor: pointer; font-size: 1rem; transition: background 0.2s; margin-top: 8px; }
            button:hover { background: var(--accent-hover); }
        </style>
    </head>
    <body>
        <div class="login-box">
            <h2 class="login-title">Server Manager 🚀</h2>
            <form method="POST" action="/login">
                <div class="form-group">
                    <label>Username</label>
                    <input type="text" name="username" placeholder="Username" required autofocus>
                </div>
                <div class="form-group">
                    <label>Password</label>
                    <input type="password" name="password" placeholder="Password" required>
                </div>
                <button type="submit">Sign In</button>
            </form>
        </div>
    </body>
    </html>
    "#;
    Html(html).into_response()
}

#[derive(Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

async fn login_handler(
    State(state): State<SharedState>,
    session: Session,
    Form(payload): Form<LoginPayload>,
) -> impl IntoResponse {
    // Reload users on login attempt to get fresh data
    let user_manager = state.get_users().await;

    if let Some(user) = user_manager
        .verify_async(&payload.username, &payload.password)
        .await
    {
        let session_user = SessionUser {
            username: user.username,
            role: user.role,
        };
        session.clear().await;
        if let Err(e) = session.insert(SESSION_KEY, session_user).await {
            error!("Failed to insert session: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create session",
            )
                .into_response();
        }
        Redirect::to("/").into_response()
    } else {
        // Simple error handling: redirect back to login
        warn!("Failed login attempt for user: {}", payload.username);
        Redirect::to("/login").into_response()
    }
}

async fn logout(session: Session) -> impl IntoResponse {
    session.delete().await.ok();
    Redirect::to("/login")
}

// Helper for HTML escaping
struct Escaped<'a>(&'a str);

impl<'a> std::fmt::Display for Escaped<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in self.0.chars() {
            match c {
                '&' => f.write_str("&amp;")?,
                '<' => f.write_str("&lt;")?,
                '>' => f.write_str("&gt;")?,
                '"' => f.write_str("&quot;")?,
                '\'' => f.write_str("&#39;")?,
                _ => f.write_char(c)?,
            }
        }
        Ok(())
    }
}

// Helper for common HTML head
fn write_html_head(out: &mut String, title: &str) {
    let _ = writeln!(
        out,
        r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>{}</title>
        <style>
            :root {{
                --bg-main: #0f172a;
                --bg-card: #1e293b;
                --bg-card-hover: #334155;
                --border-color: #334155;
                --text-main: #f8fafc;
                --text-muted: #94a3b8;
                --accent-blue: #38bdf8;
                --accent-green: #34d399;
                --accent-red: #f87171;
                --accent-gray: #64748b;
            }}
            * {{ box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }}
            body {{ background: var(--bg-main); color: var(--text-main); min-height: 100vh; padding: 24px 16px; line-height: 1.5; }}
            .container {{ max-width: 1080px; margin: 0 auto; background: var(--bg-card); padding: 28px; border-radius: 16px; border: 1px solid var(--border-color); box-shadow: 0 10px 30px -5px rgba(0,0,0,0.3); }}
            .header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; flex-wrap: wrap; gap: 12px; }}
            .header h1 {{ font-size: 1.75rem; font-weight: 700; color: var(--text-main); }}
            .nav {{ display: flex; gap: 16px; margin-bottom: 24px; border-bottom: 1px solid var(--border-color); padding-bottom: 12px; flex-wrap: wrap; }}
            .nav a {{ color: var(--text-muted); text-decoration: none; font-weight: 600; padding: 6px 12px; border-radius: 8px; transition: all 0.2s; }}
            .nav a:hover {{ color: var(--accent-blue); background: rgba(56, 189, 248, 0.1); }}
            .stats-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 16px; margin-bottom: 28px; }}
            .stat-card {{ background: #0f172a; padding: 20px; border-radius: 12px; border: 1px solid var(--border-color); }}
            .stat-label {{ font-size: 0.875rem; color: var(--text-muted); margin-bottom: 4px; }}
            .stat-value {{ font-size: 1.5rem; font-weight: 700; color: var(--accent-blue); }}
            table {{ width: 100%; border-collapse: collapse; margin-top: 16px; border-radius: 8px; overflow: hidden; }}
            th, td {{ padding: 14px 16px; text-align: left; border-bottom: 1px solid var(--border-color); }}
            th {{ background: #0f172a; font-size: 0.875rem; color: var(--text-muted); font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; }}
            tr:last-child td {{ border-bottom: none; }}
            tr:hover td {{ background: rgba(255,255,255,0.02); }}
            .btn {{ padding: 8px 16px; border-radius: 8px; font-weight: 600; font-size: 0.875rem; text-decoration: none; border: none; cursor: pointer; display: inline-flex; align-items: center; justify-content: center; transition: all 0.2s; color: #0f172a; }}
            .btn-primary {{ background: var(--accent-blue); color: #0f172a; }}
            .btn-primary:hover {{ background: #0284c7; color: #fff; }}
            .btn-danger {{ background: var(--accent-red); color: #0f172a; }}
            .btn-danger:hover {{ background: #dc2626; color: #fff; }}
            .btn-enable {{ background: var(--accent-green); color: #0f172a; }}
            .btn-enable:hover {{ background: #059669; color: #fff; }}
            .btn-disable {{ background: var(--accent-red); color: #0f172a; }}
            .btn-disable:hover {{ background: #dc2626; color: #fff; }}
            .btn-logout {{ background: var(--accent-gray); color: #fff; }}
            .btn-logout:hover {{ background: #475569; }}
            .status-enabled {{ color: var(--accent-green); font-weight: 600; }}
            .status-disabled {{ color: var(--accent-red); font-weight: 600; }}
            input, select {{ width: 100%; padding: 10px 14px; border-radius: 8px; border: 1px solid var(--border-color); background: #0f172a; color: var(--text-main); font-size: 0.95rem; }}
            input:focus, select:focus {{ outline: none; border-color: var(--accent-blue); }}
            @media (max-width: 768px) {{
                body {{ padding: 12px 8px; }}
                .container {{ padding: 16px; border-radius: 12px; }}
                table {{ display: block; overflow-x: auto; }}
                .stats-grid {{ grid-template-columns: repeat(2, 1fr); gap: 10px; }}
            }}
        </style>
    </head>
    <body>
        <div class="container">
    "#,
        title
    );
}

fn write_html_foot(out: &mut String) {
    out.push_str(
        r#"
        </div>
    </body>
    </html>
    "#,
    );
}

// Protected Dashboard
async fn dashboard(State(state): State<SharedState>, session: Session) -> impl IntoResponse {
    let user: SessionUser = match session.get(SESSION_KEY).await {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };

    let is_admin = matches!(user.role, Role::Admin);

    let services = services::get_all_services();
    let config = state.get_config().await;

    // System Stats
    let (ram_used, ram_total, swap_used, swap_total, cpu_usage, disk_total, disk_used) = {
        let state_clone = state.clone();
        tokio::task::spawn_blocking(move || {
            let mut sys = state_clone
                .system
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let now = SystemTime::now();
            let mut last_refresh = state_clone
                .last_system_refresh
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            // Throttle refresh to max once every 500ms
            if now
                .duration_since(*last_refresh)
                .unwrap_or_default()
                .as_millis()
                > 500
            {
                sys.refresh_cpu();
                sys.refresh_memory();
                sys.refresh_disks();
                *last_refresh = now;
            }
            let ram_used = sys.used_memory() / 1024 / 1024; // MB
            let ram_total = sys.total_memory() / 1024 / 1024; // MB
            let swap_used = sys.used_swap() / 1024 / 1024; // MB
            let swap_total = sys.total_swap() / 1024 / 1024; // MB
            let cpu_usage = sys.global_cpu_info().cpu_usage();

            // Simple Disk Usage (Root or fallback)
            let mut disk_total = 0;
            let mut disk_used = 0;

            let target_disk = sys
                .disks()
                .iter()
                .find(|d| d.mount_point() == std::path::Path::new("/"))
                .or_else(|| sys.disks().first());

            if let Some(disk) = target_disk {
                disk_total = disk.total_space() / 1024 / 1024 / 1024; // GB
                disk_used = (disk.total_space() - disk.available_space()) / 1024 / 1024 / 1024;
                // GB
            }
            (
                ram_used, ram_total, swap_used, swap_total, cpu_usage, disk_total, disk_used,
            )
        })
        .await
        .expect("Blocking task should not panic")
    };

    let mut html = String::with_capacity(8192);
    write_html_head(&mut html, "Dashboard - Server Manager");

    let _ = writeln!(
        html,
        r#"
        <div class="header">
            <h1>Server Manager 🚀</h1>
            <form method="POST" action="/logout" style="margin: 0;">
                <button type="submit" class="btn btn-logout">Logout ({})</button>
            </form>
        </div>
    "#,
        Escaped(&user.username)
    );

    // Navigation
    html.push_str(r#"<div class="nav"><a href="/">Dashboard</a>"#);
    if is_admin {
        html.push_str(
            r#"<a href="/users">User Management</a><a href="/updates">Updates &amp; Software</a>"#,
        );
    }
    html.push_str(r#"<a href="/user/profile">My Profile</a></div>"#);

    // Stats Grid
    let _ = writeln!(
        html,
        r#"
        <div class="stats-grid">
            <div class="stat-card">
                <div>CPU Usage</div>
                <div class="stat-value">{:.1}%</div>
            </div>
            <div class="stat-card">
                <div>RAM Usage</div>
                <div class="stat-value">{} / {} MB</div>
            </div>
            <div class="stat-card">
                <div>Swap Usage</div>
                <div class="stat-value">{} / {} MB</div>
            </div>
            <div class="stat-card">
                <div>Disk (/)</div>
                <div class="stat-value">{} / {} GB</div>
            </div>
        </div>
    "#,
        cpu_usage, ram_used, ram_total, swap_used, swap_total, disk_used, disk_total
    );

    // Services Table
    html.push_str(
        r#"
        <h2>Services</h2>
        <table>
            <thead>
                <tr>
                    <th>Service</th>
                    <th>Image</th>
                    <th>Status</th>
                    <th>Action</th>
                </tr>
            </thead>
            <tbody>
    "#,
    );

    for svc in services {
        let name = svc.name();
        let enabled = config.is_enabled(name);
        let status_class = if enabled {
            "status-enabled"
        } else {
            "status-disabled"
        };
        let status_text = if enabled { "Enabled" } else { "Disabled" };

        let _ = writeln!(
            html,
            r#"
            <tr>
                <td>{}</td>
                <td>{}</td>
                <td class="{}">{}</td>
                <td>
        "#,
            name,
            svc.image(),
            status_class,
            status_text
        );

        if is_admin {
            let _ = writeln!(
                html,
                r#"
                    <form method="POST" action="/api/services/{}/{}">
                        <button type="submit" class="btn {}">{}</button>
                    </form>
             "#,
                name,
                if enabled { "disable" } else { "enable" },
                if enabled { "btn-disable" } else { "btn-enable" },
                if enabled {
                    "Stack Disable"
                } else {
                    "Stack Enable"
                }
            );
        } else {
            html.push_str("<span>System Service</span>");
        };

        html.push_str("</td></tr>");
    }

    // User App Management Section (Quickbox.io style)
    let user_manager = state.get_users().await;
    let current_user_opt = user_manager.get_user(&user.username);
    let installed_apps = current_user_opt
        .map(|u| u.installed_apps.clone())
        .unwrap_or_default();

    html.push_str(
        r#"
        <h2 style="margin-top: 40px;">My Applications (User Portal)</h2>
        <p>Manage your own application stack individually (Quickbox.io style).</p>
        <table>
            <thead>
                <tr>
                    <th>Application</th>
                    <th>Status</th>
                    <th>Action</th>
                </tr>
            </thead>
            <tbody>
    "#,
    );

    for svc in services {
        let name = svc.name();
        let is_app_installed = installed_apps.contains(name);
        let status_class = if is_app_installed {
            "status-enabled"
        } else {
            "status-disabled"
        };
        let status_text = if is_app_installed {
            "Installed"
        } else {
            "Not Installed"
        };

        let _ = writeln!(
            html,
            r#"
            <tr>
                <td><strong>{}</strong></td>
                <td class="{}">{}</td>
                <td>
                    <form method="POST" action="/user/apps/{}/{}" style="display:inline-block;">
                        <button type="submit" class="btn {}">{}</button>
                    </form>
                </td>
            </tr>
            "#,
            name,
            status_class,
            status_text,
            name,
            if is_app_installed {
                "uninstall"
            } else {
                "install"
            },
            if is_app_installed {
                "btn-danger"
            } else {
                "btn-primary"
            },
            if is_app_installed {
                "1-Click Uninstall"
            } else {
                "1-Click Install"
            }
        );
    }

    html.push_str("</tbody></table>");

    html.push_str(
        r#"
            </tbody>
        </table>
        <p><em>Note: Actions may take a moment to apply.</em></p>
    "#,
    );
    write_html_foot(&mut html);

    Html(html).into_response()
}

// User Management Page
async fn users_page(State(state): State<SharedState>, session: Session) -> impl IntoResponse {
    let user: SessionUser = match session.get(SESSION_KEY).await {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };

    if !matches!(user.role, Role::Admin) {
        return Redirect::to("/").into_response();
    }

    let user_manager = state.get_users().await;
    let mut html = String::with_capacity(4096);
    write_html_head(&mut html, "User Management - Server Manager");

    html.push_str(r#"
        <div class="header">
            <h1>User Management 👥</h1>
            <form method="POST" action="/logout">
                <button type="submit" class="btn btn-logout">Logout</button>
            </form>
        </div>
        <div class="nav">
            <a href="/">Dashboard</a>
            <a href="/users">User Management</a>
            <a href="/updates">Updates &amp; Software</a>
        </div>

        <h3>Add New User</h3>
        <form method="POST" action="/users/add" style="background: #f8f9fa; padding: 15px; border-radius: 6px; margin-bottom: 20px; display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 10px; align-items: end;">
            <div>
                <label>Username</label><br>
                <input type="text" name="username" required style="width: 100%; padding: 8px;">
            </div>
            <div>
                <label>Password</label><br>
                <input type="password" name="password" required style="width: 100%; padding: 8px;">
            </div>
            <div>
                <label>Role</label><br>
                <select name="role" style="width: 100%; padding: 8px;">
                    <option value="Observer">Observer</option>
                    <option value="Admin">Admin</option>
                </select>
            </div>
            <div>
                <label>Quota (GB) <small>(0 = unlimited)</small></label><br>
                <input type="number" name="quota" value="0" style="width: 100%; padding: 8px;">
            </div>
            <button type="submit" class="btn btn-primary" style="height: 35px;">Add User</button>
        </form>

        <h3>Existing Users</h3>
        <table>
            <thead>
                <tr>
                    <th>Username</th>
                    <th>Role</th>
                    <th>Quota (GB)</th>
                    <th>Actions</th>
                </tr>
            </thead>
            <tbody>
    "#);

    for u in user_manager.list_users() {
        let _ = writeln!(
            html,
            r#"
            <tr>
                <td>{}</td>
                <td>{:?}</td>
                <td>"#,
            Escaped(&u.username),
            u.role
        );

        match u.quota_gb {
            Some(gb) if gb > 0 => {
                let _ = writeln!(html, "{} GB", gb);
            }
            _ => {
                html.push_str("Unlimited");
            }
        }

        // Don't allow deleting self or last admin logic is handled in delete handler/manager
        // But let's show delete button generally
        let _ = writeln!(
            html,
            r#"</td>
                <td>
                    <form method="POST" action="/users/delete/{}" onsubmit="return confirm('Are you sure you want to delete this user? This will delete their system account and data.');">
                        <button type="submit" class="btn btn-danger">Delete</button>
                    </form>
                </td>
            </tr>
        "#,
            Escaped(&u.username)
        );
    }

    html.push_str("</tbody></table>");
    write_html_foot(&mut html);

    Html(html).into_response()
}

#[derive(Deserialize)]
struct AddUserPayload {
    username: String,
    password: String,
    role: String,
    quota: Option<u64>,
}

async fn add_user_handler(
    State(state): State<SharedState>,
    session: Session,
    Form(payload): Form<AddUserPayload>,
) -> impl IntoResponse {
    let session_user: SessionUser = match session.get(SESSION_KEY).await {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };

    if !matches!(session_user.role, Role::Admin) {
        return (StatusCode::FORBIDDEN, "Access Denied").into_response();
    }

    let role_enum = match payload.role.as_str() {
        "Admin" => Role::Admin,
        _ => Role::Observer,
    };

    let quota_val = match payload.quota {
        Some(0) => None,
        Some(v) => Some(v),
        None => None,
    };

    let mut cache = state.users_cache.write().await;
    let mut manager_clone = cache.manager.clone();

    let user_name = payload.username.clone();
    let pass = payload.password.clone();

    let res = tokio::task::spawn_blocking(move || -> anyhow::Result<UserManager> {
        manager_clone.add_user(&user_name, &pass, role_enum, quota_val)?;
        Ok(manager_clone)
    })
    .await
    .expect("Blocking task should not panic");

    match res {
        Ok(new_manager) => {
            cache.manager = new_manager;
            info!(
                "User {} added via Web UI by {}",
                payload.username, session_user.username
            );
            // Update mtime to prevent unnecessary reload
            let path = std::path::Path::new("users.yaml");
            let fallback_path = std::path::Path::new("/opt/server_manager/users.yaml");
            let file_path = if path.exists() { path } else { fallback_path };
            if let Ok(m) = std::fs::metadata(file_path) {
                cache.last_modified = m.modified().ok();
            }
        }
        Err(e) => {
            error!("Failed to add user: {}", e);
            // In a real app we'd flash a message. Here just redirect.
        }
    }

    Redirect::to("/users").into_response()
}

async fn delete_user_handler(
    State(state): State<SharedState>,
    session: Session,
    Path(username): Path<String>,
) -> impl IntoResponse {
    let session_user: SessionUser = match session.get(SESSION_KEY).await {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };

    if !matches!(session_user.role, Role::Admin) {
        return (StatusCode::FORBIDDEN, "Access Denied").into_response();
    }

    let mut cache = state.users_cache.write().await;
    let mut manager_clone = cache.manager.clone();

    let u_name = username.clone();
    let res = tokio::task::spawn_blocking(move || -> anyhow::Result<UserManager> {
        manager_clone.delete_user(&u_name)?;
        Ok(manager_clone)
    })
    .await
    .expect("Blocking task should not panic");

    match res {
        Ok(new_manager) => {
            cache.manager = new_manager;
            info!(
                "User {} deleted via Web UI by {}",
                username, session_user.username
            );
            // Update mtime to prevent unnecessary reload
            let path = std::path::Path::new("users.yaml");
            let fallback_path = std::path::Path::new("/opt/server_manager/users.yaml");
            let file_path = if path.exists() { path } else { fallback_path };
            if let Ok(m) = std::fs::metadata(file_path) {
                cache.last_modified = m.modified().ok();
            }
        }
        Err(e) => {
            error!("Failed to delete user: {}", e);
        }
    }

    Redirect::to("/users").into_response()
}

async fn enable_service(session: Session, Path(name): Path<String>) -> impl IntoResponse {
    check_admin_role(session, &name, true).await
}

async fn disable_service(session: Session, Path(name): Path<String>) -> impl IntoResponse {
    check_admin_role(session, &name, false).await
}

async fn check_admin_role(session: Session, name: &str, enable: bool) -> impl IntoResponse {
    let user: SessionUser = match session.get(SESSION_KEY).await {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };

    if !matches!(user.role, Role::Admin) {
        return (StatusCode::FORBIDDEN, "Access Denied: Admin role required").into_response();
    }

    run_cli_toggle(name, enable);
    Redirect::to("/").into_response()
}

fn run_cli_toggle(service: &str, enable: bool) {
    let action = if enable { "enable" } else { "disable" };
    info!("Web UI triggering: server_manager {} {}", action, service);

    if let Ok(exe) = std::env::current_exe() {
        match Command::new(exe).arg(action).arg(service).spawn() {
            Ok(mut child) => {
                // Spawn a background task to wait for the child process to exit.
                // This prevents zombie processes by collecting the exit status.
                tokio::spawn(async move {
                    if let Err(e) = child.wait().await {
                        error!("Failed to wait on child process: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Failed to spawn command: {}", e);
            }
        }
    } else {
        error!("Failed to determine current executable path.");
    }
}

// Updates & Software Management Page
async fn updates_page(session: Session) -> impl IntoResponse {
    let user: SessionUser = match session.get(SESSION_KEY).await {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };

    if !matches!(user.role, Role::Admin) {
        return Redirect::to("/").into_response();
    }

    let mut html = String::with_capacity(4096);
    write_html_head(&mut html, "Updates & Software - Server Manager");

    html.push_str(
        r#"
        <div class="header">
            <h1>Updates &amp; Software Management 🔄</h1>
            <form method="POST" action="/logout">
                <button type="submit" class="btn btn-logout">Logout</button>
            </form>
        </div>
        <div class="nav">
            <a href="/">Dashboard</a>
            <a href="/users">User Management</a>
            <a href="/updates">Updates &amp; Software</a>
        </div>

        <div style="background: #f8f9fa; padding: 20px; border-radius: 6px; border: 1px solid #e9ecef; margin-bottom: 20px;">
            <h3>One-Click System &amp; Stack Update</h3>
            <p>Pull the latest Docker container images for all enabled services and re-deploy the stack seamlessly.</p>
            <form method="POST" action="/api/system/update" onsubmit="return confirm('Are you sure you want to pull latest images and update all active services?');">
                <button type="submit" class="btn btn-primary" style="font-size: 1em; padding: 10px 20px;">🚀 Update Stack Now</button>
            </form>
        </div>
    "#,
    );

    write_html_foot(&mut html);
    Html(html).into_response()
}

async fn trigger_system_update(session: Session) -> impl IntoResponse {
    let user: SessionUser = match session.get(SESSION_KEY).await {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };

    if !matches!(user.role, Role::Admin) {
        return (StatusCode::FORBIDDEN, "Access Denied: Admin role required").into_response();
    }

    info!("Web UI triggering stack update: server_manager update");
    if let Ok(exe) = std::env::current_exe() {
        match Command::new(exe).arg("update").spawn() {
            Ok(mut child) => {
                tokio::spawn(async move {
                    if let Err(e) = child.wait().await {
                        error!("Failed to wait on update process: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Failed to spawn update command: {}", e);
            }
        }
    } else {
        error!("Failed to determine current executable path.");
    }

    Redirect::to("/updates").into_response()
}

async fn user_install_app_handler(
    State(state): State<SharedState>,
    session: Session,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let user: SessionUser = match session.get(SESSION_KEY).await {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };

    let mut cache = state.users_cache.write().await;
    let mut manager_clone = cache.manager.clone();
    let u_name = user.username.clone();
    let app_name = name.clone();

    let res = tokio::task::spawn_blocking(move || -> anyhow::Result<UserManager> {
        manager_clone.install_user_app(&u_name, &app_name)?;
        Ok(manager_clone)
    })
    .await
    .expect("Blocking task should not panic");

    if let Ok(new_manager) = res {
        cache.manager = new_manager;
        info!("User {} installed app {}", user.username, name);
    }

    Redirect::to("/").into_response()
}

async fn user_uninstall_app_handler(
    State(state): State<SharedState>,
    session: Session,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let user: SessionUser = match session.get(SESSION_KEY).await {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };

    let mut cache = state.users_cache.write().await;
    let mut manager_clone = cache.manager.clone();
    let u_name = user.username.clone();
    let app_name = name.clone();

    let res = tokio::task::spawn_blocking(move || -> anyhow::Result<UserManager> {
        manager_clone.uninstall_user_app(&u_name, &app_name)?;
        Ok(manager_clone)
    })
    .await
    .expect("Blocking task should not panic");

    if let Ok(new_manager) = res {
        cache.manager = new_manager;
        info!("User {} uninstalled app {}", user.username, name);
    }

    Redirect::to("/").into_response()
}

async fn user_profile_page(
    State(state): State<SharedState>,
    session: Session,
) -> impl IntoResponse {
    let user: SessionUser = match session.get(SESSION_KEY).await {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };

    let is_admin = matches!(user.role, Role::Admin);
    let user_manager = state.get_users().await;
    let current_user_opt = user_manager.get_user(&user.username);
    let quota_str = current_user_opt
        .and_then(|u| u.quota_gb)
        .map(|q| format!("{} GB", q))
        .unwrap_or_else(|| "Unlimited".to_string());

    let mut html = String::with_capacity(4096);
    write_html_head(&mut html, "My Profile - Server Manager");

    let _ = writeln!(
        html,
        r#"
        <div class="header">
            <h1>My Profile 👤</h1>
            <form method="POST" action="/logout">
                <button type="submit" class="btn btn-logout">Logout ({})</button>
            </form>
        </div>
        <div class="nav">
            <a href="/">Dashboard</a>
        "#,
        Escaped(&user.username)
    );

    if is_admin {
        html.push_str(
            r#"<a href="/users">User Management</a><a href="/updates">Updates &amp; Software</a>"#,
        );
    }

    let _ = writeln!(
        html,
        r#"
            <a href="/user/profile">My Profile</a>
        </div>

        <div style="background: #f8f9fa; padding: 20px; border-radius: 6px; border: 1px solid #e9ecef; margin-bottom: 20px;">
            <h3>User Information</h3>
            <p><strong>Username:</strong> {}</p>
            <p><strong>Role:</strong> {:?}</p>
            <p><strong>Storage Quota:</strong> {}</p>
        </div>

        <div style="background: #f8f9fa; padding: 20px; border-radius: 6px; border: 1px solid #e9ecef;">
            <h3>Change Password</h3>
            <form method="POST" action="/user/profile">
                <div style="margin-bottom: 10px;">
                    <label>New Password:</label><br>
                    <input type="password" name="password" required style="width: 100%; max-width: 300px; padding: 8px;">
                </div>
                <button type="submit" class="btn btn-primary">Update Password</button>
            </form>
        </div>
    "#,
        Escaped(&user.username),
        user.role,
        quota_str
    );

    write_html_foot(&mut html);
    Html(html).into_response()
}

#[derive(Deserialize)]
struct UserPasswdPayload {
    password: String,
}

async fn user_passwd_handler(
    State(state): State<SharedState>,
    session: Session,
    Form(payload): Form<UserPasswdPayload>,
) -> impl IntoResponse {
    let user: SessionUser = match session.get(SESSION_KEY).await {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };

    if payload.password.trim().is_empty() {
        return Redirect::to("/user/profile").into_response();
    }

    let mut cache = state.users_cache.write().await;
    let mut manager_clone = cache.manager.clone();
    let u_name = user.username.clone();
    let new_pass = payload.password.clone();

    let res = tokio::task::spawn_blocking(move || -> anyhow::Result<UserManager> {
        manager_clone.update_password(&u_name, &new_pass)?;
        Ok(manager_clone)
    })
    .await
    .expect("Blocking task should not panic");

    if let Ok(new_manager) = res {
        cache.manager = new_manager;
        info!("User {} updated their password", user.username);
    }

    Redirect::to("/user/profile").into_response()
}
