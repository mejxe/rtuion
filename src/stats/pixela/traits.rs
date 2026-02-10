use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::stats::pixela::subjects::{Minutes, SubjectUnit};

use super::{subjects::Progress, utils::StatefulList};

impl<T> Default for StatefulList<T> {
    fn default() -> StatefulList<T> {
        StatefulList::new(Vec::new())
    }
}
impl std::ops::AddAssign<usize> for Progress {
    fn add_assign(&mut self, rhs: usize) {
        match self {
            Progress::Int(i) => *i += rhs as usize,
            Progress::Float(f) => *f += rhs as f64,
        }
    }
}

impl Serialize for Progress {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Progress::Int(i) => serializer.serialize_f64(*i as f64),
            Progress::Float(f) => serializer.serialize_f64(*f),
        }
    }
}

impl<'de> Deserialize<'de> for Progress {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde_json::Value;

        let value = Value::deserialize(deserializer)?;

        let num = match value {
            Value::String(s) => s
                .parse::<f64>()
                .map_err(|_| serde::de::Error::custom("Invalid number string"))?,
            Value::Number(n) => n
                .as_f64()
                .ok_or_else(|| serde::de::Error::custom("Invalid number"))?,
            _ => return Err(serde::de::Error::custom("Expected string or number")),
        };

        if num.fract() == 0.0 && num.abs() < i32::MAX as f64 {
            Ok(Progress::Int(num as Minutes))
        } else {
            Ok(Progress::Float(num))
        }
    }
}
impl Display for Progress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Progress::Int(_) => write!(f, "{}", self.hours_floored(&SubjectUnit::Minutes)),
            Progress::Float(_) => write!(f, "{}", self.hours_floored(&SubjectUnit::Hours)),
        }
    }
}
impl From<usize> for Progress {
    fn from(value: usize) -> Self {
        Progress::Int(value)
    }
}
impl Into<usize> for Progress {
    fn into(self) -> usize {
        match self {
            Progress::Int(int) => int,
            Progress::Float(f) => f as usize,
        }
    }
}
impl From<f64> for Progress {
    fn from(value: f64) -> Self {
        Progress::Float(value)
    }
}
