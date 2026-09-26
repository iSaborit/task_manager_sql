use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use sqlx::{query, query_as, sqlite::SqlitePool};
use task_manager_sql::{CreateTaskReq, Tasks, UpdateTaskReq};

pub async fn get_tasks(
    State(pool): State<sqlx::sqlite::SqlitePool>,
) -> Result<impl IntoResponse, Response> {
    let rows = query_as::<_, Tasks>(r#"SELECT * FROM tasks"#)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": e.to_string()})),
            )
                .into_response()
        })?;

    Ok((
        StatusCode::OK,
        axum::Json(json!({
            "success": true,
            "data": rows
        })),
    ))
}

pub async fn create_task(
    State(pool): State<sqlx::sqlite::SqlitePool>,
    Json(request): Json<CreateTaskReq>,
) -> Result<impl IntoResponse, Response> {
    let title = request.title;
    let description = request.description.unwrap_or("".to_string());
    let status = match request.status {
        Some(value) => {
            if value == "in_progress" {
                "in_progress".to_owned()
            } else if value == "completed" {
                "completed".to_owned()
            } else {
                "pending".to_owned()
            }
        }
        _ => "pending".to_owned(),
    };
    let priority = request.priority.unwrap_or(1);

    query(
        r#"
    INSERT INTO tasks (title, description, status, priority, created_at)
    VALUES (?, ?, ?, ?, datetime('now'));
    "#,
    )
    .bind(title)
    .bind(description)
    .bind(status)
    .bind(priority)
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": e.to_string()
            })),
        )
            .into_response()
    })?;

    Ok((StatusCode::OK, Json(json!({"success": true}))))
}

pub async fn update_task(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(update_req): Json<UpdateTaskReq>,
) -> Result<impl IntoResponse, Response> {
    let mut query = String::from("UPDATE tasks SET");
    let mut updates = vec![];

    if update_req.title.is_some() {
        updates.push(" title = ?");
    }
    if update_req.description.is_some() {
        updates.push(" description = ?");
    }
    if update_req.status.is_some() {
        updates.push(" status = ?");
    }
    if update_req.priority.is_some() {
        updates.push(" priority = ?");
    }

    query.push_str(&updates.join(","));
    query.push_str(" WHERE id = ?");

    if updates.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"success": false, "message": "No fields to update"})),
        ));
    }

    let mut s = sqlx::query(&query);
    if let Some(title) = update_req.title {
        s = s.bind(title);
    }
    if let Some(description) = update_req.description {
        s = s.bind(description);
    }
    if let Some(status) = update_req.status {
        s = s.bind(status);
    }
    if let Some(priority) = update_req.priority {
        s = s.bind(priority);
    }
    s.bind(id).execute(&pool).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"success": true, "error": e.to_string()})),
        )
            .into_response()
    })?;

    Ok((StatusCode::OK, Json(json!({"success": true}))))
}

pub async fn delete_task(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, Response> {
    query("DELETE FROM tasks WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": e.to_string()
                })),
            )
                .into_response()
        })?;

    Ok((StatusCode::OK, Json(json!({"success": true}))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::{extract::Path, extract::State, response::IntoResponse, Json};
    use serde_json::Value;
    use sqlx::{query_scalar, sqlite::SqlitePoolOptions, Executor};

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("failed to create in-memory sqlite pool");

        pool.execute(
            r#"
            CREATE TABLE tasks (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                status TEXT CHECK(status IN('pending', 'in_progress', 'completed')) DEFAULT 'pending',
                priority INTEGER DEFAULT 1 NOT NULL,
                created_at DATETIME NOT NULL
            );
            "#,
        )
        .await
        .expect("failed to create tasks table");

        pool
    }

    #[tokio::test]
    async fn create_and_get_tasks_returns_created_task() {
        let pool = setup_pool().await;

        let request = CreateTaskReq {
            title: "Aprender Rust".to_string(),
            description: Some("CRUD con Axum".to_string()),
            status: Some("pending".to_string()),
            priority: Some(2),
        };

        let create_response = create_task(State(pool.clone()), Json(request)).await;
        assert!(create_response.is_ok());

        let response = get_tasks(State(pool)).await.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload["success"], Value::Bool(true));
        assert_eq!(payload["data"].as_array().unwrap().len(), 1);
        assert_eq!(
            payload["data"][0]["title"],
            Value::String("Aprender Rust".to_string())
        );
    }

    #[tokio::test]
    async fn update_task_changes_fields() {
        let pool = setup_pool().await;

        query(
            "INSERT INTO tasks (title, description, status, priority, created_at) VALUES (?, ?, ?, ?, datetime('now'))",
        )
        .bind("Inicial")
        .bind("Desc")
        .bind("pending")
        .bind(1_i64)
        .execute(&pool)
        .await
        .unwrap();

        let request = UpdateTaskReq {
            title: Some("Actualizada".to_string()),
            description: None,
            status: Some("completed".to_string()),
            priority: Some(3),
        };

        let response = update_task(State(pool.clone()), Path(1), Json(request)).await;
        assert!(response.is_ok());

        let row = query_as::<_, Tasks>("SELECT * FROM tasks WHERE id = ?")
            .bind(1_i64)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(row.title, "Actualizada");
        assert_eq!(row.status.as_deref(), Some("completed"));
        assert_eq!(row.priority, 3);
    }

    #[tokio::test]
    async fn update_task_without_fields_returns_bad_request() {
        let pool = setup_pool().await;
        let request = UpdateTaskReq {
            title: None,
            description: None,
            status: None,
            priority: None,
        };

        let response = update_task(State(pool), Path(1), Json(request))
            .await
            .unwrap()
            .into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn delete_task_removes_row() {
        let pool = setup_pool().await;

        query(
            "INSERT INTO tasks (title, description, status, priority, created_at) VALUES (?, ?, ?, ?, datetime('now'))",
        )
        .bind("Eliminar")
        .bind("")
        .bind("pending")
        .bind(1_i64)
        .execute(&pool)
        .await
        .unwrap();

        let response = delete_task(State(pool.clone()), Path(1)).await;
        assert!(response.is_ok());

        let count: i64 = query_scalar("SELECT COUNT(*) FROM tasks")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
}
