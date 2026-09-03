use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use server_manager::core::config::Config;
use server_manager::core::users::UserManager;
use server_manager::interface::web::{build_app, AppState};
use tower::ServiceExt;

fn test_router() -> axum::Router {
    let state = AppState::new_test(Config::default(), UserManager::default());
    build_app(state)
}

#[tokio::test]
async fn test_get_login_page_returns_200() {
    let app = test_router();
    let request = Request::builder()
        .uri("/login")
        .method("GET")
        .body(Body::empty())
        .expect("build request");

    let response = app.oneshot(request).await.expect("execute request");
    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
    assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
    assert!(headers.contains_key("content-security-policy"));
}

#[tokio::test]
async fn test_unauthenticated_dashboard_redirects_to_login() {
    let app = test_router();
    let request = Request::builder()
        .uri("/")
        .method("GET")
        .body(Body::empty())
        .expect("build request");

    let response = app.oneshot(request).await.expect("execute request");
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("location").unwrap(), "/login");
}

#[tokio::test]
async fn test_unauthenticated_users_page_redirects_to_login() {
    let app = test_router();
    let request = Request::builder()
        .uri("/users")
        .method("GET")
        .body(Body::empty())
        .expect("build request");

    let response = app.oneshot(request).await.expect("execute request");
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("location").unwrap(), "/login");
}

#[tokio::test]
async fn test_unauthenticated_audit_page_redirects_to_login() {
    let app = test_router();
    let request = Request::builder()
        .uri("/audit")
        .method("GET")
        .body(Body::empty())
        .expect("build request");

    let response = app.oneshot(request).await.expect("execute request");
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("location").unwrap(), "/login");
}

#[tokio::test]
async fn test_unauthenticated_profile_page_redirects_to_login() {
    let app = test_router();
    let request = Request::builder()
        .uri("/user/profile")
        .method("GET")
        .body(Body::empty())
        .expect("build request");

    let response = app.oneshot(request).await.expect("execute request");
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("location").unwrap(), "/login");
}

#[tokio::test]
async fn test_unauthenticated_api_service_enable_redirects_to_login() {
    let app = test_router();
    let request = Request::builder()
        .uri("/api/services/plex/enable")
        .method("POST")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("csrf_token=test"))
        .expect("build request");

    let response = app.oneshot(request).await.expect("execute request");
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("location").unwrap(), "/login");
}
