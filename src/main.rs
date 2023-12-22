use rava::startup::run;
use rava::configuration::get_configuration;
use rava::telemetry::{get_subscriber, init_subscriber};
use rava::kube::watch::watch;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("rava".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");

    // let _ = watch(&configuration).await;

    // Here we choose to bind explicitly to localhost, 127.0.0.1, for security
    // reasons. This binding may cause issues in some environments. For example,
    // it causes connectivity issues running in WSL2, where you cannot reach the
    // server when it is bound to WSL2's localhost interface. As a workaround,
    // you can choose to bind to all interfaces, 0.0.0.0, instead, but be aware
    // of the security implications when you expose the server on all interfaces.
    let address = format!(
        "{}:{}",
        configuration.application.host, 
        configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener)?.await?;

    Ok(())
}
