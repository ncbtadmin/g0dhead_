//! Connectivity probe: `cargo run -p godhead-store --example db_probe -- <url>`.
//! Prints the server version on success. Diagnostic tool only.

#[tokio::main]
async fn main() {
    let url = std::env::args()
        .nth(1)
        .expect("usage: db_probe <database-url>");
    match sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
    {
        Ok(pool) => {
            let version: String = sqlx::query_scalar("SELECT version()")
                .fetch_one(&pool)
                .await
                .expect("query");
            println!("CONNECTED: {version}");
            let pgvector: Option<String> = sqlx::query_scalar(
                "SELECT default_version FROM pg_available_extensions WHERE name = 'vector'",
            )
            .fetch_optional(&pool)
            .await
            .expect("extension query");
            match pgvector {
                Some(v) => println!("PGVECTOR: available, version {v}"),
                None => println!("PGVECTOR: NOT available on this server"),
            }
        }
        Err(e) => println!("FAILED: {e}"),
    }
}
