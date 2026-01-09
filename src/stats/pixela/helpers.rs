use std::process::exit;

use crate::error::{PixelaResponseError, Result, StatsError};

use super::{
    complex_pixel::ComplexPixel,
    subjects::{SubjectDataType, SubjectUnit},
    utils::{PixelaResponse, PixelaValue},
};

pub async fn get_from_pixela(
    pixel: &ComplexPixel,
    client: &reqwest::Client,
    api_key: &str,
) -> Result<usize> {
    let request = client
        .get(format!(
            "{}/{}",
            pixel.subject().url(),
            pixel.date_pixela_formatted()
        ))
        .header("X-USER-TOKEN", api_key);
    let request = request.send().await?;
    let status = request.status();
    match status.is_success() {
        true => {
            let response = request.json::<PixelaValue>().await?;
            Ok(response.quantity().parse().map_err(|_| {
                PixelaResponseError::FatalError(
                    "Failed parsing".into(),
                    reqwest::StatusCode::BAD_REQUEST,
                )
            })?)
        }
        false => {
            if status == reqwest::StatusCode::SERVICE_UNAVAILABLE
                || status == reqwest::StatusCode::INTERNAL_SERVER_ERROR
            {
                Err(PixelaResponseError::RetryableError("Server is busy".into(), status).into())
            } else if status == reqwest::StatusCode::NOT_FOUND {
                return Ok(0);
            } else {
                Err(PixelaResponseError::FatalError(
                    format!("Request denied: {}", request.text().await.unwrap()),
                    status,
                )
                .into())
            }
        }
    }
}

pub async fn send_to_pixela(
    pixel: &ComplexPixel,
    client: &reqwest::Client,
    api_key: &str,
) -> Result<()> {
    let subject_url = pixel.subject().url();
    let date = pixel.date_pixela_formatted();
    let url = format!("{subject_url}/{date}/add");
    let request = client
        .put(url)
        .json(&serde_json::json!({
        "quantity":pixel.progress().to_string()
            }))
        .header("X-USER-TOKEN", api_key)
        .send()
        .await?;
    let status = request.status();
    match status.is_success() {
        true => {
            request.json::<PixelaResponse>().await?;
            Ok(())
        }
        false => {
            if status == reqwest::StatusCode::SERVICE_UNAVAILABLE
                || status == reqwest::StatusCode::INTERNAL_SERVER_ERROR
            {
                Err(PixelaResponseError::RetryableError("Server is busy".into(), status).into())
            } else {
                Err(PixelaResponseError::FatalError(
                    format!("Request denied: {}", request.text().await.unwrap()),
                    status,
                )
                .into())
            }
        }
    }
}
pub fn check_if_quantity_is_big_enough(pixel: &ComplexPixel) -> Result<()> {
    if *pixel.subject().unit() == SubjectUnit::Hours
        && *pixel.subject().data_type() == SubjectDataType::Int
        && pixel.progress().get_as_int() < 1
    {
        return Err(StatsError::QuantityIsNotBigEnough.into());
    }
    Ok(())
}
