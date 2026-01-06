use serde::{Deserialize, Serialize};
use strum_macros::Display;

use crate::{
    error::{Result, StatsError},
    HOUR_INCREMENT, MINUTES_INCREMENT,
};

use super::graph::PixelaColors;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub struct Subject {
    id: String,
    #[serde(default = "default_url")]
    url: String,
    #[serde(rename = "name")]
    graph_name: String,
    #[serde(rename = "type")]
    data_type: SubjectDataType,
    unit: SubjectUnit,
    color: PixelaColors,
}
fn default_url() -> String {
    String::new()
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Display, PartialOrd, Hash)]
pub enum SubjectDataType {
    #[serde(rename = "int")]
    Int,
    #[serde(rename = "float")]
    Float,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Display, PartialOrd, Hash)]
pub enum SubjectUnit {
    #[serde(rename = "minutes")]
    Minutes,
    #[serde(rename = "hours")]
    Hours,
}

impl Subject {
    fn try_new(
        id: String,
        url: String,
        graph_name: String,
        data_type: SubjectDataType,
        unit: SubjectUnit,
        color: PixelaColors,
    ) -> Result<Subject> {
        if data_type == SubjectDataType::Float && unit == SubjectUnit::Minutes {
            Err(StatsError::SubjectCreationFailed.into())
        } else {
            Ok(Subject {
                id,
                url,
                graph_name,
                data_type,
                unit,
                color,
            })
        }
    }
    pub fn new_dummy() -> Subject {
        Subject {
            id: "".to_string(),
            url: "".to_string(),
            graph_name: "None".to_string(),
            data_type: SubjectDataType::Int,
            unit: SubjectUnit::Minutes,
            color: PixelaColors::Ajisai,
        }
    }
    pub fn is_dummy(&self) -> bool {
        self.graph_name == *"None" && self.id == *""
    }
    pub fn transform_progress(&self, duration: i64) -> Progress {
        let mut progress: Progress = match self.data_type {
            SubjectDataType::Int => Progress::from(duration as i32),
            SubjectDataType::Float => Progress::from(duration as f64),
        };
        match self.unit {
            SubjectUnit::Minutes => progress.divide(60),
            SubjectUnit::Hours => progress.divide(3600),
        };
        progress
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn data_type(&self) -> &SubjectDataType {
        &self.data_type
    }

    pub fn unit(&self) -> &SubjectUnit {
        &self.unit
    }

    pub fn graph_name(&self) -> &str {
        &self.graph_name
    }
    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn color(&self) -> &PixelaColors {
        &self.color
    }

    pub fn set_graph_name(&mut self, graph_name: String) {
        self.graph_name = graph_name;
    }
    pub fn get_min_increment(&self) -> usize {
        // in minutes
        match self.unit() {
            SubjectUnit::Hours if *self.data_type() == SubjectDataType::Float => MINUTES_INCREMENT,
            SubjectUnit::Hours => HOUR_INCREMENT,
            SubjectUnit::Minutes => MINUTES_INCREMENT,
        }
    }
}
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Progress {
    Int(i32),
    Float(f64),
}
impl Progress {
    pub fn get_as_int(&self) -> i32 {
        match self {
            Progress::Int(num) => *num,
            Progress::Float(num) => num.floor() as i32,
        }
    }
    pub fn divide(&mut self, divider: i32) {
        match self {
            Progress::Float(num) => *num /= divider as f64,
            Progress::Int(num) => *num /= divider,
        }
    }
    pub fn minutes(&self, unit: &SubjectUnit) -> i32 {
        match unit {
            SubjectUnit::Hours => self.get_as_int() * 60,
            SubjectUnit::Minutes => self.get_as_int(),
        }
    }
    pub fn hours(&self, unit: &SubjectUnit) -> i32 {
        match unit {
            SubjectUnit::Hours => self.get_as_int(),
            SubjectUnit::Minutes => self.get_as_int() / 60,
        }
    }
}
