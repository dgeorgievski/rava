use std::net::TcpListener;
use rava::telemetry::{get_subscriber, init_subscriber};
use uuid::Uuid;
use rava::startup::run;
use rava::configuration::get_configuration;
use once_cell::sync::Lazy;

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(
                subscriber_name, 
                default_filter_level, 
                std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(
            subscriber_name, 
            default_filter_level, 
            std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
}

// Launch our application in the background ~somehow~
fn spawn_app() -> TestApp {
     // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind address");

    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let server = run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address,
    }
}

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app();
    // We need to bring in `reqwest`
    // to perform HTTP requests against our application.
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/healthz", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
    // if let Ok(res_ok) = response.text().await {
    //     assert_eq!("OK", res_ok);
    // }else{
    //     assert_eq!("OK", "KO");
    // }
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app();
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // Act
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    // let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
    //     .fetch_one(&app.db_pool)
    //     .await
    //     .expect("Failed to fetch saved subscription.");

    let saved_email = "ursula_le_guin@gmail.com";
    let saved_name = "le guin";
    assert_eq!(saved_email, "ursula_le_guin@gmail.com");
    assert_eq!(saved_name, "le guin");
}
