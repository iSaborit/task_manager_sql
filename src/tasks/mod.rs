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
    let rows = query_as!(Tasks, r#"SELECT * FROM tasks"#)
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

    query!(
        r#"
    INSERT INTO tasks (title, description, status, priority, created_at)
    VALUES (?, ?, ?, ?, datetime('now'));
    "#,
        title,
        description,
        status,
        priority
    )
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
    
    query!("DELETE FROM tasks WHERE id = ?", id)
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
