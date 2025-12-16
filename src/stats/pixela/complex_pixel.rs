use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::{
    helpers::{check_if_quantity_is_big_enough, get_from_pixela, send_to_pixela},
    pixela_user::PixelaUser,
    subjects::{Progress, Subject},
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct ComplexPixel {
    progress: Progress,
    date: String,
    subject: Subject,
}

impl ComplexPixel {
    pub fn new(progress: Progress, subject: Subject, date: String) -> ComplexPixel {
        ComplexPixel {
            progress,
            date,
            subject,
        }
    }
    pub async fn upload(&self, client: reqwest::Client, pixela_user: &PixelaUser) -> Result<()> {
        check_if_quantity_is_big_enough(self)?;
        send_to_pixela(self, &client, &pixela_user.token()).await
    }
    pub async fn increment_self_quantity_from_pixela_data(
        &mut self,
        client: reqwest::Client,
        pixela_user: &PixelaUser,
    ) -> Result<()> {
        self.progress += get_from_pixela(self, &client, &pixela_user.token()).await?;
        Ok(())
    }

    pub fn progress(&self) -> &Progress {
        &self.progress
    }

    pub fn date(&self) -> String {
        self.date.to_string()
    }
    pub fn date_no_time(&self) -> String {
        self.date.split(" ").next().unwrap().to_string()
    }
    pub fn date_pixela_formatted(&self) -> String {
        if !self.date.contains("/") {
            return self.date.clone();
        }
        self.date()
            .replace("/", "")
            .split_ascii_whitespace()
            .next()
            .unwrap()
            .to_string()
    }

    pub fn subject(&self) -> &Subject {
        &self.subject
    }
    pub fn into_aggregate(&mut self) {
        if self.subject().graph_name().contains("(Aggregated)") {
            return;
        }
        let new_name = format!("(Aggregated) {}", self.subject().graph_name());
        self.subject.set_graph_name(new_name);
    }
}
