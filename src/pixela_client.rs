use std::{fs, process::exit, time::Duration, u8, vec};

use crate::{
    error::{Error, PixelaResponseError, Result, SettingsError, StatsError},
    graph::Graph,
    stats::{ComplexPixel, SimplePixel},
};
use chrono::{DateTime, Local};
use directories::{ProjectDirs, UserDirs};
use futures::{stream, StreamExt};
use ratatui::widgets::ListState;
use reqwest::Client;
use serde_json::Value;
use tokio::time;

use crate::stats::{self, Pixel, PixelaUser, Progress, Subject};
#[derive(Debug)]
pub struct PixelaClient {
    pub client: reqwest::Client,
    pub user: stats::PixelaUser,
    pub pixels: StatefulList<Pixel>,
    pub subjects: StatefulList<Subject>,
    pub logged_in: bool,
    focused_pane: u8, // 0:pixels; 1:user; 2:subjects
    pixels_to_send: Vec<usize>,
    current_subject_index: usize,
    current_graph: Option<Graph>,
}
impl PixelaClient {
    pub fn try_new(user: PixelaUser) -> Result<PixelaClient> {
        let pixel_res = PixelaClient::load_pixels(&user)?;
        let pixels = StatefulList::new(pixel_res);
        let pixel_len = pixels.items.len();
        Ok(PixelaClient {
            client: Client::new(),
            pixels,
            user,
            subjects: StatefulList::default(),
            logged_in: false,
            focused_pane: 1,
            pixels_to_send: vec![0; 2 * pixel_len],
            current_subject_index: 0,
            current_graph: None,
        })
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
                date.format("%Y/%m/%d %H:%M").to_string(),
            )));
        } else {
            self.pixels.push(Pixel::Simple(SimplePixel::new(
                progress.into(),
                date.format("%Y/%m/%d %H:%M").to_string(),
            )));
        }
    }
    pub fn select_pixel(&mut self, index: usize) {
        if index > self.pixels_to_send.len() - 1 {
            self.update_pixels_to_send_size();
        }
        if let Some(pixel) = self.pixels().get(index) {
            match pixel {
                Pixel::Simple(_) => self.pixels_to_send[index] = 2,
                Pixel::Complex(_) => self.pixels_to_send[index] = 1,
            }
        }
    }
    pub fn unselect_pixel(&mut self, elem: usize) {
        self.pixels_to_send[elem] = 0;
    }
    pub fn selected_to_send(&self, index: usize) -> bool {
        self.pixels_to_send[index] == 1
    }
    fn update_pixels_to_send_size(&mut self) {
        self.pixels_to_send
            .append(&mut vec![0, 2 * self.pixels_to_send.len()]);
    }
    pub fn pixels_to_send_is_empty(&self) -> bool {
        !self.pixels_to_send.contains(&1)
    }
    pub fn save_pixels(&self) -> Result<()> {
        let path = ProjectDirs::from("romodoro", "mejxedev", "romodoro")
            .ok_or(SettingsError::HomeDirNotFound)?;
        let path = path.config_dir();
        let path = path.join("pixels");
        if !path.exists() {
            fs::create_dir_all(&path)?
        }
        let filename = format!("{}.json", self.user.username());
        let json_string: String = serde_json::to_string(self.pixels.items()).unwrap();
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
            let val: Vec<Pixel> = serde_json::from_str(data).unwrap();
            Ok(val)
        } else {
            Err(Error::SettingsError(SettingsError::LoadError(
                "Cannot load your pixels".to_string(),
            )))
        }
    }
    pub async fn send_pixels(&mut self) -> Result<Vec<Pixel>> {
        let mut unresolved_pixels = Vec::<Pixel>::new();
        let mut resolved_pixels = Vec::<Pixel>::new();

        let mut pixel_pool = self.take_selected_pixels(true);
        while !pixel_pool.is_empty() {
            let pixels = std::mem::take(&mut pixel_pool);
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
                            pixel_pool.push(Pixel::Complex(pixel.clone()));
                            time::sleep(Duration::from_millis(100)).await;
                        }
                        _ => {
                            // if not expected/fatal error variant write it back to store later
                            unresolved_pixels.push(Pixel::Complex(pixel.clone()));
                            return Err(err);
                        }
                    }
                } else {
                    resolved_pixels.push(Pixel::Complex(pixel.clone()));
                }
            }
        }
        let len = self.pixels_to_send.len();

        self.pixels_to_send = vec![0; len];
        self.pixels.items.append(&mut unresolved_pixels);
        self.save_pixels()?;
        Ok(resolved_pixels)
    }
    pub fn get_current_subject(&self) -> Option<Subject> {
        self.subjects
            .items
            .get(self.current_subject_index())
            .cloned()
    }
    pub fn get_subject(&self, index: usize) -> Option<Subject> {
        self.subjects.items.get(index).cloned()
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
                    self.subjects.set_items(subjects);
                    break;
                }
            }
        }

        Ok(())
    }
    pub async fn request_graph(&mut self) -> Result<()> {
        if let Some(subject_index) = self.subjects.state.selected() {
            if let Some(subject) = self.get_subject(subject_index) {
                self.set_current_graph(Some(
                    Graph::download_graph(self.user.clone(), self.client.clone(), subject).await?,
                ));
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
                let mut subjects: Vec<Subject> =
                    serde_json::from_value(graphs_value.clone()).unwrap();
                for subject in &mut subjects {
                    subject.set_url(format!(
                        "https://pixe.la/v1/users/{}/graphs/{}",
                        username,
                        subject.id()
                    ));
                }
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
                        Err(PixelaResponseError::FatalError(
                            format!("Request denied: {}", err),
                            status,
                        )
                        .into())
                    }
                } else {
                    Err(err.into())
                }
            }
        }
    }
    pub fn change_focused_pane(&mut self, forward: bool) {
        match forward {
            true => self.focused_pane = (self.focused_pane.saturating_add(1)) % 3,
            false => {
                self.focused_pane = {
                    if self.focused_pane == 0 {
                        2
                    } else {
                        self.focused_pane.saturating_sub(1) % 3
                    }
                }
            }
        };
    }
    pub fn delete_pixel(&mut self) -> Result<()> {
        if let Some(index) = self.pixels.state.selected() {
            self.pixels.items.remove(index);
            self.save_pixels()?;
        };
        Ok(())
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
    pub fn select_next(&mut self) {
        match self.focused_pane {
            0 => self.pixels.select_next(),
            1 => {}
            2 => self.subjects.select_next(),
            _ => {}
        }
    }
    pub fn select_previous(&mut self) {
        match self.focused_pane {
            0 => self.pixels.select_previous(),
            1 => {}
            2 => self.subjects.select_previous(),
            _ => {}
        }
    }
    pub async fn push_pixel(&mut self) -> Result<()> {
        if let Some(index) = self.pixels.state.selected() {
            if let Some(Pixel::Complex(pixel)) = self.pixels().get(index) {
                loop {
                    let response = pixel.upload(self.client.clone(), &self.user).await;
                    match response {
                        Ok(_) => {
                            self.pixels().remove(index);
                            break;
                        }
                        Err(err) => match err {
                            Error::PixelaResponseError(PixelaResponseError::RetryableError(
                                _,
                                _,
                            )) => {
                                continue;
                            }
                            _ => return Err(err),
                        },
                    };
                }
            } else {
                return Err(StatsError::WrongPixelData.into());
            }
        }
        Ok(())
    }
    pub fn get_selected_pixels(&self) -> Vec<Pixel> {
        let pixels: Vec<usize> = self
            .pixels_to_send
            .iter()
            .enumerate()
            .filter_map(
                |(index, element)| {
                    if *element != 0 {
                        Some(index)
                    } else {
                        None
                    }
                },
            )
            .collect();
        pixels
            .iter()
            .filter_map(|index| self.pixels.items.get(*index).cloned())
            .collect()
    }
    pub fn take_selected_pixels(&mut self, complex_only: bool) -> Vec<Pixel> {
        let pixels: Vec<usize> = self
            .pixels_to_send
            .iter()
            .enumerate()
            .filter_map(|(index, element)| {
                if complex_only && *element == 1 {
                    Some(index)
                } else if !complex_only && *element != 0 {
                    Some(index)
                } else {
                    None
                }
            })
            .collect();
        pixels
            .into_iter()
            .rev()
            .map(|index| self.pixels.items.remove(index))
            .collect()
    }

    pub fn current_subject_index(&self) -> usize {
        self.current_subject_index
    }

    pub fn set_current_subject_index(&mut self, current_subject_index: usize) {
        self.current_subject_index = current_subject_index;
    }

    pub fn set_current_graph(&mut self, current_graph: Option<Graph>) {
        self.current_graph = current_graph;
    }

    pub fn current_graph(&self) -> Option<&Graph> {
        self.current_graph.as_ref()
    }

    pub fn current_graph_mut(&mut self) -> &mut Option<Graph> {
        &mut self.current_graph
    }
}
#[derive(Debug)]
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

    pub fn push(&mut self, value: T) {
        self.items.push(value)
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.items.iter()
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn state(&self) -> &ListState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut ListState {
        &mut self.state
    }

    pub fn select_next(&mut self) {
        self.state.select_next()
    }

    pub fn select_previous(&mut self) {
        self.state.select_previous()
    }
}
impl<T> Default for StatefulList<T> {
    fn default() -> StatefulList<T> {
        StatefulList {
            items: Vec::new(),
            state: ListState::default(),
        }
    }
}
