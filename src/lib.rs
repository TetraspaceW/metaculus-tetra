use log::info;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;

pub struct Metaculus<'a> {
    domain: &'a str,
}

impl Metaculus<'_> {
    pub fn new() -> Metaculus<'static> {
        Metaculus {
            domain: "www",
        }
    }

    pub fn get_prediction_for(&self, id: &str) -> Option<f64> {
        self.get_question(id)?.get_best_prediction()
    }

    pub fn get_question(&self, id: &str) -> Option<Question> {
        let url = format!(
            "https://{}.metaculus.com/api2/questions/{}",
            self.domain, id
        );
        let response = reqwest::blocking::get(url).ok()?.text().ok()?;
        println!("{}", response);
        let question_response = serde_json::from_str(&response).ok()?;
        info!("Question id {} retrieved successfully.", id);
        return Some(question_response);
    }
}

#[derive(Serialize, Deserialize)]
pub struct Question {
    pub title_short: String,
    prediction_timeseries: Option<Vec<PredictionTimeseriesPoint>>,
    metaculus_prediction: Option<MetaculusPrediction>,
    resolution: Option<f64>,
}

impl Question {
    pub fn get_best_prediction(&self) -> Option<f64> {
        return match self.get_resolution() {
            None => match self.get_metaculus_prediction() {
                None => self.get_community_prediction(),
                mp => mp,
            },
            r => r,
        };
    }

    pub fn get_community_prediction(&self) -> Option<f64> {
        Some(
            self.prediction_timeseries.as_ref()?
                .last()
                .unwrap()
                .community_prediction,
        )
    }

    pub fn get_metaculus_prediction(&self) -> Option<f64> {
        Some(self.metaculus_prediction.as_ref()?.full)
    }

    pub fn get_resolution(&self) -> Option<f64> {
        Some(self.resolution?)
    }
}

#[derive(Serialize, Deserialize)]
struct PredictionTimeseriesPoint {
    community_prediction: f64,
}

#[derive(Serialize, Deserialize)]
struct MetaculusPrediction {
    full: f64,
}
