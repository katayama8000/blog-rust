use dotenv::dotenv;
use sqlx::PgPool;
use std::env;

#[derive(Debug, Clone)]
struct DbConfig {
    db_user: String,
    db_password: String,
    db_host: String,
    db_name: String,
}

impl DbConfig {
    fn from_env() -> Self {
        dotenv().ok();

        let env = env::var("ENV").expect("ENV must be set in the .env file");

        let (db_user, db_password, db_host, db_name) = match env.as_str() {
            "local" => (
                env::var("DB_USER_DEV").expect("DB_USER_DEV must be set in the .env file"),
                env::var("DB_PASSWORD_DEV").expect("DB_PASSWORD_DEV must be set in the .env file"),
                env::var("DB_HOST_DEV").expect("DB_HOST_DEV must be set in the .env file"),
                env::var("DB_NAME_DEV").expect("DB_NAME_DEV must be set in the .env file"),
            ),
            "production" => (
                env::var("DB_USER").expect("DB_USER must be set in the .env file"),
                env::var("DB_PASSWORD").expect("DB_PASSWORD must be set in the .env file"),
                env::var("DB_HOST").expect("DB_HOST must be set in the .env file"),
                env::var("DB_NAME").expect("DB_NAME must be set in the .env file"),
            ),
            _ => unreachable!(),
        };

        DbConfig {
            db_user,
            db_password,
            db_host,
            db_name,
        }
    }

    fn connection(&self) -> String {
        format!(
            "postgres://{}:{}@{}/{}?sslmode=require",
            self.db_user, self.db_password, self.db_host, self.db_name
        )
    }
}

pub async fn connect() -> Result<PgPool, sqlx::Error> {
    let config = DbConfig::from_env();
    let database_url = config.connection();

    println!("Connecting to {}", database_url);
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}
