use axum::Router;
use axum::routing::get;
use anyhow::Result;
use axum::extract::Path;
use axum::response::IntoResponse;

///init shorten router
fn init_shorten_router()-> anyhow::Result<Router> {

    let shorten_router= Router::new()
        .route("/:id", get(|| async { "Hello, World! id" }))
        .route("/create",get(|| async { "Hello, World! create" }))
        ;

    Ok(shorten_router)
}


/// create shorten link by
async fn create_shorten_link(Path(id): Path<String>) -> impl IntoResponse{

    todo!()


}