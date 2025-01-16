use anyhow::{Context, Error};
use axum::{extract::State, http::StatusCode, routing::get, Router};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;

use clap::{Parser, Subcommand};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    RunMigrations,
}

#[derive(Debug, sqlx::FromRow)]
struct DBBots {
    #[allow(dead_code)]
    id: i32,
    name: String,
    thoughts: String,
}

async fn run_database_migration() -> Result<(), Error> {
    let url: String = "postgres://postgres:postgres@localhost:5432/dbmigration".to_string();
    println!("Running database migration on {url}...");
    let db_pool = PgPool::connect(url.as_str())
        .await
        .context("Failed to connect to database")?;

    sqlx::migrate!("./migrations").run(&db_pool).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::RunMigrations) => run_database_migration().await,
        None => run_server().await,
    }
}

async fn run_server() -> Result<(), Error> {
    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/dbmigration".to_string());
    let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "DEV".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap();

    // set up connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can't connect to database");

    // build our application with some routes
    let app = Router::new().route("/", get(show_stuff)).with_state(pool);

    let local_ip = match env.as_str() {
        "PROD" => Ipv4Addr::new(0, 0, 0, 0),
        _ => Ipv4Addr::new(127, 0, 0, 1),
    };

    let address = SocketAddr::new(IpAddr::from(local_ip), port);
    let listener = TcpListener::bind(address).await.unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn show_stuff(State(pool): State<PgPool>) -> Result<String, (StatusCode, String)> {
    let query = r#"select * from dbmigrationtest"#;
    let all_bots: Vec<DBBots> = sqlx::query_as(query).fetch_all(&pool).await.unwrap();
    let res = all_bots
        .iter()
        .map(|item| format!("{}:{}", item.name, item.thoughts))
        .collect::<Vec<String>>()
        .join("\n");
    Ok(res)
}
