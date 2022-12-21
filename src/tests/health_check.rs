use reqwest;
use std::net::TcpListener;
use sqlx::{PgPool, Connection, Executor, PgConnection};
use email_subscription_api::configuration::{get_configuration, DatabaseSettings};
use email_subscription_api::startup::run;
use uuid::Uuid;
 

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool
}

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app(); 
    // performs HTTP request against our Application
    let client = reqwest::client::new();
    // check for Response
    let response = client.get(&format!("{}/health_check", &address)).send().await.expect("Failed to get request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// launches our Application in the Background using Tokio::Spawn
// this abstracts the MAIN-APP from the test functions
async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    // retrieve the assigned port by the OS
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let mut configuration = get_configuration().expect("Failed to read config.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;
    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // create DB
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
            .await.expect("Failed to connect to Postgres");
    connection.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
            .await.expect("Failed to create database.");
    
    // Migrate the DB
    let connection_pool = PgPool::connect(&config.connection_string()).await.expect("Failed to connect to Postgres");
    sqlx::Migrate!("./migrations").run(&connection_pool).await.expect("");
    connection_pool
}

#[tokio::test]
async fn subscribe_returns_200() {
    let app = spawn_app().await;
    let client = reqwest::client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client.post(&format!("{}/subscriptions", &app.address))
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .body(body).send().await.expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
                .fetch_one(&app.db_pool).await.expect("Failed to fetch saved subsccription.");
    
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400() {
    let address = spawn_app();
    let client = reqwest::client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email")
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client.post(&format!("{}/subscriptions", &address))
            .header("Content_Type", "application/x-www-form-urlencoded")
            .body(invalid_body).send().await.expect("Failed to execute request.");
        
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );

    }
}

