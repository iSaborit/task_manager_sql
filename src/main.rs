mod tasks;

use std::{env, net::SocketAddr};

use axum::{routing::{get, patch, delete, post}, Router};
use sqlx::sqlite::SqlitePoolOptions;

use tasks::{create_task, delete_task, get_tasks, update_task};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let pool = SqlitePoolOptions::new()
        .connect(&db_url)
        .await 
        .expect("Failed to connect to the DB.");
    
    sqlx::migrate!("./migrations").run(&pool).await?;

    let app = Router::new()
        .route("/task", get(get_tasks).post(create_task))
        .route("/task/{id}", patch(update_task).delete(delete_task))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening in ->> {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}
