use linkify::LinkKind;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{spawn_app, TestApp};

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=arun%20manivannan&email=arun%40arun.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let response = app.post_subscription(body.to_string()).await;

    let saved = sqlx::query!("select email, name from subscriptions",)
        .fetch_one(&app.pool)
        .await
        .expect("Failed to fetch saved subscrptions");
    assert_eq!(200, response.status().as_u16());
    assert_eq!(saved.email, "arun@arun.com");
    assert_eq!(saved.name, "arun manivannan");
}

#[tokio::test]
async fn subscribe_returns_400_for_missing_name_or_email() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = [
        ("name=arun%20manivannan", "missing email"),
        ("email=arun%40arun.com", "missing name"),
        ("", "missing both name and email"),
    ];

    for (each_body, each_err) in body {
        let response = app.post_subscription(each_body.to_string()).await;

        println!("Response status:{}", response.status().as_u16());
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API failed with 400 when the payload was {}",
            each_err
        );
    }
}

#[tokio::test]
async fn subscribe_returns_a_200_when_fields_are_present_but_empty() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=&email=ursula%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        let response = app.post_subscription(body.to_string()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 200 when the payload was {}",
            description
        );
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=arun%20manivannan&email=arun%40arun.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = app.post_subscription(body.to_string()).await;

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    let app = spawn_app().await;
    let body = "name=arun%20manivannan&email=arun%40arun.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscription(body.to_string()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == LinkKind::Url)
            .collect();

        println!("Links :::: {:?}", &links);
        assert_eq!(links.len(), 1);

        links[0].as_str().to_string()
    };

    let html_link = get_link(&body["HtmlBody".to_string()].as_str().unwrap());
    let text_link = get_link(&body["TextBody".to_string()].as_str().unwrap());

    println!("Html Link :::: {}", &html_link);

    assert_eq!(html_link, text_link);
}
