use crate::helpers::{spawn_app, TestApp};

#[tokio::test]
async fn health_check_test() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    println!("Address {}", &app.addr);
    let response = client
        .get(format!("{}/health_check", &app.addr))
        .send()
        .await
        .expect("Failed to execute request");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
