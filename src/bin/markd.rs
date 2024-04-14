use dotenv::dotenv;
use sqlx::types::chrono::Utc;
use sqlx::PgPool;
use std::env;
use std::fs::File;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    println!("start uploading...");
    let args: Vec<String> = env::args().collect();

    let inserter = match File::open(&args[2]) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content).map_err(|error| {
                eprint!("Error reading file: {:?}", file);
                sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, error))
            })?;
            content
        }
        Err(error) => Err(sqlx::Error::Io(error))?,
    };

    let pool = connect().await.expect("database should connect");
    let now = Utc::now();
    sqlx::query("insert into myposts (post_title, post_body, post_date) values ($1, $2, $3)")
        .bind(&args[1])
        .bind(inserter)
        .bind(&now)
        .execute(&pool)
        .await?;

    println!("completed uploading!");
    Ok(())
}

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

    println!("🤖Connecting to {}🤖", database_url);
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}
