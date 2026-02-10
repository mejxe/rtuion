use std::{collections::HashMap, fs, future::Future, process::exit, time::Duration, vec};

use crate::{
    error::{Error, FatalError, PixelaResponseError, Result, SettingsError, StatsError},
    stats::{
        pixel::{Pixel, SimplePixel},
        pixela::subjects::SubjectDataType,
    },
};
use chrono::{DateTime, Local};
use directories::ProjectDirs;
use futures::{stream, StreamExt};
use reqwest::{Client, StatusCode};
use serde_json::Value;
use tokio::time;

use super::{
    complex_pixel::ComplexPixel,
    graph::Graph,
    pixela_user::PixelaUser,
    subjects::{Progress, Subject, SubjectUnit},
    utils::StatefulList,
};
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PixelaTabs {
    Pixels,
    Subject,
}
impl PixelaTabs {
    fn right(&mut self) {
        *self = match self {
            PixelaTabs::Pixels => PixelaTabs::Subject,
            PixelaTabs::Subject => PixelaTabs::Subject,
        };
    }
    fn left(&mut self) {
        *self = match self {
            PixelaTabs::Pixels => PixelaTabs::Pixels,
            PixelaTabs::Subject => PixelaTabs::Pixels,
        };
    }
}

#[derive(Debug)]
pub struct PixelaClient {
    pub client: reqwest::Client,
    pub user: PixelaUser,
    pub pixels: StatefulList<Pixel>,
    pub subjects: StatefulList<Subject>,
    pub logged_in: bool,
    focused_pane: PixelaTabs,
    pixels_to_send: Vec<usize>,
    current_subject_index: usize,
    current_graph: Option<Graph>,
}
impl PixelaClient {
    pub fn try_new(user: PixelaUser) -> Result<PixelaClient> {
        let pixel_res = PixelaClient::load_pixels(&user);
        let pixel_vec: Vec<Pixel> = match pixel_res {
            Ok(pixels) => pixels,
            Err(err) => {
                if let Error::SettingsError(SettingsError::NoData()) = err {
                    Vec::new()
                } else {
                    return Err(err);
                }
            }
        };
        let pixels = StatefulList::new(pixel_vec);
        let pixel_len = pixels.items().len();
        let mut subjects: StatefulList<Subject> = StatefulList::default();
        subjects.push(Subject::new_dummy());
        Ok(PixelaClient {
            client: Client::new(),
            pixels,
            user,
            subjects,
            logged_in: false,
            focused_pane: PixelaTabs::Subject,
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
        if self.pixels.items().len() > self.pixels_to_send.len() {
            self.update_pixels_to_send_size();
        }
    }
    pub fn select_pixel(&mut self, index: usize) {
        if let Some(Pixel::Complex(_)) = self.pixels().get(index) {
            self.pixels_to_send[index] = 1
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
        let path = ProjectDirs::from("rtuion", "rtuion", "rtuion")
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
        let path = ProjectDirs::from("rtuion", "rtuion", "rtuion")
            .ok_or(SettingsError::HomeDirNotFound)?;
        let path = path.config_dir();
        let path = path.join("pixels");
        if !path.exists() {
            fs::create_dir_all(&path)?;
            return Err(Error::SettingsError(SettingsError::LoadError(
                "User does not have a progress file".to_string(),
            )));
        };
        if !path.join(format!("{}.json", user.username())).exists() {
            return Err(SettingsError::NoData().into());
        }
        if let Ok(data) = &fs::read_to_string(path.join(format!("{}.json", user.username()))) {
            let val: Vec<Pixel> = serde_json::from_str(data).unwrap();
            Ok(val)
        } else {
            Err(Error::SettingsError(SettingsError::LoadError(
                "Cannot load your pixels".to_string(),
            )))
        }
    }
    pub async fn send_and_handle_response<F, Fut>(
        &mut self,
        mut pixel_pool: Vec<Pixel>,
        mut func: F,
    ) -> Result<Vec<Pixel>>
    where
        F: FnMut(ComplexPixel, Client, PixelaUser) -> Fut,
        Fut: Future<Output = (ComplexPixel, Result<()>)>,
    {
        let mut unresolved_pixels = Vec::<Pixel>::new();
        let mut resolved_pixels = Vec::<Pixel>::new();
        let mut errors: Vec<FatalError> = vec![];
        while !pixel_pool.is_empty() {
            let pixels = std::mem::take(&mut pixel_pool);
            let futures = pixels.into_iter().filter_map(|pixel| match pixel {
                Pixel::Complex(pixel) => {
                    let client_clone = self.client.clone();
                    let user_clone = self.user.clone();
                    Some(func(pixel, client_clone, user_clone))
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
                        Error::PixelaResponseError(PixelaResponseError::FatalError(_, _)) => {
                            // if not expected fatal error write it back to store later
                            unresolved_pixels.push(Pixel::Complex(pixel.clone()));
                            errors.push(err.try_into()?);
                        }
                        _ => {
                            unresolved_pixels.push(Pixel::Complex(pixel.clone()));
                            errors.push(FatalError::new(err.to_string(), StatusCode::IM_A_TEAPOT));
                        }
                    }
                } else {
                    resolved_pixels.push(Pixel::Complex(pixel.clone()));
                }
            }
        }
        if !errors.is_empty() {
            return Err(PixelaResponseError::FatalSendingPixelsError {
                errors,
                pixels: unresolved_pixels,
            }
            .into());
        }
        Ok(resolved_pixels)
    }
    pub async fn send_pixels(&mut self) -> Result<Vec<Pixel>> {
        let pixel_pool = PixelaClient::combine_similiar_pixels(self.take_selected_pixels(true));
        let operation = |pixel: ComplexPixel, client: Client, user: PixelaUser| async move {
            let response = pixel.upload(client, &user).await;
            (pixel, response)
        };
        let result: Result<Vec<Pixel>> =
            match self.send_and_handle_response(pixel_pool, operation).await {
                Ok(resolved_pixels) => Ok(resolved_pixels),
                Err(mut err) => {
                    if let Error::PixelaResponseError(
                        PixelaResponseError::FatalSendingPixelsError { errors: _, pixels },
                    ) = &mut err
                    {
                        self.pixels.items_mut().append(pixels);
                    }
                    Err(err)
                }
            };
        let len = self.pixels_to_send.len();

        self.pixels_to_send = vec![0; len];
        self.pixels.refresh_state();
        self.save_pixels()?;
        result
    }
    pub fn get_current_subject(&self) -> Option<Subject> {
        if self.current_subject_index == 0 {
            return None;
        };
        self.subjects
            .items()
            .get(self.current_subject_index())
            .cloned()
    }
    pub fn get_subject(&self, index: usize) -> Option<Subject> {
        self.subjects.items().get(index).cloned()
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
                Ok(mut subjects) => {
                    self.subjects.items_mut().append(&mut subjects);
                    break;
                }
            }
        }

        Ok(())
    }
    pub async fn request_graph(&mut self) -> Result<()> {
        if let Some(subject_index) = self.subjects.state().selected() {
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
                let mut json_data: Value = request.json().await?;
                if let Some(graphs_value) = json_data["graphs"].take().as_array_mut() {
                    let mut subjects: Vec<Subject> = graphs_value
                        .iter_mut()
                        .filter_map(|subject| serde_json::from_value(subject.take()).ok())
                        .filter_map(|subject: Subject| {
                            match *subject.unit() == SubjectUnit::Minutes
                                && *subject.data_type() == SubjectDataType::Float
                            {
                                true => None,
                                false => Some(subject),
                            }
                        })
                        .collect();
                    for subject in &mut subjects {
                        subject.set_url(format!(
                            "https://pixe.la/v1/users/{}/graphs/{}",
                            username,
                            subject.id()
                        ));
                    }
                    Ok(subjects)
                } else {
                    Err(PixelaResponseError::FatalError(
                        "Request sucessful but response not serializable".to_string(),
                        reqwest::StatusCode::NO_CONTENT,
                    )
                    .into())
                }
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
            true => self.focused_pane.right(),
            false => self.focused_pane.left(),
        }
    }
    pub fn delete_pixel(&mut self) -> Result<()> {
        if let Some(index) = self.pixels.state().selected() {
            self.pixels.items_mut().remove(index);
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

    pub fn pixels(&self) -> Vec<&Pixel> {
        self.pixels.iter().collect()
    }
    pub fn select_next(&mut self) {
        match self.focused_pane {
            PixelaTabs::Pixels => self.pixels.select_next(),
            PixelaTabs::Subject => self.subjects.select_next(),
        }
    }
    pub fn select_previous(&mut self) {
        match self.focused_pane {
            PixelaTabs::Pixels => self.pixels.select_previous(),
            PixelaTabs::Subject => self.subjects.select_previous(),
        }
    }
    pub async fn push_pixel(&mut self) -> Result<()> {
        if let Some(index) = self.pixels.state().selected() {
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
            .filter_map(|index| self.pixels.items().get(*index).cloned())
            .collect()
    }
    pub fn combine_similiar_pixels(pixels: Vec<Pixel>) -> Vec<Pixel> {
        let pixels: Vec<Pixel> = pixels
            .into_iter()
            .filter_map(|pixel| match pixel {
                Pixel::Complex(pixel) => Some(pixel),
                Pixel::Simple(_) => None,
            })
            .fold(
                HashMap::<(String, Subject), (usize, usize)>::new(),
                // (Date, Subject): (total_quantity, values_summed_up)
                |mut acc, pixel| {
                    let key = (pixel.date_no_time().clone(), pixel.subject().clone());
                    let total_quantity = acc.entry(key).or_insert((0, 0));
                    total_quantity.0 += pixel.progress().minutes(&SubjectUnit::Minutes) as usize; //
                    total_quantity.1 += 1;
                    acc
                },
            )
            .into_iter()
            .map(|entry| {
                let mut pixel = ComplexPixel::new(
                    Progress::Int(entry.1 .0),
                    entry.0 .1.clone(),
                    entry.0 .0.clone().to_string(),
                );
                if entry.1 .1 > 1 {
                    pixel.into_aggregate();
                }
                Pixel::Complex(pixel)
            })
            .collect();
        pixels
    }
    pub fn take_selected_pixels(&mut self, complex_only: bool) -> Vec<Pixel> {
        let pixels: Vec<usize> = self
            .pixels_to_send
            .iter()
            .enumerate()
            .filter_map(|(index, element)| {
                if complex_only && *element == 1 || !complex_only && *element != 0 {
                    Some(index)
                } else {
                    None
                }
            })
            .collect();
        pixels
            .into_iter()
            .rev()
            .map(|index| self.pixels.items_mut().remove(index))
            .collect()
    }
    pub fn clone_selected_pixels(&self, complex_only: bool) -> Vec<Pixel> {
        let pixels: Vec<usize> = self
            .pixels_to_send
            .iter()
            .enumerate()
            .filter_map(|(index, element)| {
                if complex_only && *element == 1 || !complex_only && *element != 0 {
                    Some(index)
                } else {
                    None
                }
            })
            .collect();
        pixels
            .into_iter()
            .rev()
            .map(|index| self.pixels.items().get(index).unwrap().clone())
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

    pub fn focused_pane(&self) -> PixelaTabs {
        self.focused_pane
    }
}
