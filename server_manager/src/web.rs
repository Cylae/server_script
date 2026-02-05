use axum::{
    extract::Path,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use std::fmt::Write;
use std::net::SocketAddr;
use crate::services;
use crate::core::config::Config;
use std::process::Command;
use log::{info, error};
use std::fmt::Write;

pub async fn start_server(port: u16) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(dashboard))
        .route("/api/services/:name/enable", post(enable_service))
        .route("/api/services/:name/disable", post(disable_service));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Starting Web UI on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn dashboard() -> impl IntoResponse {
    let services = services::get_all_services();

    // Attempt to load config, default to all enabled if fail
    // Check both CWD and /opt/server_manager
    let config = match Config::load_async().await {
        Ok(c) => c,
        Err(_) => {
            // Try explicit path
            if let Ok(content) = tokio::fs::read_to_string("/opt/server_manager/config.yaml").await {
                serde_yaml_ng::from_str(&content).unwrap_or_default()
            } else {
                Config::default()
            }
        }
    };

    Html(render_dashboard_html(&services, &config))
}

fn render_dashboard_html(services: &[Box<dyn crate::services::Service>], config: &Config) -> String {
    let mut html = String::from(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Server Manager Dashboard</title>
        <style>
            body { font-family: sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }
            table { width: 100%; border-collapse: collapse; margin-top: 20px; }
            th, td { padding: 10px; border-bottom: 1px solid #ddd; text-align: left; }
            th { background-color: #f4f4f4; }
            .btn { padding: 5px 10px; text-decoration: none; border-radius: 4px; color: white; border: none; cursor: pointer; }
            .btn-enable { background-color: #28a745; }
            .btn-disable { background-color: #dc3545; }
            .status-enabled { color: #28a745; font-weight: bold; }
            .status-disabled { color: #dc3545; font-weight: bold; }
        </style>
    </head>
    <body>
        <h1>Server Manager Dashboard 🚀</h1>
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
    "#);

    for svc in services {
        let name = svc.name();
        let enabled = config.is_enabled(name);
        let status_class = if enabled { "status-enabled" } else { "status-disabled" };
        let status_text = if enabled { "Enabled" } else { "Disabled" };

        let _ = write!(html, r#"
            <tr>
                <td>{}</td>
                <td>{}</td>
                <td class="{}">{}</td>
                <td>
                    <form method="POST" action="/api/services/{}/{}">
                        <button type="submit" class="btn {}">{}</button>
                    </form>
                </td>
            </tr>
        "#,
        name,
        svc.image(),
        status_class,
        status_text,
        name,
        if enabled { "disable" } else { "enable" },
        if enabled { "btn-disable" } else { "btn-enable" },
        if enabled { "Disable" } else { "Enable" }
        );
    }

    html.push_str(r#"
            </tbody>
        </table>
        <p><em>Note: Actions may take a moment to apply as Docker Compose updates.</em></p>
    </body>
    </html>
    "#);

    html
}

async fn enable_service(Path(name): Path<String>) -> impl IntoResponse {
    run_cli_toggle(&name, true);
    Redirect::to("/")
}

async fn disable_service(Path(name): Path<String>) -> impl IntoResponse {
    run_cli_toggle(&name, false);
    Redirect::to("/")
}

// Helper to invoke the CLI logic (or essentially re-run the toggle logic)
// Since this is running inside the same binary, we could call the function directly,
// but `run_toggle_service` is in main.rs (binary crate).
// For simplicity/decoupling, we can spawn a subprocess calling ourself,
// OR simpler: Move the toggle logic to lib.rs or duplicate/invoke here.
// But `main.rs` functions are not exported.
//
// SOLUTION: We will shell out to `server_manager enable/disable <service>`
// This ensures we run in a clean context/process and uses the CLI lock mechanisms if any (none currently).
fn run_cli_toggle(service: &str, enable: bool) {
    let action = if enable { "enable" } else { "disable" };
    info!("Web UI triggering: server_manager {} {}", action, service);

    // We assume `server_manager` is in PATH or current dir.
    // Use `std::env::current_exe()` to be safe.
    if let Ok(exe) = std::env::current_exe() {
        let _ = Command::new(exe)
            .arg(action)
            .arg(service)
            .spawn(); // Spawn async/detached so we don't block the web request too long
    } else {
        error!("Failed to determine current executable path.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_render_content() {
        let services = crate::services::get_all_services();
        let config = Config::default();
        let html = render_dashboard_html(&services, &config);

        assert!(html.contains("Server Manager Dashboard"));
        assert!(html.contains("<td>plex</td>"));
        assert!(html.contains("<th>Service</th>"));
    }

    #[test]
    fn benchmark_render() {
        let services = crate::services::get_all_services();
        let config = Config::default();

        let start = Instant::now();
        let iterations = 10_000;

        for _ in 0..iterations {
            let _ = render_dashboard_html(&services, &config);
        }

        let elapsed = start.elapsed();
        println!("Benchmark: {:?} for {} iterations", elapsed, iterations);
    }
}
