use serde::{Deserialize, Serialize};

use super::pixela::complex_pixel::ComplexPixel;

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone)]
pub enum Pixel {
    Simple(SimplePixel),
    Complex(ComplexPixel),
}
#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone)]
pub struct SimplePixel {
    progress: i32,
    date: String,
}
impl SimplePixel {
    pub fn new(progress: i32, date: String) -> SimplePixel {
        SimplePixel { progress, date }
    }

    pub fn progress(&self) -> i32 {
        self.progress
    }

    pub fn date(&self) -> &str {
        &self.date
    }
}
