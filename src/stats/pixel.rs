use serde::{Deserialize, Serialize};

use super::pixela::{
    complex_pixel::{ComplexPixel, IsRounded},
    subjects::Minutes,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone)]
pub enum Pixel {
    Simple(SimplePixel),
    Complex(ComplexPixel),
}
#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone)]
pub struct SimplePixel {
    progress: Minutes,
    date: String,
}
impl Pixel {
    pub fn check_if_value_rounded(&self) -> IsRounded {
        match self {
            Pixel::Simple(_) => IsRounded::No,
            Pixel::Complex(pix) => pix.is_rounded(),
        }
    }
}
impl SimplePixel {
    pub fn new(progress: Minutes, date: String) -> SimplePixel {
        SimplePixel { progress, date }
    }

    pub fn progress(&self) -> Minutes {
        self.progress
    }

    pub fn date(&self) -> &str {
        &self.date
    }
}
