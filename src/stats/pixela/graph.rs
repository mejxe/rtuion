use core::f64;
use std::{collections::HashMap, fs};

use chrono::{Duration, NaiveDate};
use directories::ProjectDirs;
use ratatui::{style::Style, widgets::Bar};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    error::{Error, PixelaResponseError, Result, SettingsError},
    ui::{BLUE, GREEN, RED, YELLOW},
};

use super::{
    complex_pixel::ComplexPixel,
    pixela_user::PixelaUser,
    subjects::{Progress, Subject},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Graph {
    data: Vec<DataPoint>,
    color: PixelaColors,
    subject: Subject,
}
impl Graph {
    pub fn save_graph(&self) -> Result<()> {
        let path = ProjectDirs::from("romodoro", "mejxedev", "romodoro")
            .ok_or(SettingsError::HomeDirNotFound)?;
        let path = path.config_dir();
        let path = path.join("graphs");
        if !path.exists() {
            fs::create_dir_all(&path)?
        }
        let filename = format!("{}.json", self.subject.graph_name());
        let json_string: String = serde_json::to_string(self).expect("format should be correct");
        fs::write(path.join(filename), json_string)?;
        Ok(())
    }
    fn load_graph(name: String) -> Result<Graph> {
        let path = ProjectDirs::from("romodoro", "mejxedev", "romodoro")
            .ok_or(SettingsError::HomeDirNotFound)?;
        let path = path.config_dir();
        let path = path.join("graphs");
        if !path.exists() {
            fs::create_dir_all(&path)?;
            return Err(Error::SettingsError(SettingsError::LoadError(
                "User does not have a progress file".to_string(),
            )));
        };
        if let Ok(data) = &fs::read_to_string(path.join(format!("{}.json", name))) {
            let val: serde_json::Result<Graph> = serde_json::from_str(data);
            if let Ok(val) = val {
                Ok(val)
            } else {
                Err(Error::SettingsError(SettingsError::LoadError(
                    "Cannot load your graph".to_string(),
                )))
            }
        } else {
            Err(Error::SettingsError(SettingsError::LoadError(
                "Cannot load your graph".to_string(),
            )))
        }
    }
    pub async fn download_graph(
        user: PixelaUser,
        client: Client,
        subject: Subject,
    ) -> Result<Graph> {
        // gets 30 latests pixels
        let start_date = (chrono::Local::now() - Duration::days(29)).format("%Y%m%d");
        let today = (chrono::Local::now() + Duration::days(2)).format("%Y%m%d");
        let request = client
            .get(format!("{}/pixels", subject.url()))
            .query(
                &serde_json::json!({"from": start_date.to_string(),"to": today.to_string(), "withBody": true}),
            )
            .header("X-USER-TOKEN", user.token())
            .send()
            .await?;
        let status = request.status();
        match status.is_success() {
            true => {
                let json_data: Value = request.json().await?;
                let pixels_value = &json_data["pixels"];
                let data: serde_json::Result<Vec<DataPoint>> =
                    serde_json::from_value(pixels_value.to_owned());
                if let Ok(data_points) = data {
                    let mut g = Graph {
                        subject: subject.clone(),
                        data: data_points,
                        color: subject.color().clone(), // TODO: Hardcoded
                    };
                    g.fill_empty_days();
                    Ok(g)
                } else {
                    Err(PixelaResponseError::FatalError(
                        json_data
                            .get("message")
                            .expect("error message should be here")
                            .to_string(),
                        status,
                    )
                    .into())
                }
            }

            false => Err(PixelaResponseError::FatalError(
                request.error_for_status().expect_err("failed").to_string(),
                status,
            )
            .into()),
        }
    }
    pub fn datapoints_into_f64(&self) -> Vec<(f64, f64)> {
        let mut days: Vec<(f64, f64)> = vec![];
        self.data.iter().for_each(|point| {
            days.push((days.len() as f64, point.quantity_as_f64(&self.subject)));
        });
        days
    }
    pub fn datapoints_into_u64(&self) -> Vec<(u64, u64)> {
        let mut days: Vec<(u64, u64)> = vec![];
        self.data.iter().for_each(|point| {
            days.push((days.len() as u64, point.quantity_as_u64(&self.subject)));
        });
        days
    }
    fn fill_empty_days(&mut self) {
        let data_map: HashMap<NaiveDate, Progress> = self
            .data
            .iter()
            .map(|dp| {
                (
                    NaiveDate::parse_from_str(&dp.date, "%Y%m%d").unwrap(),
                    dp.quantity.clone(),
                )
            })
            .collect();
        let mut result: Vec<DataPoint> = vec![];
        let today = chrono::Local::now().naive_local().date();
        let mut start_date = today - Duration::days(29);
        while start_date <= today {
            let prgrs = data_map.get(&start_date).unwrap_or(&Progress::Int(0));
            result.push(DataPoint {
                date: start_date.format("%Y%m%d").to_string(),
                quantity: prgrs.to_owned(),
            });
            start_date += Duration::days(1);
        }
        self.data = result;
    }
    pub fn get_labels(&self) -> Vec<String> {
        self.data
            .iter()
            .map(|dp| dp.date.split_at(6).1.into())
            .collect()
    }
    pub fn get_bounds(&self) -> ((f64, f64), (f64, f64)) {
        // returns bounds in format (x_low, x_high), (y_low, y_high) | if empty returns (0, 0), (0,
        // 0)
        if !self.data.is_empty() {
            let x_low: f64 = -0.5;
            let x_high: f64 = self.data.len() as f64 + 0.5;
            let y_low: f64 = 0.0;
            let y_high: f64 = self.data().iter().fold(f64::NEG_INFINITY, |current, next| {
                current.max(next.quantity_as_f64(&self.subject))
            });
            return ((x_low, x_high), (y_low, y_high));
        }
        ((0.0, 0.0), (0.0, 0.0))
    }
    pub fn get_max_quantity(&self) -> u64 {
        self.data().iter().fold(u64::MIN, |current, next| {
            current.max(next.quantity_as_u64(&self.subject))
        })
    }
    pub fn into_bars(&self) -> Vec<Bar> {
        self.data()
            .iter()
            .map(|dp| {
                Bar::default()
                    .value(dp.quantity_as_u64(self.subject()))
                    .label(dp.label().into())
                    .style(Style::default().fg(self.subject().color().to_ratatui_color()))
            })
            .collect()
    }

    pub fn data(&self) -> &[DataPoint] {
        &self.data
    }

    pub fn color(&self) -> &PixelaColors {
        &self.color
    }

    pub fn subject(&self) -> &Subject {
        &self.subject
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DataPoint {
    date: String,
    quantity: Progress,
}
impl DataPoint {
    pub fn into_f64(&self, subject: &Subject) -> (f64, f64) {
        let date_f64: f64 = self.date.parse().expect("should be fine");
        let progress_f64: f64 = self.quantity.minutes(subject.unit()) as f64 / 60.0;
        (date_f64, progress_f64)
    }
    pub fn quantity_as_f64(&self, subject: &Subject) -> f64 {
        self.quantity.minutes(subject.unit()) as f64 / 60.0
    }
    pub fn quantity_as_u64(&self, subject: &Subject) -> u64 {
        self.quantity.minutes(subject.unit()) as u64 / 60
    }
    pub fn label(&self) -> String {
        self.date.split_at(6).1.into()
    }
    pub fn date(&self) -> &str {
        &self.date
    }
}
impl From<ComplexPixel> for DataPoint {
    fn from(value: ComplexPixel) -> Self {
        DataPoint {
            date: value.date().into(),
            quantity: value.progress().clone(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum PixelaColors {
    Shibafu,
    Momiji,
    Sora,
    Ichou,
    Ajisai,
    Kuro,
}
impl PixelaColors {
    pub fn to_ratatui_color(&self) -> ratatui::style::Color {
        match self {
            Self::Shibafu => GREEN,
            Self::Momiji => RED,
            Self::Sora => BLUE,
            Self::Ichou => YELLOW,
            Self::Ajisai => ratatui::style::Color::LightMagenta,
            Self::Kuro => ratatui::style::Color::DarkGray,
        }
    }
}
