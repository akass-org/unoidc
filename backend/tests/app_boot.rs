#[tokio::test]
async fn app_builds_router() {
    let app = backend::build_app().await;
    let _ = app;
}
