use rava::startup::run;
use rava::configuration::get_configuration;
use rava::telemetry::{get_subscriber, init_subscriber};
use rava::kube::watch::watch;
use rava::output;
use std::net::TcpListener;
use tokio::sync::mpsc::{Sender, Receiver};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("rava".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");

    match watch(&configuration).await {
        Ok(rx) => {
            let (tx, rx): (Sender<WatchEvent>, Receiver<WatchEvent>) = channel(32);
            let _ = output::simple_print_process(rx).await?;
        },
        Err(error) => {
            tracing::error!("Failed to watch configured resources {:?}", error)
        }
    };

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
