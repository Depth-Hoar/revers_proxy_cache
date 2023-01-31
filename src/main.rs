use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use awc::http::StatusCode;
use awc::Client;
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod cache;
use cache::{Cache, CacheRequest};

// GET request handler
async fn handle(
    req: HttpRequest,
    data: web::Data<Arc<Mutex<Cache<CacheRequest, web::Bytes>>>>,
    origin: web::Data<String>,
) -> impl Responder {
    let req_info = CacheRequest::new(req.path().to_string(), req.query_string().to_string());
    let path = format!("https://{}{}", origin.get_ref(), req_info.path);
    let path = if req_info.query_string != "" {
        format!("{}?{}", path, req_info.query_string)
    } else {
        path
    };

    println!("Received GET request [{}]", path);
    if let Some(resp) = data.lock().unwrap().get(&req_info) {
        println!("Serving from cache.");
        return HttpResponse::Ok().body(resp.clone());
    }

    let client = Client::default();
    let mut res = client.get(path).send().await.unwrap();
    let response = res.body().await.unwrap();
    println!(
        "Response from origin: {}",
        String::from_utf8_lossy(&response)
    );

    match res.status() {
        StatusCode::OK => {
            let body = res.body().await.unwrap();
            println!("Caching response.");
            data.lock().unwrap().insert(req_info, body.clone());
            HttpResponse::Ok().body(body)
        }
        code => {
            println!("Error {}, not caching.", code);
            let body = res.body().await.unwrap();
            HttpResponse::build(code).body(body)
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let origin_host = "blockstream.info".to_string();
    let listening_port = 3000;

    println!("Starting reverse proxy server");
    println!("Forwarding requests to: {}", origin_host);
    println!("Listening on port: {}", listening_port);
    println!();

    let cache: web::Data<Arc<Mutex<Cache<CacheRequest, web::Bytes>>>> =
        web::Data::new(Arc::new(Mutex::new(Cache::new(Duration::new(30, 0)))));
    let origin = web::Data::new(origin_host);

    HttpServer::new(move || {
        App::new()
            .app_data(cache.clone())
            .app_data(origin.clone())
            .route("/{path:.*}", web::get().to(handle))
    })
    .bind(("localhost", listening_port as u16))?
    .run()
    .await
}

// TODO
async fn start_cache_cleanup_task(cache: web::Data<Arc<Mutex<Cache<CacheRequest, web::Bytes>>>>) {
    let interval = Duration::new(30, 0);
    loop {
        // specify interval TODO
        let mut cache = cache.lock().unwrap();
        cache.remove_expired();
    }
}
