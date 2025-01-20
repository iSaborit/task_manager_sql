use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Serialize, Deserialize)]
pub struct Tasks {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: i64,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct CreateTaskReq {
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateTaskReq {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<i64>,
}
