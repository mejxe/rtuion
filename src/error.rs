use std::io;

use crate::popup::Popup;

#[derive(thiserror::Error, Debug)]
pub enum SettingsError {
    #[error("Update failed! Stop the timer first")]
    UpdateError(),

    #[error("There was an error with saving your data: {0}")]
    SaveError(String),

    #[error("There was an error with loading your data: {0}")]
    LoadError(String),

    #[error("Couldn't locate a suitable directory to keep your config in.")]
    HomeDirNotFound,

    #[error("Error with filesystem: {0}")]
    IO(#[from] io::Error),
}
#[derive(thiserror::Error, Debug)]
pub enum StatsError {
    #[error("Failed to create a PixelaUser: Credentials not provided")]
    UserNotProvided(),
    #[error("Stats module is off, turn on and login")]
    StatsTrackingTurnedOff(),
    #[error("You have no subjects, make sure to sync")]
    SubjectsAreEmpty,
    #[error("Can't upload this pixel")]
    WrongPixelData,
}
#[derive(thiserror::Error, Debug)]
pub enum PixelaResponseError {
    #[error("{1}: {0}")]
    RetryableError(String, reqwest::StatusCode),
    #[error("{1}: {0}")]
    FatalError(String, reqwest::StatusCode),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] io::Error),

    #[error("Settings Error: {0}")]
    SettingsError(#[from] SettingsError),

    #[error("Stats Error: {0}")]
    StatsError(#[from] StatsError),

    #[error("Request Error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Pixela Error: {0}")]
    PixelaResponseError(#[from] PixelaResponseError),

    #[error("Toml Deserialization Error: {0}")]
    TomlDeError(#[from] toml::de::Error),
    #[error("Toml Serialization Error: {0}")]
    TomlSerError(#[from] toml::ser::Error),
}
impl Error {
    pub fn handle_error_and_consume_data<T>(result: Result<T>) -> Option<Popup> {
        if let Err(e) = result {
            Some(Popup::from(e))
        } else {
            None
        }
    }
}
pub type Result<T> = std::result::Result<T, Error>;
