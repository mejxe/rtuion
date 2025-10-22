use std::fs;

use crate::error::{Error, Result, SettingsError};
use chrono::{Date, DateTime, Local};
use directories::ProjectDirs;
use reqwest::Client;

use crate::stats::{self, Pixel, PixelaUser, Progress, Subject};
#[derive(Debug)]
pub struct PixelaClient {
    client: reqwest::Client,
    user: stats::PixelaUser,
    pixels: Vec<Pixel>,
}
impl PixelaClient {
    pub fn try_new(user: PixelaUser) -> Result<PixelaClient> {
        Ok(PixelaClient {
            client: Client::new(),
            pixels: PixelaClient::load_pixels(&user)?,
            user,
        })
    }
    pub fn add_pixel(&mut self, date: DateTime<Local>, subject: Subject, progress: Progress) {
        self.pixels
            .push(Pixel::new(progress, subject, date.to_string()));
    }
    pub fn save_to_file(&self) -> Result<()> {
        let path = ProjectDirs::from("romodoro", "mejxedev", "romodoro")
            .ok_or(SettingsError::HomeDirNotFound)?;
        let path = path.config_dir();
        let path = path.join("pixels");
        if !path.exists() {
            fs::create_dir_all(&path)?
        }
        let filename = format!("{}.toml", self.user.username());
        let toml_cfg: String = toml::to_string(&self.pixels).expect("pixels instantiated");
        fs::write(path.join(filename), toml_cfg)?;
        Ok(())
        // TODO: test if it works
    }
    fn load_pixels(user: &PixelaUser) -> Result<Vec<Pixel>> {
        let path = ProjectDirs::from("romodoro", "mejxedev", "romodoro")
            .ok_or(SettingsError::HomeDirNotFound)?;
        let path = path.config_dir();
        if !path.exists() {
            return Err(Error::SettingsError(SettingsError::LoadError(
                "User does not have a progress file".to_string(),
            )));
        };
        if let Ok(data) = &fs::read_to_string(path.join(format!("{}.toml", user.username()))) {
            let val: Vec<Pixel> = toml::from_str(data).unwrap();
            Ok(val)
        } else {
            Err(Error::SettingsError(SettingsError::LoadError(
                "Cannot load your pixels".to_string(),
            )))
        }
    }
}
