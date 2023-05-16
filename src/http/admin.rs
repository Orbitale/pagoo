use actix_web::get;
use actix_web::HttpResponse;
use actix_web::Responder;

pub(crate) fn frontend_assets(path: String) -> Option<HttpResponse> {
    let path = &mut path.clone();

    let last_char = path.chars().last().unwrap().to_string();
    if last_char == "/" {
        path.push_str("index.html");
    }
    path.remove(0);

    let assets = crate::generate();
    let asset = assets.get(path.as_str());
    if asset.is_none() {
        return None;
    }
    let asset = asset.unwrap();

    Some(
        HttpResponse::Ok()
            .insert_header(("Content-Type", asset.mime_type))
            .body(asset.data),
    )
}

#[get("/api")]
pub(crate) async fn api_root() -> impl Responder {
    "Api endpoint.".to_string()
}
