use worker::{
    console_log, wasm_bindgen, wasm_bindgen_futures, worker_sys, Date, Env, Method, Request,
    Response, Result,
};

mod database;
mod utils;
mod vfs;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[worker::event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    log_request(&req);
    utils::set_panic_hook();

    if !matches!(req.method(), Method::Get) {
        return Response::error("Method Not Allowed", 405);
    }

    let namespace = env.durable_object("DATABASE")?;
    let stub = namespace.id_from_name("main")?.get_stub()?;
    stub.fetch_with_str("http://sqlite/").await

    // Response::ok("OK")
    // Response::error("Bad Request", 400)
}
