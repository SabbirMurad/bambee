use mongodb::bson::doc;
use crate::util::response::Response;
use crate::model::ImageStruct;
use crate::builtins::{mongo::MongoDB, sqlite};
use actix_web::{web, Error, HttpResponse};

/* Same shape as webp.rs: metadata first, blob read on the blocking pool. */
pub async fn task(image_id: web::Path<String>) -> Result<HttpResponse, Error> {
    let image_id = image_id.trim().to_string();

    let collection = MongoDB.connect().collection::<ImageStruct>("image");
    let meta = match collection.find_one(doc!{"uuid": &image_id}).await {
        Ok(Some(meta)) if !meta.temporary && !meta.deleted => meta,
        Ok(_) => return Ok(Response::not_found("Image not found!")),
        Err(error) => return Ok(Response::internal_server_error(&error.to_string())),
    };

    // The stored bytes may be gif/jpeg/webp as well as png; `original_type` holds
    // the MIME type detected at upload. Announcing image/png for all of them was
    // a lie the browser had to sniff its way past - and with nosniff now set,
    // sniffing is no longer an option.
    match sqlite::image_blob("original", image_id).await {
        Ok(Some(bytes)) => Ok(
            HttpResponse::Ok()
            .content_type(meta.original_type.clone())
            .insert_header(("Cache-Control", "public, max-age=86400"))
            .body(bytes)
        ),
        Ok(None) => Ok(Response::not_found("Image not found!")),
        Err(error) => {
            log::error!("image original: {}", error);
            Ok(Response::internal_server_error(&format!("Storage unavailable: {}", error)))
        }
    }
}
