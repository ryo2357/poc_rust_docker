use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::Row;

use crate::{
    model::{NoteModel, NoteModelResponse},
    schema::{CreateNoteSchema, FilterOptions, UpdateNoteSchema},
    AppState,
};

fn filter_db_record(note: &NoteModel) -> NoteModelResponse {
    NoteModelResponse {
        id: note.id.to_owned(),
        title: note.title.to_owned(),
        content: note.content.to_owned(),
        category: note.category.to_owned().unwrap(),
        published: note.published != 0,
        createdAt: note.created_at.unwrap(),
        updatedAt: note.updated_at.unwrap(),
    }
}

pub async fn note_list_handler(
    opts: Option<Query<FilterOptions>>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    // let notes = sqlx::query_as!(
    //     NoteModel,
    //     r#"SELECT * FROM notes ORDER by id LIMIT ? OFFSET ?"#,
    //     limit as i32,
    //     offset as i32
    // )
    // .fetch_all(&data.db)
    // .await
    // .map_err(|e| {
    //     let error_response = serde_json::json!({
    //         "status": "fail",
    //         "message": format!("Database error: {}", e),
    //     });
    //     (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    // })?;

    let notes = sqlx::query("SELECT * FROM notes ORDER by id LIMIT $1 OFFSET $2")
        .bind(limit as i32)
        .bind(offset as i32)
        .try_map(|row: sqlx::mysql::MySqlRow| {
            Ok(NoteModel {
                id: row.try_get("id")?,
                title: row.try_get("title")?,
                content: row.try_get("content")?,
                category: row.try_get("category")?,
                published: row.try_get("published")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })
        })
        .fetch_all(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let note_responses = notes
        .iter()
        .map(|note| filter_db_record(&note))
        .collect::<Vec<NoteModelResponse>>();

    let json_response = serde_json::json!({
        "status": "success",
        "results": note_responses.len(),
        "notes": note_responses
    });

    Ok(Json(json_response))
}

pub async fn get_note_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // let query_result = sqlx::query_as!(
    //     NoteModel,
    //     r#"SELECT * FROM notes WHERE id = ?"#,
    //     id.to_string()
    // )
    // .fetch_one(&data.db)
    // .await;

    let query = format!("SELECT * FROM notes WHERE id = '{}'", id);
    let query_result = sqlx::query_as::<_, NoteModel>(&query)
        .fetch_one(&data.db)
        .await;

    match query_result {
        Ok(note) => {
            let note_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "note": filter_db_record(&note)
            })});

            Ok(Json(note_response))
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Note with ID: {} not found", id)
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error","message": format!("{:?}", e)})),
        )),
    }
}
