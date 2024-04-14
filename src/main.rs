use crate::config::connect;
use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, Router},
};
use sqlx::types::chrono::{self, FixedOffset, Local, TimeZone};
use sqlx::FromRow;
use std::sync::Arc;
use tower_http::services::ServeDir;
pub mod config;

// post template
// localhost:3000/post/:query_title
#[derive(Template)]
#[template(path = "posts.html")]
struct PostTemplate<'a> {
    post_title: &'a str,
    post_date: String,
    post_body: &'a str,
}

// homepage template
// localhost:3000/
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub id: &'a Vec<String>,
    pub index_title: String,
    pub index_links: &'a Vec<String>,
}

// SQL query will return all posts
// into a Vec<Post>
#[derive(FromRow, Debug, Clone)]
pub struct Post {
    pub post_id: i32,
    pub post_title: String,
    pub post_date: chrono::NaiveDateTime,
    pub post_body: String,
}

// Our custom Askama filter to replace spaces with dashes in the title
mod filters {

    // now in our templates with can add tis filter e.g. {{ post_title|rmdash }}
    pub fn rmdashes(title: &str) -> askama::Result<String> {
        Ok(title.replace("-", " ").into())
    }
}

// post router uses two extractors
// Path to extract the query: localhost:3000/post/thispart
// State that holds a Vec<Post> used to render the post that the query matches
async fn post(Path(id): Path<String>, State(state): State<Arc<Vec<Post>>>) -> impl IntoResponse {
    let id = match id.parse::<usize>() {
        Ok(parsed_id) if parsed_id > 0 && parsed_id <= state.len() => parsed_id - 1,
        _ => return (StatusCode::NOT_FOUND, "404 not found").into_response(),
    };

    let post = &state[id];
    let local = Local.from_local_datetime(&post.post_date).unwrap();
    let datetime = FixedOffset::east_opt(9 * 3600).expect("offset to be valid");
    let post_date_jst = local.with_timezone(&datetime);

    let template = PostTemplate {
        post_title: &post.post_title,
        post_date: post_date_jst.format("%Y-%m-%d %H:%M:%S").to_string(),
        post_body: &post.post_body,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "try again later").into_response(),
    }
}

// index router (homepage) will return all blog titles in anchor links
async fn index(State(state): State<Arc<Vec<Post>>>) -> impl IntoResponse {
    let mut plinks: Vec<String> = Vec::new();
    let mut ids: Vec<String> = Vec::new();

    for post in state.iter() {
        plinks.push(post.post_title.clone());
        ids.push(post.post_id.to_string());
    }

    let template = IndexTemplate {
        id: &ids,
        index_title: String::from("My blog 🐈‍⬛"),
        index_links: &plinks,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to render template. Error {}", err),
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() {
    let pool = connect().await.expect("database should connect");
    let mut posts = sqlx::query_as::<_, Post>("select * from myposts")
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            eprintln!("Error: {:?}", e);
            e
        })
        .expect("posts should be fetched");

    for post in &mut posts {
        post.post_title = post.post_title.replace(" ", "-");
    }

    let shared_state = Arc::new(posts);

    let app = Router::new()
        .route("/", get(index))
        .route("/post/:id", get(post))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Listening on: http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
