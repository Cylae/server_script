use axum::{
    extract::{Path, Form},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
    http::StatusCode,
};
use std::net::SocketAddr;
use crate::services;
use crate::core::config::Config;
use crate::core::users::{UserManager, Role};
use std::process::Command;
use log::{info, error, warn};
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer};
use serde::{Deserialize, Serialize};
use time::Duration;

#[derive(Serialize, Deserialize, Clone)]
struct SessionUser {
    username: String,
    role: Role,
}

const SESSION_KEY: &str = "user";

pub async fn start_server(port: u16) -> anyhow::Result<()> {
    // Session setup
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Localhost/LAN, http usually
        .with_expiry(Expiry::OnInactivity(Duration::hours(24)));

    let app = Router::new()
        .route("/", get(dashboard))
        .route("/api/services/:name/enable", post(enable_service))
        .route("/api/services/:name/disable", post(disable_service))
        .route("/logout", post(logout))
        .route("/login", get(login_page).post(login_handler))
        .layer(session_layer);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Starting Web UI on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn login_page(session: Session) -> impl IntoResponse {
    if let Some(_user) = session.get::<SessionUser>(SESSION_KEY).await.unwrap_or(None) {
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

async fn login_handler(session: Session, Form(payload): Form<LoginPayload>) -> impl IntoResponse {
    // Reload users on login attempt to get fresh data
    let user_manager = UserManager::load().unwrap_or_default();

    if let Some(user) = user_manager.verify(&payload.username, &payload.password) {
        let session_user = SessionUser {
            username: user.username,
            role: user.role,
        };
        session.insert(SESSION_KEY, session_user).await.expect("Failed to insert session");
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

// Protected Dashboard
async fn dashboard(session: Session) -> impl IntoResponse {
    let user: SessionUser = match session.get(SESSION_KEY).await {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };

    let services = services::get_all_services();

    let config = match Config::load() {
        Ok(c) => c,
        Err(_) => {
            if let Ok(content) = std::fs::read_to_string("/opt/server_manager/config.yaml") {
                serde_yaml::from_str(&content).unwrap_or_default()
            } else {
                Config::default()
            }
        }
    };

    let is_admin = matches!(user.role, Role::Admin);
    let escaped_username = user.username.replace('&', "&amp;")
                                        .replace('<', "&lt;")
                                        .replace('>', "&gt;")
                                        .replace('"', "&quot;")
                                        .replace('\'', "&#39;");

    let mut html = format!(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Server Manager Dashboard</title>
        <style>
            body {{ font-family: sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
            table {{ width: 100%; border-collapse: collapse; margin-top: 20px; }}
            th, td {{ padding: 10px; border-bottom: 1px solid #ddd; text-align: left; }}
            th {{ background-color: #f4f4f4; }}
            .btn {{ padding: 5px 10px; text-decoration: none; border-radius: 4px; color: white; border: none; cursor: pointer; }}
            .btn-enable {{ background-color: #28a745; }}
            .btn-disable {{ background-color: #dc3545; }}
            .btn-logout {{ background-color: #6c757d; float: right; }}
            .status-enabled {{ color: #28a745; font-weight: bold; }}
            .status-disabled {{ color: #dc3545; font-weight: bold; }}
            .header {{ overflow: hidden; margin-bottom: 20px; }}
        </style>
    </head>
    <body>
        <div class="header">
            <h1 style="float: left;">Server Manager 🚀</h1>
            <form method="POST" action="/logout" style="float: right; margin-top: 20px;">
                <button type="submit" class="btn btn-logout">Logout ({})</button>
            </form>
        </div>
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
    "#, escaped_username);

    for svc in services {
        let name = svc.name();
        let enabled = config.is_enabled(name);
        let status_class = if enabled { "status-enabled" } else { "status-disabled" };
        let status_text = if enabled { "Enabled" } else { "Disabled" };

        let action_html = if is_admin {
             format!(r#"
                    <form method="POST" action="/api/services/{}/{}">
                        <button type="submit" class="btn {}">{}</button>
                    </form>
             "#,
             name,
             if enabled { "disable" } else { "enable" },
             if enabled { "btn-disable" } else { "btn-enable" },
             if enabled { "Disable" } else { "Enable" }
             )
        } else {
            "<span>Read-only</span>".to_string()
        };

        html.push_str(&format!(r#"
            <tr>
                <td>{}</td>
                <td>{}</td>
                <td class="{}">{}</td>
                <td>
                   {}
                </td>
            </tr>
        "#,
        name,
        svc.image(),
        status_class,
        status_text,
        action_html
        ));
    }

    html.push_str(r#"
            </tbody>
        </table>
        <p><em>Note: Actions may take a moment to apply as Docker Compose updates.</em></p>
    </body>
    </html>
    "#);

    Html(html).into_response()
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
        let _ = Command::new(exe)
            .arg(action)
            .arg(service)
            .spawn();
    } else {
        error!("Failed to determine current executable path.");
    }
}
