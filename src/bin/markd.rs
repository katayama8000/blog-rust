use dotenv::dotenv;
use sqlx::PgPool;
use std::env;
use std::fs::File;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let args: Vec<String> = env::args().collect();
    println!("{}", args[1]);
    println!("{}", args[2]);

    let inserter;

    match File::open(&args[2]) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            inserter = content;
        }
        Err(error) => {
            panic!("{}", format!("Problem opening the file: {:?}", error));
        }
    }

    let pool = connect().await.expect("database should connect");
    let post_id = 2;
    let _row: (i64,) = sqlx::query_as(
    "insert into myposts (post_id, post_title, post_body) values ($1, $2, $3) returning post_id",
)
.bind(post_id)
.bind(&args[1])
.bind(inserter)
.fetch_one(&pool)
.await?;

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

    println!("Connecting to {}", database_url);
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}
