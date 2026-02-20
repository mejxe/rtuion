use serde::{Deserialize, Serialize};
use strum_macros::Display;

use crate::{
    error::{Result, StatsError},
    utils::misc,
    MIN_HOUR_INCREMENT, MIN_MINS_INCREMENT,
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
    #[serde(rename = "minutes", alias = "Minutes", alias = "mins", alias = "Mins")]
    Minutes,
    #[serde(rename = "hours", alias = "hrs", alias = "Hours")]
    Hours,
}
impl SubjectUnit {
    pub fn short_string(&self) -> String {
        match self {
            SubjectUnit::Hours => "h".to_string(),
            SubjectUnit::Minutes => "m".to_string(),
        }
    }
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
    pub fn shortened_graph_name(&self) -> String {
        self.graph_name
            .split_ascii_whitespace()
            .map(|w| w.chars().nth(0).unwrap_or('.'))
            .collect()
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
    pub fn get_min_increment(&self) -> Minutes {
        // in minutes
        match self.unit() {
            SubjectUnit::Hours if *self.data_type() == SubjectDataType::Float => MIN_MINS_INCREMENT,
            SubjectUnit::Hours => MIN_HOUR_INCREMENT,
            SubjectUnit::Minutes => MIN_MINS_INCREMENT,
        }
    }

    pub fn set_unit(&mut self, unit: SubjectUnit) {
        self.unit = unit;
    }
    pub fn set_datatype(&mut self, data_type: SubjectDataType) {
        self.data_type = data_type;
    }
}
pub enum TimeUnit {
    Hours(Hours),
    Minutes(Minutes),
    Seconds(Seconds),
}
pub type Hours = f64;
pub type HoursRounded = usize;
pub type Minutes = usize;
pub type Seconds = i64;
impl TimeUnit {
    pub fn round_hours(hours: Hours) -> HoursRounded {
        let f_part = hours % 1.0;
        if f_part > 0.8 {
            return hours.ceil() as usize;
        } else {
            return hours.floor() as usize;
        }
    }
}
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Progress {
    Int(Minutes),
    Float(Hours),
}
impl Progress {
    pub fn new_minutes(time: TimeUnit) -> Progress {
        match time {
            TimeUnit::Minutes(mins) => Progress::Int(mins),
            TimeUnit::Hours(hours) => Progress::Int((hours * 60.0) as usize),
            TimeUnit::Seconds(seconds) => Progress::Int((seconds / 60) as usize),
        }
    }
    pub fn get_as_int(&self) -> usize {
        match self {
            Progress::Int(num) => *num,
            Progress::Float(num) => TimeUnit::round_hours(*num) as usize,
        }
    }
    pub fn get_as_float(&self) -> f64 {
        match self {
            Progress::Int(num) => *num as f64,
            Progress::Float(num) => *num,
        }
    }
    pub fn divide(&mut self, divider: usize) {
        match self {
            Progress::Float(num) => *num /= divider as f64,
            Progress::Int(num) => *num /= divider,
        }
    }
    pub fn minutes(&self, unit: &SubjectUnit) -> Minutes {
        match unit {
            SubjectUnit::Hours => self.get_as_int() * 60,
            SubjectUnit::Minutes => self.get_as_int(),
        }
    }
    pub fn hours(&self, unit: &SubjectUnit) -> Hours {
        match unit {
            SubjectUnit::Hours => misc::round_to_precision(self.get_as_float(), 2),
            SubjectUnit::Minutes => misc::round_to_precision(self.get_as_float() / 60.0, 2),
        }
    }
    pub fn hours_floored(&self, unit: &SubjectUnit) -> HoursRounded {
        match unit {
            SubjectUnit::Hours => self.get_as_int(),
            SubjectUnit::Minutes => self.get_as_int() / 60,
        }
    }
}
