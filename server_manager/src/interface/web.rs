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
        let current_mtime = tokio::fs::metadata(file_path).await
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
        let current_mtime_2 = tokio::fs::metadata(file_path).await
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
    let sys = System::new();
    // We do NOT refresh everything here to save startup time.
    // Dashboard will trigger refresh on first access because we init last_system_refresh to EPOCH.

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
        last_system_refresh: Mutex::new(SystemTime::UNIX_EPOCH),
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
        .route("/api/services/:name/enable", post(enable_service))
        .route("/api/services/:name/disable", post(disable_service))
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
        .unwrap_or(None)
    {
        return Redirect::to("/").into_response();
    }

    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Login - Server Manager</title>
        <style>
            body { font-family: sans-serif; display: flex; justify-content: center; align-items: center; height: 100vh; background-color: #f4f4f4; }
            .login-box { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 0 10px rgba(0,0,0,0.1); width: 300px; }
            input { width: 100%; padding: 10px; margin: 10px 0; box-sizing: border-box; }
            button { width: 100%; padding: 10px; background-color: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer; }
            button:hover { background-color: #0056b3; }
        </style>
    </head>
    <body>
        <div class="login-box">
            <h2 style="text-align: center;">Login</h2>
            <form method="POST" action="/login">
                <input type="text" name="username" placeholder="Username" required>
                <input type="password" name="password" placeholder="Password" required>
                <button type="submit">Login</button>
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

async fn login_handler(State(state): State<SharedState>, session: Session, Form(payload): Form<LoginPayload>) -> impl IntoResponse {
    // Reload users on login attempt to get fresh data
    let user_manager = state.get_users().await;

    if let Some(user) = user_manager.verify_async(&payload.username, &payload.password).await {
        let session_user = SessionUser {
            username: user.username,
            role: user.role,
        };
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
    let _ = write!(out, r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>{}</title>
        <style>
            body {{ font-family: sans-serif; max-width: 900px; margin: 0 auto; padding: 20px; background-color: #f9f9f9; }}
            .container {{ background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 5px rgba(0,0,0,0.1); }}
            table {{ width: 100%; border-collapse: collapse; margin-top: 20px; }}
            th, td {{ padding: 12px; border-bottom: 1px solid #eee; text-align: left; }}
            th {{ background-color: #f4f4f4; }}
            .btn {{ padding: 6px 12px; text-decoration: none; border-radius: 4px; color: white; border: none; cursor: pointer; display: inline-block; }}
            .btn-primary {{ background-color: #007bff; }}
            .btn-danger {{ background-color: #dc3545; }}
            .btn-enable {{ background-color: #28a745; }}
            .btn-disable {{ background-color: #dc3545; }}
            .btn-logout {{ background-color: #6c757d; }}
            .status-enabled {{ color: #28a745; font-weight: bold; }}
            .status-disabled {{ color: #dc3545; font-weight: bold; }}
            .header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }}
            .nav {{ margin-bottom: 20px; padding-bottom: 10px; border-bottom: 1px solid #ddd; }}
            .nav a {{ margin-right: 15px; text-decoration: none; color: #333; font-weight: bold; }}
            .nav a:hover {{ color: #007bff; }}
            .stats-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 20px; }}
            .stat-card {{ background: #f8f9fa; padding: 15px; border-radius: 6px; border: 1px solid #e9ecef; }}
            .stat-value {{ font-size: 1.5em; font-weight: bold; color: #007bff; }}
        </style>
    </head>
    <body>
        <div class="container">
    "#, title);
}

fn write_html_foot(out: &mut String) {
    out.push_str(r#"
        </div>
    </body>
    </html>
    "#);
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
    let mut sys = state.system.lock().unwrap();
    let now = SystemTime::now();
    let mut last_refresh = state.last_system_refresh.lock().unwrap();

    // Throttle refresh to max once every 500ms
    if now
        .duration_since(*last_refresh)
        .unwrap_or_default()
        .as_millis()
        > 500
    {
        sys.refresh_cpu();
        sys.refresh_memory();
        sys.refresh_disks_list();
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
        disk_used = (disk.total_space() - disk.available_space()) / 1024 / 1024 / 1024; // GB
    }
    drop(sys); // Release lock explicitely

    let mut html = String::with_capacity(8192);
    write_html_head(&mut html, "Dashboard - Server Manager");

    let _ = write!(
        html,
        r#"
        <div class="header">
            <h1>Server Manager 🚀</h1>
            <form method="POST" action="/logout" style="margin: 0;">
                <button type="submit" class="btn btn-logout">Logout ({})</button>
            </form>
        </div>
    "#, Escaped(&user.username));

    // Navigation
    if is_admin {
        html.push_str(
            r#"
        <div class="nav">
            <a href="/">Dashboard</a>
            <a href="/users">User Management</a>
        </div>
        "#,
        );
    }

    // Stats Grid
    let _ = write!(
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

        let _ = write!(
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
            let _ = write!(
                html,
                r#"
                    <form method="POST" action="/api/services/{}/{}">
                        <button type="submit" class="btn {}">{}</button>
                    </form>
             "#,
                name,
                if enabled { "disable" } else { "enable" },
                if enabled { "btn-disable" } else { "btn-enable" },
                if enabled { "Disable" } else { "Enable" }
            );
        } else {
            html.push_str("<span>Read-only</span>");
        };

        html.push_str("</td></tr>");
    }

    html.push_str(
        r#"
            </tbody>
        </table>
        <p><em>Note: Actions may take a moment to apply.</em></p>
    "#);
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
        let _ = write!(
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
                let _ = write!(html, "{} GB", gb);
            }
            _ => {
                html.push_str("Unlimited");
            }
        }

        // Don't allow deleting self or last admin logic is handled in delete handler/manager
        // But let's show delete button generally
        let _ = write!(html, r#"</td>
                <td>
                    <form method="POST" action="/users/delete/{}" onsubmit="return confirm('Are you sure you want to delete this user? This will delete their system account and data.');">
                        <button type="submit" class="btn btn-danger">Delete</button>
                    </form>
                </td>
            </tr>
        "#, Escaped(&u.username));
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

async fn add_user_handler(State(state): State<SharedState>, session: Session, Form(payload): Form<AddUserPayload>) -> impl IntoResponse {
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
    let mut manager = cache.manager.clone();
    let username = payload.username.clone();
    let password = payload.password.clone();

    // Perform heavy lifting (hashing, useradd) in a blocking thread
    let res = tokio::task::spawn_blocking(move || -> anyhow::Result<UserManager> {
        manager.add_user(&username, &password, role_enum, quota_val)?;
        Ok(manager)
    })
    .await;

    match res {
        Ok(Ok(updated_manager)) => {
            cache.manager = updated_manager;
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
        Ok(Err(e)) => {
            error!("Failed to add user: {}", e);
        }
        Err(e) => {
            error!("Task join error: {}", e);
        }
    }

    Redirect::to("/users").into_response()
}

async fn delete_user_handler(State(state): State<SharedState>, session: Session, Path(username): Path<String>) -> impl IntoResponse {
    let session_user: SessionUser = match session.get(SESSION_KEY).await {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };

    if !matches!(session_user.role, Role::Admin) {
        return (StatusCode::FORBIDDEN, "Access Denied").into_response();
    }

    let mut cache = state.users_cache.write().await;
    let mut manager = cache.manager.clone();
    let username_clone = username.clone();

    // Perform heavy lifting (userdel) in a blocking thread
    let res = tokio::task::spawn_blocking(move || -> anyhow::Result<UserManager> {
        manager.delete_user(&username_clone)?;
        Ok(manager)
    })
    .await;

    match res {
        Ok(Ok(updated_manager)) => {
            cache.manager = updated_manager;
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
        Ok(Err(e)) => {
            error!("Failed to delete user: {}", e);
        }
        Err(e) => {
            error!("Task join error: {}", e);
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
