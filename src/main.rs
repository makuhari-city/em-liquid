use actix_cors::Cors;
use actix_web::{get, middleware, post, web, App, HttpServer, Responder};
use em_liquid::calculate;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;
use vote::rpc::{JsonRPCRequest, JsonRPCResponse};

type ModuleMap = Mutex<HashMap<String, String>>;

const MODULE_NAME: &str = "/liquid";
const PORT: &str = "8102";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=trace,actix_redis=trace,vote=debug");
    env_logger::init();

    let modules: web::Data<ModuleMap> = web::Data::new(Mutex::new(HashMap::new()));

    let addr = format!("0.0.0.0:{}", PORT);

    HttpServer::new(move || {
        // TODO: change this
        let cors = Cors::permissive();

        App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .app_data(modules.clone())
            .service(web::scope(MODULE_NAME).service(hello).service(rpc))
    })
    .bind(&addr)?
    .run()
    .await
}

#[get("/hello/")]
async fn hello() -> impl Responder {
    "hello"
}

#[post("/rpc/")]
async fn rpc(rpc: web::Json<JsonRPCRequest>) -> impl Responder {
    let rpc = rpc.into_inner();
    let mut response = JsonRPCResponse::new(&rpc.id());
    let result = calculate(rpc.vote_info()).await;
    response.result(&json!(result));
    web::Json(response)
}
