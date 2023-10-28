use actix_multipart::Multipart;
use actix_web::{
    error::InternalError, http::header::ContentDisposition, http::StatusCode, post, Error,
    HttpResponse,
};
use csv::Reader as CsvReader;
use futures::{StreamExt, TryStreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Cursor;
use uuid::Uuid;

#[post("/upload")]
pub async fn upload(mut payload: Multipart) -> Result<HttpResponse, Error> {
    while let Some(mut field) = payload.try_next().await? {
        let content_disposition: &ContentDisposition = field.content_disposition();
        let filename = content_disposition
            .get_filename()
            .map_or_else(|| Uuid::new_v4().to_string(), sanitize_filename::sanitize);

        if !filename.ends_with(".csv") {
            return Ok(HttpResponse::BadRequest().body("Only CSV files are allowed."));
        }

        let mut content: Vec<u8> = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            content.extend_from_slice(&data);
        }

        // Parse the CSV content
        let cursor = Cursor::new(content);
        let mut rdr = CsvReader::from_reader(cursor);
        let mut records = Vec::new();

        let headers = rdr
            .headers()
            .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?
            .clone();

        for result in rdr.records() {
            let record =
                result.map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;

            let mut map = HashMap::new();
            for (header, value) in headers.iter().zip(record.iter()) {
                map.insert(header.to_string(), Value::String(value.to_string()));
            }

            let json_value: Value = serde_json::to_value(map).unwrap();
            if let Value::Object(converted_map) = json_value {
                records.push(Value::Object(converted_map));
            }
        }

        return Ok(HttpResponse::Ok().json(json!({
            "data": records
        })));
    }

    Ok(HttpResponse::BadRequest().body("No valid CSV provided"))
}
