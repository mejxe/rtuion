use std::{fs, process::exit, vec};

use crate::{
    error::{Error, PixelaResponseError, Result, SettingsError, StatsError},
    stats::{ComplexPixel, SimplePixel},
};
use chrono::{DateTime, Local};
use directories::ProjectDirs;
use futures::{stream, StreamExt};
use ratatui::widgets::ListState;
use reqwest::Client;
use serde_json::Value;

use crate::stats::{self, Pixel, PixelaUser, Progress, Subject};
pub struct StatefulList<T> {
    items: Vec<T>,
    state: ListState,
}
impl<T> StatefulList<T> {
    pub fn new(items: Vec<T>) -> StatefulList<T> {
        Self {
            items,
            state: ListState::default(),
        }
    }
    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
    }
}
#[derive(Debug)]
pub struct PixelaClient {
    pub client: reqwest::Client,
    pub user: stats::PixelaUser,
    pub pixels: Vec<Pixel>,
    pub logged_in: bool,
    subjects: Vec<Subject>,
    focused_pane: u8,
}
impl PixelaClient {
    pub fn new(user: PixelaUser) -> PixelaClient {
        PixelaClient {
            client: Client::new(),
            pixels: PixelaClient::load_pixels(&user).unwrap_or_default(),
            user,
            subjects: Vec::new(),
            logged_in: false,
            focused_pane: 0,
        }
    }
    pub fn add_pixel(
        &mut self,
        date: DateTime<Local>,
        subject: Option<Subject>,
        progress: Progress,
    ) {
        if let Some(sub) = subject {
            self.pixels.push(Pixel::Complex(ComplexPixel::new(
                progress,
                sub,
                date.format("%Y/%m/%d").to_string(),
            )));
        } else {
            self.pixels.push(Pixel::Simple(SimplePixel::new(
                progress.into(),
                date.format("%Y/%m/%d %H:%M").to_string(),
            )));
        }
    }
    pub fn save_to_file(&self) -> Result<()> {
        if self.pixels.is_empty() {
            return Err(Error::SettingsError(SettingsError::SaveError(
                "Nothing to save".into(),
            )));
        }
        let path = ProjectDirs::from("romodoro", "mejxedev", "romodoro")
            .ok_or(SettingsError::HomeDirNotFound)?;
        let path = path.config_dir();
        let path = path.join("pixels");
        if !path.exists() {
            fs::create_dir_all(&path)?
        }
        let filename = format!("{}.json", self.user.username());
        let json_string: String = serde_json::to_string(&self.pixels).unwrap();
        fs::write(path.join(filename), json_string)?;
        Ok(())
    }
    fn load_pixels(user: &PixelaUser) -> Result<Vec<Pixel>> {
        let path = ProjectDirs::from("romodoro", "mejxedev", "romodoro")
            .ok_or(SettingsError::HomeDirNotFound)?;
        let path = path.config_dir();
        let path = path.join("pixels");
        if !path.exists() {
            fs::create_dir_all(&path)?;
            return Err(Error::SettingsError(SettingsError::LoadError(
                "User does not have a progress file".to_string(),
            )));
        };
        if let Ok(data) = &fs::read_to_string(path.join(format!("{}.json", user.username()))) {
            let val: Vec<Pixel> = serde_json::from_str(data).unwrap_or_default();
            Ok(val)
        } else {
            Err(Error::SettingsError(SettingsError::LoadError(
                "Cannot load your pixels".to_string(),
            )))
        }
    }
    async fn send_pixels(&mut self) {
        let mut unresolved_pixels = vec![];
        while !self.pixels.is_empty() {
            let pixels = std::mem::take(&mut self.pixels);
            let futures = pixels.iter().filter_map(|pixel| match pixel {
                Pixel::Complex(pixel) => {
                    let client_clone = self.client.clone();
                    let user_clone = self.user.clone();
                    let fut = async move {
                        let response = pixel.upload(client_clone, &user_clone).await;
                        (pixel, response)
                    };
                    Some(fut)
                }
                Pixel::Simple(_) => None,
            });
            let mut futures = stream::iter(futures).buffer_unordered(4);
            while let Some((pixel, result)) = futures.next().await {
                if let Err(err) = result {
                    match err {
                        Error::PixelaResponseError(PixelaResponseError::RetryableError(_, _)) => {
                            self.pixels.push(Pixel::Complex(pixel.clone()));
                        }
                        _ => {
                            // if not expected/fatal error variant write it back to store later
                            unresolved_pixels.push(Pixel::Complex(pixel.clone()));
                        }
                    }
                }
            }
        }
        self.pixels = unresolved_pixels;
    }
    pub fn get_subject(&self, index: u8) -> Option<Subject> {
        self.subjects.get(index as usize).cloned()
    }
    pub async fn log_in(&mut self) -> Result<()> {
        if !self.user.validate_not_empty() {
            return Err(StatsError::UserNotProvided().into());
        };
        self.sync_subjects().await?;
        self.logged_in = true;
        Ok(())
    }
    async fn sync_subjects(&mut self) -> Result<()> {
        let mut response = PixelaClient::request_subjects(
            self.client.clone(),
            self.user.username(),
            self.user.token(),
        );
        loop {
            match response.await {
                Err(resp) => {
                    match resp {
                        Error::PixelaResponseError(PixelaResponseError::RetryableError(_, _)) => {
                            response = PixelaClient::request_subjects(
                                self.client.clone(),
                                self.user.username(),
                                self.user.token(),
                            )
                        }
                        _ => return Err(resp),
                    };
                }
                Ok(subjects) => {
                    self.subjects = subjects;
                    break;
                }
            }
        }

        Ok(())
    }
    async fn request_subjects(
        client: Client,
        username: &str,
        api_key: &str,
    ) -> Result<Vec<Subject>> {
        let url = format!("https://pixe.la/v1/users/{}/graphs", username);
        let request = client
            .get(url)
            .header("X-USER-TOKEN", api_key)
            .send()
            .await?;
        match request.error_for_status() {
            Ok(request) => {
                let json_data: Value = request.json().await?;
                let graphs_value = &json_data["graphs"];
                let subjects: Vec<Subject> = serde_json::from_value(graphs_value.clone()).unwrap();
                Ok(subjects)
            }
            Err(err) => {
                if let Some(status) = err.status() {
                    if status == reqwest::StatusCode::SERVICE_UNAVAILABLE
                        || status == reqwest::StatusCode::INTERNAL_SERVER_ERROR
                    {
                        Err(
                            PixelaResponseError::RetryableError("Server is busy".into(), status)
                                .into(),
                        )
                    } else {
                        Err(PixelaResponseError::FatalError("Request denied".into(), status).into())
                    }
                } else {
                    Err(err.into())
                }
            }
        }
    }
    pub fn change_focused_pane(&mut self, forward: bool) {
        match forward {
            true => self.focused_pane = (self.focused_pane + 1) % 3,
            false => self.focused_pane = (self.focused_pane - 1) % 3,
        }
    }
    pub fn subjects(&self) -> Vec<&Subject> {
        self.subjects.iter().collect()
    }

    pub fn logged_in(&self) -> bool {
        self.logged_in
    }

    pub fn focused_pane(&self) -> u8 {
        self.focused_pane
    }

    pub fn pixels(&self) -> Vec<&Pixel> {
        self.pixels.iter().collect()
    }
}
