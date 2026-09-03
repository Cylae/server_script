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
        .route("/users/update/:username", post(update_user_handler))
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
        .layer(axum::middleware::from_fn(security_headers_middleware))
        .layer(session_layer)
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Starting Web UI on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn security_headers_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        axum::http::HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        axum::http::header::X_FRAME_OPTIONS,
        axum::http::HeaderValue::from_static("DENY"),
    );
    headers.insert(
        axum::http::header::X_XSS_PROTECTION,
        axum::http::HeaderValue::from_static("1; mode=block"),
    );
    headers.insert(
        axum::http::header::REFERRER_POLICY,
        axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        axum::http::header::STRICT_TRANSPORT_SECURITY,
        axum::http::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    headers.insert(
        axum::http::header::CONTENT_SECURITY_POLICY,
        axum::http::HeaderValue::from_static("default-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline'"),
    );
    response
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
                --bg-main: #090d16;
                --bg-card: #131c2e;
                --bg-input: #0e1626;
                --border-color: #26354f;
                --border-hover: #3b82f6;
                --text-main: #f1f5f9;
                --text-muted: #94a3b8;
                --accent-indigo: #6366f1;
            }
            * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, sans-serif; }
            body { background: var(--bg-main); color: var(--text-main); display: flex; justify-content: center; align-items: center; min-height: 100vh; padding: 20px; }
            .login-box { background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 20px; padding: 40px 32px; width: 100%; max-width: 400px; box-shadow: 0 20px 40px -10px rgba(0,0,0,0.5); }
            .login-title { font-size: 1.75rem; font-weight: 800; text-align: center; margin-bottom: 6px; color: var(--text-main); letter-spacing: -0.025em; }
            .login-subtitle { font-size: 0.875rem; color: var(--text-muted); text-align: center; margin-bottom: 28px; }
            .form-group { margin-bottom: 18px; }
            label { display: block; font-size: 0.85rem; font-weight: 600; color: var(--text-muted); margin-bottom: 6px; }
            input { width: 100%; padding: 12px 14px; border-radius: 10px; border: 1px solid var(--border-color); background: var(--bg-input); color: var(--text-main); font-size: 0.95rem; transition: all 0.2s; }
            input:focus { outline: none; border-color: var(--border-hover); box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.15); }
            button { width: 100%; padding: 12px; background: var(--accent-indigo); color: #ffffff; font-weight: 700; border: none; border-radius: 10px; cursor: pointer; font-size: 1rem; transition: all 0.2s ease-in-out; margin-top: 10px; box-shadow: 0 4px 12px rgba(99,102,241,0.25); }
            button:hover { background: #4f46e5; transform: translateY(-1px); }
        </style>
    </head>
    <body>
        <div class="login-box">
            <h2 class="login-title">Server Manager 🚀</h2>
            <div class="login-subtitle">Sign in to manage your server infrastructure</div>
            <form method="POST" action="/login">
                <div class="form-group">
                    <label>Username</label>
                    <input type="text" name="username" placeholder="Enter username" required autofocus>
                </div>
                <div class="form-group">
                    <label>Password</label>
                    <input type="password" name="password" placeholder="Enter password" required>
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
                --bg-main: #090d16;
                --bg-card: #131c2e;
                --bg-card-alt: #1a263d;
                --bg-input: #0e1626;
                --border-color: #26354f;
                --border-hover: #3b82f6;
                --text-main: #f1f5f9;
                --text-muted: #94a3b8;
                --accent-blue: #38bdf8;
                --accent-indigo: #6366f1;
                --accent-green: #34d399;
                --accent-red: #f87171;
                --accent-amber: #fbbf24;
                --accent-gray: #64748b;
            }}
            * {{ box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, sans-serif; }}
            body {{ background: var(--bg-main); color: var(--text-main); min-height: 100vh; padding: 32px 16px; line-height: 1.5; }}
            .container {{ max-width: 1140px; margin: 0 auto; background: var(--bg-card); padding: 32px; border-radius: 20px; border: 1px solid var(--border-color); box-shadow: 0 20px 40px -10px rgba(0,0,0,0.5); }}
            .header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 28px; flex-wrap: wrap; gap: 16px; border-bottom: 1px solid var(--border-color); padding-bottom: 20px; }}
            .header h1 {{ font-size: 1.85rem; font-weight: 800; color: var(--text-main); letter-spacing: -0.025em; display: flex; align-items: center; gap: 10px; }}
            .nav {{ display: flex; gap: 8px; margin-bottom: 28px; background: var(--bg-main); padding: 6px; border-radius: 12px; border: 1px solid var(--border-color); flex-wrap: wrap; }}
            .nav a {{ color: var(--text-muted); text-decoration: none; font-weight: 600; font-size: 0.9rem; padding: 8px 16px; border-radius: 8px; transition: all 0.2s ease-in-out; }}
            .nav a:hover {{ color: var(--text-main); background: var(--bg-card); }}
            .nav a.active {{ color: #ffffff; background: var(--accent-indigo); shadow: 0 4px 12px rgba(99,102,241,0.3); }}
            .stats-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 20px; margin-bottom: 32px; }}
            .stat-card {{ background: var(--bg-main); padding: 22px; border-radius: 16px; border: 1px solid var(--border-color); position: relative; overflow: hidden; }}
            .stat-label {{ font-size: 0.85rem; color: var(--text-muted); font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 8px; }}
            .stat-value {{ font-size: 1.75rem; font-weight: 800; color: var(--accent-blue); display: flex; justify-content: space-between; align-items: baseline; }}
            .progress-bar-bg {{ width: 100%; height: 8px; background: var(--border-color); border-radius: 4px; margin-top: 12px; overflow: hidden; }}
            .progress-bar-fill {{ height: 100%; background: linear-gradient(90deg, var(--accent-indigo), var(--accent-blue)); border-radius: 4px; transition: width 0.4s ease; }}
            .section-title {{ font-size: 1.35rem; font-weight: 700; margin-top: 36px; margin-bottom: 16px; color: var(--text-main); display: flex; align-items: center; gap: 10px; }}
            .card-panel {{ background: var(--bg-main); padding: 24px; border-radius: 16px; border: 1px solid var(--border-color); margin-bottom: 28px; }}
            table {{ width: 100%; border-collapse: separate; border-spacing: 0; margin-top: 12px; border-radius: 12px; overflow: hidden; border: 1px solid var(--border-color); }}
            th, td {{ padding: 14px 18px; text-align: left; border-bottom: 1px solid var(--border-color); font-size: 0.925rem; }}
            th {{ background: var(--bg-main); font-size: 0.8rem; color: var(--text-muted); font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; }}
            tr:last-child td {{ border-bottom: none; }}
            tr {{ background: var(--bg-card); transition: background 0.15s; }}
            tr:hover td {{ background: var(--bg-card-alt); }}
            .badge {{ display: inline-flex; align-items: center; padding: 4px 10px; border-radius: 20px; font-size: 0.775rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.04em; }}
            .badge-success {{ background: rgba(52, 211, 153, 0.15); color: var(--accent-green); border: 1px solid rgba(52, 211, 153, 0.3); }}
            .badge-danger {{ background: rgba(248, 113, 113, 0.15); color: var(--accent-red); border: 1px solid rgba(248, 113, 113, 0.3); }}
            .badge-admin {{ background: rgba(99, 102, 241, 0.15); color: #818cf8; border: 1px solid rgba(99, 102, 241, 0.3); }}
            .badge-observer {{ background: rgba(251, 191, 36, 0.15); color: var(--accent-amber); border: 1px solid rgba(251, 191, 36, 0.3); }}
            .btn {{ padding: 8px 16px; border-radius: 10px; font-weight: 600; font-size: 0.875rem; text-decoration: none; border: none; cursor: pointer; display: inline-flex; align-items: center; justify-content: center; transition: all 0.2s ease-in-out; gap: 6px; }}
            .btn-primary {{ background: var(--accent-indigo); color: #ffffff; box-shadow: 0 4px 12px rgba(99,102,241,0.25); }}
            .btn-primary:hover {{ background: #4f46e5; transform: translateY(-1px); }}
            .btn-danger {{ background: rgba(248, 113, 113, 0.2); color: var(--accent-red); border: 1px solid rgba(248, 113, 113, 0.4); }}
            .btn-danger:hover {{ background: var(--accent-red); color: #090d16; transform: translateY(-1px); }}
            .btn-enable {{ background: rgba(52, 211, 153, 0.2); color: var(--accent-green); border: 1px solid rgba(52, 211, 153, 0.4); }}
            .btn-enable:hover {{ background: var(--accent-green); color: #090d16; transform: translateY(-1px); }}
            .btn-disable {{ background: rgba(248, 113, 113, 0.2); color: var(--accent-red); border: 1px solid rgba(248, 113, 113, 0.4); }}
            .btn-disable:hover {{ background: var(--accent-red); color: #090d16; transform: translateY(-1px); }}
            .btn-logout {{ background: var(--bg-main); color: var(--text-muted); border: 1px solid var(--border-color); }}
            .btn-logout:hover {{ color: var(--text-main); border-color: var(--accent-gray); background: var(--bg-card-alt); }}
            input, select {{ width: 100%; padding: 10px 14px; border-radius: 10px; border: 1px solid var(--border-color); background: var(--bg-input); color: var(--text-main); font-size: 0.925rem; transition: all 0.2s; }}
            input:focus, select:focus {{ outline: none; border-color: var(--border-hover); box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.15); }}
            .grid-form {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 14px; align-items: end; }}
            .apps-grid {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 16px; margin-top: 16px; }}
            .app-card {{ background: var(--bg-main); border: 1px solid var(--border-color); padding: 20px; border-radius: 14px; display: flex; flex-direction: column; justify-content: space-between; gap: 16px; transition: border-color 0.2s; }}
            .app-card:hover {{ border-color: var(--accent-indigo); }}
            .app-info {{ display: flex; justify-content: space-between; align-items: flex-start; }}
            .app-title {{ font-weight: 700; font-size: 1.1rem; color: var(--text-main); }}
            .app-subtitle {{ font-size: 0.8rem; color: var(--text-muted); margin-top: 2px; }}
            @media (max-width: 768px) {{
                body {{ padding: 12px 8px; }}
                .container {{ padding: 18px; border-radius: 14px; }}
                table {{ display: block; overflow-x: auto; }}
                .stats-grid {{ grid-template-columns: repeat(2, 1fr); gap: 12px; }}
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
        match tokio::task::spawn_blocking(move || {
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
        {
            Ok(stats) => stats,
            Err(e) => {
                error!("Failed to join system stats task: {}", e);
                (0, 0, 0, 0, 0.0, 0, 0)
            }
        }
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
    html.push_str(r#"<div class="nav"><a href="/" class="active">Dashboard</a>"#);
    if is_admin {
        html.push_str(
            r#"<a href="/users">User Management</a><a href="/updates">Updates &amp; Software</a>"#,
        );
    }
    html.push_str(r#"<a href="/user/profile">My Profile</a></div>"#);

    let ram_pct = if ram_total > 0 {
        (ram_used as f64 / ram_total as f64 * 100.0).min(100.0)
    } else {
        0.0
    };
    let swap_pct = if swap_total > 0 {
        (swap_used as f64 / swap_total as f64 * 100.0).min(100.0)
    } else {
        0.0
    };
    let disk_pct = if disk_total > 0 {
        (disk_used as f64 / disk_total as f64 * 100.0).min(100.0)
    } else {
        0.0
    };

    // Stats Grid
    let _ = writeln!(
        html,
        r#"
        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-label">CPU Usage</div>
                <div class="stat-value"><span>{:.1}%</span></div>
                <div class="progress-bar-bg"><div class="progress-bar-fill" style="width: {:.1}%;"></div></div>
            </div>
            <div class="stat-card">
                <div class="stat-label">RAM Usage</div>
                <div class="stat-value"><span>{} MB</span><small style="font-size: 0.85rem; color: var(--text-muted);">/ {} MB</small></div>
                <div class="progress-bar-bg"><div class="progress-bar-fill" style="width: {:.1}%;"></div></div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Swap Usage</div>
                <div class="stat-value"><span>{} MB</span><small style="font-size: 0.85rem; color: var(--text-muted);">/ {} MB</small></div>
                <div class="progress-bar-bg"><div class="progress-bar-fill" style="width: {:.1}%;"></div></div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Disk Storage (/)</div>
                <div class="stat-value"><span>{} GB</span><small style="font-size: 0.85rem; color: var(--text-muted);">/ {} GB</small></div>
                <div class="progress-bar-bg"><div class="progress-bar-fill" style="width: {:.1}%;"></div></div>
            </div>
        </div>
    "#,
        cpu_usage,
        cpu_usage,
        ram_used,
        ram_total,
        ram_pct,
        swap_used,
        swap_total,
        swap_pct,
        disk_used,
        disk_total,
        disk_pct
    );

    // System Services Table
    html.push_str(
        r#"
        <div class="section-title">⚡ System Stack Services</div>
        <table>
            <thead>
                <tr>
                    <th>Service Name</th>
                    <th>Docker Image</th>
                    <th>Port(s)</th>
                    <th>System Status</th>
                    <th>System Action</th>
                </tr>
            </thead>
            <tbody>
    "#,
    );

    for svc in services {
        let name = svc.name();
        let enabled = config.is_enabled(name);
        let ports_str = svc.ports().join(", ");
        let display_ports = if ports_str.is_empty() {
            "-"
        } else {
            &ports_str
        };

        let status_badge = if enabled {
            r#"<span class="badge badge-success">Enabled</span>"#
        } else {
            r#"<span class="badge badge-danger">Disabled</span>"#
        };

        let _ = writeln!(
            html,
            r#"
            <tr>
                <td><strong>{}</strong></td>
                <td><code style="background: var(--bg-main); padding: 2px 6px; border-radius: 6px; font-size: 0.825rem; border: 1px solid var(--border-color);">{}</code></td>
                <td>{}</td>
                <td>{}</td>
                <td>
        "#,
            name,
            svc.image(),
            display_ports,
            status_badge
        );

        if is_admin {
            let _ = writeln!(
                html,
                r#"
                    <form method="POST" action="/api/services/{}/{}" style="margin:0;">
                        <button type="submit" class="btn {}">{}</button>
                    </form>
             "#,
                name,
                if enabled { "disable" } else { "enable" },
                if enabled { "btn-disable" } else { "btn-enable" },
                if enabled { "Disable" } else { "Enable" }
            );
        } else {
            html.push_str(r#"<span style="color: var(--text-muted); font-size: 0.85rem;">System Controlled</span>"#);
        };

        html.push_str("</td></tr>");
    }
    html.push_str("</tbody></table>");

    // User App Management Section (Quickbox.io style)
    let user_manager = state.get_users().await;
    let current_user_opt = user_manager.get_user(&user.username);
    let installed_apps = current_user_opt
        .map(|u| u.installed_apps.clone())
        .unwrap_or_default();

    html.push_str(
        r#"
        <div class="section-title">📦 Personal Apps Portal (1-Click Apps)</div>
        <p style="color: var(--text-muted); margin-bottom: 16px; font-size: 0.925rem;">Install and manage individual user applications for your profile.</p>
        <div class="apps-grid">
    "#,
    );

    for svc in services {
        let name = svc.name();
        let is_app_installed = installed_apps.contains(name);
        let status_badge = if is_app_installed {
            r#"<span class="badge badge-success">Installed</span>"#
        } else {
            r#"<span class="badge badge-danger">Not Installed</span>"#
        };

        let _ = writeln!(
            html,
            r#"
            <div class="app-card">
                <div class="app-info">
                    <div>
                        <div class="app-title">{}</div>
                        <div class="app-subtitle">{}</div>
                    </div>
                    {}
                </div>
                <div>
                    <form method="POST" action="/user/apps/{}/{}" style="margin: 0; width: 100%;">
                        <button type="submit" class="btn {}" style="width: 100%;">{}</button>
                    </form>
                </div>
            </div>
            "#,
            name,
            svc.image(),
            status_badge,
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
                "Uninstall App"
            } else {
                "1-Click Install"
            }
        );
    }

    html.push_str("</div>");
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
            <form method="POST" action="/logout" style="margin: 0;">
                <button type="submit" class="btn btn-logout">Logout</button>
            </form>
        </div>
        <div class="nav">
            <a href="/">Dashboard</a>
            <a href="/users" class="active">User Management</a>
            <a href="/updates">Updates &amp; Software</a>
            <a href="/user/profile">My Profile</a>
        </div>

        <div class="card-panel">
            <div class="section-title" style="margin-top: 0;">➕ Add New System User</div>
            <form method="POST" action="/users/add" class="grid-form">
                <div>
                    <label style="font-size: 0.825rem; font-weight: 600; color: var(--text-muted); display: block; margin-bottom: 6px;">Username</label>
                    <input type="text" name="username" required placeholder="e.g. john">
                </div>
                <div>
                    <label style="font-size: 0.825rem; font-weight: 600; color: var(--text-muted); display: block; margin-bottom: 6px;">Password</label>
                    <input type="password" name="password" required placeholder="••••••••">
                </div>
                <div>
                    <label style="font-size: 0.825rem; font-weight: 600; color: var(--text-muted); display: block; margin-bottom: 6px;">Role</label>
                    <select name="role">
                        <option value="Observer">Observer</option>
                        <option value="Admin">Admin</option>
                    </select>
                </div>
                <div>
                    <label style="font-size: 0.825rem; font-weight: 600; color: var(--text-muted); display: block; margin-bottom: 6px;">Quota (GB) <small style="color: var(--text-muted);">(0 = unlimited)</small></label>
                    <input type="number" name="quota" value="0">
                </div>
                <button type="submit" class="btn btn-primary" style="height: 42px;">Add User</button>
            </form>
        </div>

        <div class="section-title">👥 Existing User Accounts</div>
        <table>
            <thead>
                <tr>
                    <th>Username</th>
                    <th>Role</th>
                    <th>Storage Quota</th>
                    <th>Installed Apps</th>
                    <th>Update Settings</th>
                    <th>Actions</th>
                </tr>
            </thead>
            <tbody>
    "#);

    for u in user_manager.list_users() {
        let role_badge = match u.role {
            Role::Admin => r#"<span class="badge badge-admin">Admin</span>"#,
            Role::Observer => r#"<span class="badge badge-observer">Observer</span>"#,
        };

        let quota_val = u.quota_gb.unwrap_or(0);
        let apps_count = u.installed_apps.len();
        let apps_display = if apps_count > 0 {
            format!("{} app(s)", apps_count)
        } else {
            "None".to_string()
        };

        let _ = writeln!(
            html,
            r#"
            <tr>
                <td><strong>{}</strong></td>
                <td>{}</td>
                <td>{}</td>
                <td><span class="badge" style="background: var(--bg-main); border: 1px solid var(--border-color); color: var(--text-main);">{}</span></td>
                <td>
                    <form method="POST" action="/users/update/{}" style="display: flex; gap: 8px; align-items: center; margin: 0;">
                        <select name="role" style="padding: 6px 10px; font-size: 0.85rem; width: 110px;">
                            <option value="Observer"{}>Observer</option>
                            <option value="Admin"{}>Admin</option>
                        </select>
                        <input type="number" name="quota" value="{}" style="padding: 6px 10px; font-size: 0.85rem; width: 90px;" placeholder="Quota">
                        <button type="submit" class="btn btn-primary" style="padding: 6px 12px; font-size: 0.8rem;">Save</button>
                    </form>
                </td>
                <td>
                    <form method="POST" action="/users/delete/{}" style="margin:0;" onsubmit="return confirm('Are you sure you want to delete user {}?');">
                        <button type="submit" class="btn btn-danger" style="padding: 6px 12px; font-size: 0.8rem;">Delete</button>
                    </form>
                </td>
            </tr>
        "#,
            Escaped(&u.username),
            role_badge,
            if quota_val > 0 {
                format!("{} GB", quota_val)
            } else {
                "Unlimited".to_string()
            },
            apps_display,
            Escaped(&u.username),
            if matches!(u.role, Role::Observer) {
                " selected"
            } else {
                ""
            },
            if matches!(u.role, Role::Admin) {
                " selected"
            } else {
                ""
            },
            quota_val,
            Escaped(&u.username),
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

    let res = match tokio::task::spawn_blocking(move || -> anyhow::Result<UserManager> {
        manager_clone.add_user(&user_name, &pass, role_enum, quota_val)?;
        Ok(manager_clone)
    })
    .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("Blocking task join error in add_user: {}", e);
            Err(anyhow::anyhow!(
                "Internal server error: failed to join background task"
            ))
        }
    };

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

#[derive(Deserialize)]
struct UpdateUserPayload {
    role: String,
    quota: Option<u64>,
}

async fn update_user_handler(
    State(state): State<SharedState>,
    session: Session,
    Path(username): Path<String>,
    Form(payload): Form<UpdateUserPayload>,
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

    let u_name = username.clone();
    let res = match tokio::task::spawn_blocking(move || -> anyhow::Result<UserManager> {
        manager_clone.update_user_role_and_quota(&u_name, role_enum, quota_val)?;
        Ok(manager_clone)
    })
    .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("Blocking task join error in update_user: {}", e);
            Err(anyhow::anyhow!(
                "Internal server error: failed to join background task"
            ))
        }
    };

    match res {
        Ok(new_manager) => {
            cache.manager = new_manager;
            info!(
                "User {} updated via Web UI by {}",
                username, session_user.username
            );
            let path = std::path::Path::new("users.yaml");
            let fallback_path = std::path::Path::new("/opt/server_manager/users.yaml");
            let file_path = if path.exists() { path } else { fallback_path };
            if let Ok(m) = std::fs::metadata(file_path) {
                cache.last_modified = m.modified().ok();
            }
        }
        Err(e) => {
            error!("Failed to update user: {}", e);
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
    let res = match tokio::task::spawn_blocking(move || -> anyhow::Result<UserManager> {
        manager_clone.delete_user(&u_name)?;
        Ok(manager_clone)
    })
    .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("Blocking task join error in delete_user: {}", e);
            Err(anyhow::anyhow!(
                "Internal server error: failed to join background task"
            ))
        }
    };

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
            <form method="POST" action="/logout" style="margin:0;">
                <button type="submit" class="btn btn-logout">Logout</button>
            </form>
        </div>
        <div class="nav">
            <a href="/">Dashboard</a>
            <a href="/users">User Management</a>
            <a href="/updates" class="active">Updates &amp; Software</a>
            <a href="/user/profile">My Profile</a>
        </div>

        <div class="card-panel">
            <div class="section-title" style="margin-top:0;">🚀 One-Click System &amp; Stack Update</div>
            <p style="color: var(--text-muted); margin-bottom: 20px; font-size: 0.95rem; line-height: 1.6;">
                Pull the latest Docker container images for all active media services and seamlessly re-deploy the container stack without downtime.
            </p>
            <form method="POST" action="/api/system/update" onsubmit="return confirm('Are you sure you want to pull latest images and update all active services?');" style="margin:0;">
                <button type="submit" class="btn btn-primary" style="font-size: 0.95rem; padding: 12px 24px;">🚀 Update Stack Now</button>
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

    let res = match tokio::task::spawn_blocking(move || -> anyhow::Result<UserManager> {
        manager_clone.install_user_app(&u_name, &app_name)?;
        Ok(manager_clone)
    })
    .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("Blocking task join error in install_user_app: {}", e);
            Err(anyhow::anyhow!(
                "Internal server error: failed to join background task"
            ))
        }
    };

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

    let res = match tokio::task::spawn_blocking(move || -> anyhow::Result<UserManager> {
        manager_clone.uninstall_user_app(&u_name, &app_name)?;
        Ok(manager_clone)
    })
    .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("Blocking task join error in uninstall_user_app: {}", e);
            Err(anyhow::anyhow!(
                "Internal server error: failed to join background task"
            ))
        }
    };

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

    let role_badge = match user.role {
        Role::Admin => r#"<span class="badge badge-admin">Admin</span>"#,
        Role::Observer => r#"<span class="badge badge-observer">Observer</span>"#,
    };

    let _ = writeln!(
        html,
        r#"
        <div class="header">
            <h1>My Profile 👤</h1>
            <form method="POST" action="/logout" style="margin:0;">
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
            <a href="/user/profile" class="active">My Profile</a>
        </div>

        <div class="card-panel">
            <div class="section-title" style="margin-top:0;">👤 Account Details</div>
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 16px; margin-top: 16px;">
                <div>
                    <div style="font-size: 0.8rem; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em;">Username</div>
                    <div style="font-size: 1.1rem; font-weight: 700; margin-top: 4px;">{}</div>
                </div>
                <div>
                    <div style="font-size: 0.8rem; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em;">Access Role</div>
                    <div style="margin-top: 4px;">{}</div>
                </div>
                <div>
                    <div style="font-size: 0.8rem; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em;">Disk Quota</div>
                    <div style="font-size: 1.1rem; font-weight: 700; margin-top: 4px; color: var(--accent-blue);">{}</div>
                </div>
            </div>
        </div>

        <div class="card-panel">
            <div class="section-title" style="margin-top:0;">🔐 Security &amp; Password Update</div>
            <form method="POST" action="/user/profile" style="max-width: 400px; margin-top: 16px;">
                <div style="margin-bottom: 16px;">
                    <label style="font-size: 0.85rem; font-weight: 600; color: var(--text-muted); display: block; margin-bottom: 6px;">New Password</label>
                    <input type="password" name="password" required placeholder="Enter new password">
                </div>
                <button type="submit" class="btn btn-primary">Update Password</button>
            </form>
        </div>
    "#,
        Escaped(&user.username),
        role_badge,
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

    let res = match tokio::task::spawn_blocking(move || -> anyhow::Result<UserManager> {
        manager_clone.update_password(&u_name, &new_pass)?;
        Ok(manager_clone)
    })
    .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("Blocking task join error in update_password: {}", e);
            Err(anyhow::anyhow!(
                "Internal server error: failed to join background task"
            ))
        }
    };

    if let Ok(new_manager) = res {
        cache.manager = new_manager;
        info!("User {} updated their password", user.username);
    }

    Redirect::to("/user/profile").into_response()
}
