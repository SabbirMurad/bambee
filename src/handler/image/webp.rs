use mongodb::bson::doc;
use crate::util::response::Response;
use crate::model::ImageStruct;
use crate::builtins::{mongo::MongoDB, sqlite};
use actix_web::{web, Error, HttpResponse};

/*
  Metadata first, then the bytes. Checking that the image is live before
  reading the blob means a temporary or deleted image costs no SQLite read,
  and the read itself goes through `sqlite::image_blob`, which runs on the
  blocking pool — so no connection is held while Mongo is awaited.

  An id never gets new bytes (a changed picture is uploaded under a new id),
  so the browser may keep it for a day like the rest of the site's media.
*/
pub async fn task(image_id: web::Path<String>) -> Result<HttpResponse, Error> {
    let image_id = image_id.trim().to_string();

    let collection = MongoDB.connect().collection::<ImageStruct>("image");
    match collection.find_one(doc!{"uuid": &image_id}).await {
        Ok(Some(meta)) if !meta.temporary && !meta.deleted => {}
        Ok(_) => return Ok(Response::not_found("Image not found!")),
        Err(error) => return Ok(Response::internal_server_error(&error.to_string())),
    }

    match sqlite::image_blob("webp", image_id).await {
        Ok(Some(bytes)) => Ok(
            HttpResponse::Ok()
            .content_type("image/webp")
            .insert_header(("Cache-Control", "public, max-age=86400"))
            .body(bytes)
        ),
        Ok(None) => Ok(Response::not_found("Image not found!")),
        Err(error) => {
            log::error!("image webp: {}", error);
            Ok(Response::internal_server_error(&format!("Storage unavailable: {}", error)))
        }
    }
}
