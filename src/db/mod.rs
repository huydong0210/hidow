pub mod schema;
pub mod loader;
pub mod queries;

use anyhow::Result;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

/// Connect to SurrealDB, authenticate, and select namespace/database.
pub async fn connect(url: &str, ns: &str, db_name: &str) -> Result<Surreal<Client>> {
    let db = Surreal::new::<Ws>(url).await?;

    // Sign in as root user
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;

    db.use_ns(ns).use_db(db_name).await?;
    Ok(db)
}
