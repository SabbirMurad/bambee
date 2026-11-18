use mongodb::bson::doc;
use rusqlite::OptionalExtension;
use crate::model::AudioStruct;
use crate::util::response::Response;
use crate::builtins::{mongo::MongoDB, sqlite};
use actix_web::{web, Error, HttpResponse};

/// Streams a stored voice note back to the client, with the mime type it was
/// uploaded with, so `audioplayers` can play it directly from the URL.
///
/// Metadata first, then the blob through `sqlite::run`, so no connection is
/// held across the Mongo await (see `sqlite::run`).
pub async fn task(audio_id: web::Path<String>) -> Result<HttpResponse, Error> {
    let audio_id = audio_id.trim().to_string();

    let collection = MongoDB.connect().collection::<AudioStruct>("audio");
    match collection.find_one(doc!{"uuid": &audio_id}).await {
        Ok(Some(meta)) if !meta.deleted => {}
        Ok(_) => return Ok(Response::not_found("Audio not found!")),
        Err(error) => {
            log::error!("{:?}", error);
            return Ok(Response::internal_server_error(&error.to_string()));
        }
    }

    let result = sqlite::run(sqlite::DBF::IMG, move |conn| {
        Ok(conn
            .query_row(
                "SELECT data, mime FROM audio WHERE uuid = ?1",
                [&audio_id],
                |row| Ok((row.get::<usize, Vec<u8>>(0)?, row.get::<usize, String>(1)?)),
            )
            .optional()?)
    })
    .await;

    match result {
        Ok(Some((audio_data, mime))) => Ok(HttpResponse::Ok().content_type(mime).body(audio_data)),
        Ok(None) => Ok(Response::not_found("Audio not found!")),
        Err(error) => {
            log::error!("audio serve: {}", error);
            Ok(Response::internal_server_error(&error.to_string()))
        }
    }
}
