use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
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
// 動作検証用のハンドラー
pub async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

pub async fn url_handler(State(state): State<Arc<AppState>>) -> Html<String> {
    let html_string: String = format!(
        "<h1>Hello, World!</h1>\
        <p>DatabaseUrl : {:?}</p>\
    ",
        &state.as_ref().database_url
    );
    Html(html_string)
}

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Rust CRUD API Example with Axum Framework and MySQL";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

// 1. すべてのレコードを取得するハンドラー
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

    let notes = sqlx::query("SELECT * FROM notes ORDER by id LIMIT ? OFFSET ?")
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
// 2. レコードを挿入するハンドラー
pub async fn create_note_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateNoteSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = uuid::Uuid::new_v4().to_string();
    let query_result =
        sqlx::query(r#"INSERT INTO notes (id,title,content,category) VALUES (?, ?, ?, ?)"#)
            .bind(user_id.clone())
            .bind(body.title.to_string())
            .bind(body.content.to_string())
            .bind(body.category.to_owned().unwrap_or_default())
            .execute(&data.db)
            .await
            .map_err(|err: sqlx::Error| err.to_string());

    if let Err(err) = query_result {
        if err.contains("Duplicate entry") {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Note with that title already exists",
            });
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }

        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error","message": format!("{:?}", err)})),
        ));
    }

    // let note = sqlx::query_as!(NoteModel, r#"SELECT * FROM notes WHERE id = ?"#, user_id)
    //     .fetch_one(&data.db)
    //     .await
    //     .map_err(|e| {
    //         (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             Json(json!({"status": "error","message": format!("{:?}", e)})),
    //         )
    //     })?;

    let query = format!("SELECT * FROM notes WHERE id = '{}'", user_id);
    let note = sqlx::query_as::<_, NoteModel>(&query)
        .fetch_one(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            )
        })?;

    let note_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "note": filter_db_record(&note)
    })});

    Ok(Json(note_response))
}

// 3. レコードを取得するハンドラー
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

// 4. レコードを編集するためのハンドラー

pub async fn edit_note_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateNoteSchema>,
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

    let note = match query_result {
        Ok(note) => note,
        Err(sqlx::Error::RowNotFound) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Note with ID: {} not found", id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
        }
    };

    let published = body.published.unwrap_or(note.published != 0);
    let i8_published = published as i8;

    let update_result = sqlx::query(
        r#"UPDATE notes SET title = ?, content = ?, category = ?, published = ? WHERE id = ?"#,
    )
    .bind(body.title.to_owned().unwrap_or_else(|| note.title.clone()))
    .bind(
        body.content
            .to_owned()
            .unwrap_or_else(|| note.content.clone()),
    )
    .bind(
        body.category
            .to_owned()
            .unwrap_or_else(|| note.category.clone().unwrap()),
    )
    .bind(i8_published)
    .bind(id.to_string())
    .execute(&data.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error","message": format!("{:?}", e)})),
        )
    })?;

    if update_result.rows_affected() == 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Note with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    // 挿入後、データベースに問い合わせを行う。挿入IDの結果を返す

    // let updated_note = sqlx::query_as!(
    //     NoteModel,
    //     r#"SELECT * FROM notes WHERE id = ?"#,
    //     id.to_string()
    // )
    // .fetch_one(&data.db)
    // .await
    // .map_err(|e| {
    //     (
    //         StatusCode::INTERNAL_SERVER_ERROR,
    //         Json(json!({"status": "error","message": format!("{:?}", e)})),
    //     )
    // })?;

    let query = format!("SELECT * FROM notes WHERE id = '{}'", id);
    let updated_note = sqlx::query_as::<_, NoteModel>(&query)
        .fetch_one(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            )
        })?;

    let note_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "note": filter_db_record(&updated_note)
    })});

    Ok(Json(note_response))
}

// 5. レコードを削除するハンドラー
pub async fn delete_note_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // let query_result = sqlx::query!(r#"DELETE FROM notes WHERE id = ?"#, id.to_string())
    //     .execute(&data.db)
    //     .await
    //     .map_err(|e| {
    //         (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             Json(json!({"status": "error","message": format!("{:?}", e)})),
    //         )
    //     })?;
    let query = format!("DELETE FROM notes WHERE id = '{}'", id);
    let query_result = sqlx::query(&query).execute(&data.db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error","message": format!("{:?}", e)})),
        )
    })?;

    if query_result.rows_affected() == 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Note with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    // 削除できたら202を返す
    Ok(StatusCode::NO_CONTENT)
    // Ok(StatusCode::OK)
}
