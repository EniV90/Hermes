use actix_web::{App, http::StatusCode, test, web};
use hermes::{
    configuration::{DatabaseSettings, get_configuration},
    routes::{health_check, subscribe},
};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn spawn() -> TestApp {
        let mut configuration = get_configuration().expect("Failed to get configuration");
        configuration.database.database_name = Uuid::new_v4().to_string();
        let db_pool = configure_database(&configuration.database).await;

        TestApp {
            address: format!("http://127.0.0.1:8080"),
            db_pool,
        }
    }
}
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    //create database
    let mut connection = PgConnection::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    //Migrate database
    let db_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to conect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate databsae");

    db_pool
}

#[actix_web::test]
async fn health_check_works() {
    let app =
        test::init_service(App::new().route("/health_check", web::get().to(health_check))).await;

    let req = test::TestRequest::get().uri("/health_check").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let test_app = TestApp::spawn().await;

    let app = test::init_service(
        App::new()
            .route("/subscribe", web::post().to(subscribe))
            .app_data(web::Data::new(test_app.db_pool.clone())),
    )
    .await;

    let body = "name=victor&email=eni_v%40gmail.com";
    let req = test::TestRequest::post()
        .uri("/subscribe")
        .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
        .set_payload(body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Now check if the data was saved
    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "eni_v@gmail.com");
    assert_eq!(saved.name, "victor");
}

#[actix_web::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let test_app = TestApp::spawn().await;

    let app = test::init_service(
        App::new()
            .route("/subscribe", web::post().to(subscribe))
            .app_data(web::Data::new(test_app.db_pool.clone())),
    )
    .await;

    let test_cases = vec![
        ("name=victor%20eni", "missing email"),
        ("email=eni_v%40gmail.com", "missing name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let req = test::TestRequest::post()
            .uri("/subscribe")
            .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
            .set_payload(invalid_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "The API did not fail when the payload was {}",
            error_message
        );
    }
}
