use actix_web::*;
use engine;
use futures::Future;


fn index(req: HttpRequest) -> String {
    String::from(engine::hello())
}

#[derive(Deserialize, Serialize, Debug)]
enum Status {
    SUCCESS,
    FAILURE,
    SKIPPED
}

#[derive(Deserialize, Serialize, Debug)]
struct TestResult {
    test_name: String,
    result: Status,
}

fn ingest(req: HttpRequest) -> Box<Future<Item=HttpResponse, Error=Error>> {
    info!("request: {:?}", req);
    
    req.json()
        .from_err()
        .map_err(|err| {
            error!("error: {:?}", err);
            err
        })
        .and_then(|val: Vec<TestResult>| {
            info!("model: {:?}", val);
            Ok(httpcodes::HTTPOk.build().json(val)?)
        })
        .responder()
}

pub fn serve(port: u16) {
    HttpServer::new(
        || Application::new()
            .middleware(middleware::Logger::default())
            .resource("/", |r| r.method(Method::GET).f(index))
            .resource("/ingest", |r| r.method(Method::POST).f(ingest)))
        .bind(format!("127.0.0.1:{}", port)).unwrap()
        .run();
}
