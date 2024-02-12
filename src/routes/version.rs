use actix_web::{get, web, Responder, Result};
use crate::version::get_version;


#[get("/version")]
pub async fn healthcat_version() -> Result<impl Responder> {
    Ok(web::Json(get_version()))
}