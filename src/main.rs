use std::net::TcpListener;
use email_subscription_api::startup::run;
use email_subscription_api::configuration::get_configuration;
use sqlx::PgPool;



#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
                        .await.expect("Failed to connect to Postgres.");
    // the PORT-NUMBER is now generated automatically.
    let address = format!("127.0.0.1:{}", configuration.application_port);

    let listener= TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}

