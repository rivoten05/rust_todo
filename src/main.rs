use actix_web::{
    App, HttpResponse, HttpServer, Responder,
    middleware::Logger,
    web::{self, Json, Path},
};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, SqlitePool, prelude::FromRow};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let pool = db().await;
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%r \x1b[32m%s\x1b[0m %b %D"))
            .app_data(web::Data::new(pool.clone()))
            .route("/todo_list", web::get().to(get_todo_list))
            .route("/todo/{id}", web::get().to(get_single_todo))
            .route("/delete_todo/{id}", web::delete().to(delete_todo))
            .route("/add_todo", web::post().to(add_todo))
            .route("/update_todo/{id}", web::put().to(update_todo))
    })
    .bind("0.0.0.0:3000")
    .unwrap()
    .run()
    .await
}

async fn get_todo_list(pool: web::Data<SqlitePool>) -> impl Responder {
    let todos: Vec<Todo> = sqlx::query_as("SELECT * FROM todos")
        .fetch_all(pool.get_ref())
        .await
        .unwrap();

    let todo_json = serde_json::to_string(&todos).unwrap();

    HttpResponse::Ok().body(todo_json)
}

async fn get_single_todo(id: Path<i32>, pool: web::Data<SqlitePool>) -> impl Responder {
    let id = id.into_inner();
    let row: Vec<Todo> = sqlx::query_as("SELECT * FROM todos WHERE id = ?1")
        .bind(&id)
        .fetch_all(pool.get_ref())
        .await
        .unwrap();

    if row.len() == 0 {
        let msg = format!("Not Todo id: {} found!", id);
        HttpResponse::NotFound().body(msg)
    } else {
        let todo_json = serde_json::to_string(&row[0]).unwrap();
        HttpResponse::Ok().body(todo_json)
    }
}

async fn update_todo(
    id: Path<i32>,
    pool: web::Data<SqlitePool>,
    todo: Json<TodoRequest>,
) -> impl Responder {
    let id = id.into_inner();
    let row: Vec<Todo> = sqlx::query_as("SELECT * FROM todos WHERE id = ?1")
        .bind(&id)
        .fetch_all(pool.get_ref())
        .await
        .unwrap();

    if row.len() == 0 {
        let msg = format!("Not Todo id: {} found!", id);
        HttpResponse::NotFound().body(msg)
    } else {
        sqlx::query("UPDATE todos SET content = ?1 WHERE id = ?2")
            .bind(&todo.content)
            .bind(&id)
            .execute(pool.get_ref())
            .await
            .unwrap();
        HttpResponse::Ok().body("Todo Updated")
    }
}

async fn delete_todo(id: Path<i32>, pool: web::Data<SqlitePool>) -> impl Responder {
    let id = id.into_inner();
    let row: Vec<Todo> = sqlx::query_as("SELECT * FROM todos WHERE id = ?1")
        .bind(&id)
        .fetch_all(pool.get_ref())
        .await
        .unwrap();

    if row.len() == 0 {
        let msg = format!("Not Todo id: {} found!", id);
        HttpResponse::NotFound().body(msg)
    } else {
        sqlx::query("DELETE FROM todos WHERE id = ?1")
            .bind(&id)
            .execute(pool.get_ref())
            .await
            .unwrap();
        HttpResponse::Ok().body("Todo Deleted")
    }
}

async fn add_todo(todo: Json<TodoRequest>, pool: web::Data<SqlitePool>) -> impl Responder {
    sqlx::query("INSERT INTO todos (content) VALUES (?1)")
        .bind(&todo.content)
        .execute(pool.get_ref())
        .await
        .unwrap();
    HttpResponse::Ok().body("Add New Todo Successful")
}

async fn db() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePool::connect("sqlite:db.sqlite?mode=rwc")
        .await
        .unwrap();

    pool.execute(
        "CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT
        )",
    )
    .await
    .expect("Failed to create table due to syntax error");

    pool
}

#[derive(Serialize, FromRow)]
struct Todo {
    id: i32,
    content: String,
}

#[derive(Deserialize)]
struct TodoRequest {
    content: String,
}
