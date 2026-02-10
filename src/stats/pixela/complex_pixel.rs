use serde::{Deserialize, Serialize};

use crate::{error::Result, stats::pixela::subjects::SubjectDataType};

use super::{
    helpers::{check_if_quantity_is_big_enough, send_to_pixela},
    pixela_user::PixelaUser,
    subjects::{Progress, Subject, SubjectUnit},
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
        send_to_pixela(self, &client, pixela_user.token()).await
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
    pub fn sendable_string(&self) -> String {
        match (self.subject.unit(), self.subject.data_type()) {
            (&SubjectUnit::Hours, &SubjectDataType::Int) => {
                format!("{}", self.progress.hours_floored(&SubjectUnit::Minutes))
            }
            (&SubjectUnit::Hours, &SubjectDataType::Float) => {
                format!("{}", self.progress.hours(&SubjectUnit::Minutes))
            }
            (&SubjectUnit::Minutes, &SubjectDataType::Int) => {
                format!("{}", self.progress.minutes(&SubjectUnit::Minutes))
            }
            _ => panic!("Not allowed"),
        }
    }
    pub fn display_string(&self) -> String {
        match (self.subject.unit(), self.subject.data_type()) {
            (&SubjectUnit::Hours, &SubjectDataType::Float) => {
                format!("{}h", self.progress.hours(&SubjectUnit::Minutes))
            }
            (&SubjectUnit::Minutes, &SubjectDataType::Int)
            | (&SubjectUnit::Hours, &SubjectDataType::Int) => {
                format!("{}m", self.progress.minutes(&SubjectUnit::Minutes))
            }
            _ => panic!("Not allowed"),
        }
    }

    pub fn subject(&self) -> &Subject {
        &self.subject
    }
    pub fn into_aggregate(&mut self) {
        if self.subject().graph_name().contains("Aggregated") {
            return;
        }
        let new_name = format!("(Aggregated) {}", self.subject().graph_name());
        self.subject.set_graph_name(new_name);
    }

    pub(crate) fn is_rounded(&self) -> IsRounded {
        match (self.subject().unit(), self.subject().data_type()) {
            (SubjectUnit::Hours, SubjectDataType::Int) => {
                let value = self.progress().minutes(&SubjectUnit::Minutes) as f64 / 60.0;
                let value_to_send = self.progress().hours_floored(&SubjectUnit::Minutes) as f64;
                if !(value == value_to_send) {
                    IsRounded::Yes(value_to_send as usize)
                } else {
                    IsRounded::No
                }
            }
            _ => IsRounded::No,
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum IsRounded {
    Yes(usize),
    No,
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounding_works() {
        let progress = Progress::Float(80.0);
        let mut subject = Subject::new_dummy();
        subject.set_unit(SubjectUnit::Hours);
        subject.set_datatype(SubjectDataType::Float);
        let pixel = ComplexPixel::new(progress, subject, "80".to_string());
        assert_eq!(pixel.is_rounded(), IsRounded::No)
    }
}
