use std::net::TcpListener;
use email_subscription_api::startup::run;
use email_subscription_api::configuration::get_configuration;
use sqlx::postgres::PgPool;
use email_subscription_api::telementry::{get_subscriber, init_subscriber};
use secrecy::ExposeSecret;


#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("email_sub_api".into(), "into".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string().expose_secret())
                        .await.expect("Failed to connect to Postgres.");
    // the PORT-NUMBER is now generated automatically.
    let address = format!("127.0.0.1:{}", configuration.application_port);

    let listener= TcpListener::bind(address)?;
    run(listener, connection_pool)?.await?;
    Ok(())
}


