use std::{
    fmt::{write, Display},
    io,
};

use crate::{popup::Popup, stats::pixel::Pixel};

#[derive(thiserror::Error, Debug)]
pub enum SettingsError {
    #[error("Update failed! Stop the timer first")]
    UpdateError(),

    #[error("There was an error with saving your data: {0}")]
    SaveError(String),

    #[error("There was an error with loading your data: {0}")]
    LoadError(String),

    #[error("Saved pixels not found, loading the defaults")]
    NoData(),

    #[error("Couldn't locate a suitable directory to keep your config in.")]
    HomeDirNotFound,

    #[error("Error with filesystem: {0}")]
    IO(#[from] io::Error),

    #[error("Incorrect setting value: {0}")]
    WrongSetting(String),
}
#[derive(thiserror::Error, Debug)]
pub enum StatsError {
    #[error("Failed to create a PixelaUser: Credentials not provided")]
    UserNotProvided(),
    #[error("Stats module is off, turn on and login")]
    StatsTrackingTurnedOff(),
    #[error("You have no subjects, make sure to sync")]
    SubjectsAreEmpty,
    #[error("The sync has failed")]
    SubjectsSyncFailed,
    #[error("Cannot create a subject with these parameters")]
    SubjectCreationFailed,
    #[error("Can't upload this pixel")]
    WrongPixelData,
    #[error("Your graph cannot store progress this small")]
    QuantityIsNotBigEnough,
}
#[derive(thiserror::Error, Debug)]
pub enum PixelaResponseError {
    #[error("{1}: {0}")]
    RetryableError(String, reqwest::StatusCode),
    #[error("{1}: {0}")]
    FatalError(String, reqwest::StatusCode),
    #[error("{}", .errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n"))]
    FatalSendingPixelsError {
        errors: Vec<FatalError>,
        pixels: Vec<Pixel>,
    },
}
#[derive(Debug)]
pub struct FatalError {
    message: String,
    code: reqwest::StatusCode,
}
impl FatalError {
    pub fn new(message: String, code: reqwest::StatusCode) -> Self {
        Self { message, code }
    }
}
impl TryFrom<Error> for FatalError {
    type Error = self::Error;
    fn try_from(value: Error) -> std::result::Result<Self, Error> {
        match value {
            Error::PixelaResponseError(PixelaResponseError::FatalError(message, code)) => {
                Ok(Self { message, code })
            }
            _ => Err(Error::MiscError("Not supported conversion".into())),
        }
    }
}
impl Display for FatalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
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
    #[error("Async Error: {0}")]
    AsyncError(String),
    #[error("Misc Error: {0}")]
    MiscError(String),
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
