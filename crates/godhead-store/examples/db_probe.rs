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
        }
        Err(e) => println!("FAILED: {e}"),
    }
}
