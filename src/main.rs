use std::net::TcpListener;
use email_subscription_api::startup::run;
use email_subscription_api::configuration::get_configuration;
use sqlx::postgres::PgPoolOptions;
use email_subscription_api::telementry::{get_subscriber, init_subscriber};


#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("email_sub_api".into(), "into".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");

    let connection_pool = PgPoolOptions::new().connect_timeout(std::time::Duration::from_secs(2))
                        .connect_lazy_with(configuration.database.with_db());
    
    // the PORT-NUMBER is now generated automatically.
    let address = format!("{}:{}", configuration.application.host, configuration.application.port);

    let listener= TcpListener::bind(address)?;
    run(listener, connection_pool)?.await?;
    Ok(())
}


