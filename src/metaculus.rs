use log::info;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;

pub struct Metaculus {
    domain: String,
}

impl Metaculus {
    pub fn new() -> Metaculus {
        Metaculus { domain: "www".to_string() }
    }

    pub fn get_prediction_for(&self, id: &str) -> Option<f64> {
        self.get_question(id)?.get_best_prediction()
    }

    pub fn get_question(&self, id: &str) -> Option<QuestionResponse> {
        let url = format!("https://{}.metaculus.com/api2/questions/{}", self.domain, id);
        let response = reqwest::blocking::get(url).ok()?.text().ok()?;

        let question_response = serde_json::from_str(&response).ok()?;
        info!("Question id {} retrieved successfully.", id);
        return Some(question_response);
    }
}

#[derive(Serialize, Deserialize)]
pub struct QuestionResponse {
    pub title_short: String,
    prediction_timeseries: Vec<PredictionPoint>,
    metaculus_prediction: Option<MetaculusPrediction>,
    resolution: Option<f64>,
}

impl QuestionResponse {
    pub fn get_best_prediction(&self) -> Option<f64> {
        let best_prediction = match self.get_resolution() {
            None => match self.get_metaculus_prediction() {
                None => self.get_community_prediction(),
                mp => mp
            },
            r => r
        };
        best_prediction
    }

    pub fn get_community_prediction(&self) -> Option<f64> {
        Some(self.prediction_timeseries.last().unwrap().community_prediction)
    }

    pub fn get_metaculus_prediction(&self) -> Option<f64> {
        Some(self.metaculus_prediction.as_ref()?.full)
    }

    pub fn get_resolution(&self) -> Option<f64> {
        Some(self.resolution?)
    }
}

#[derive(Serialize, Deserialize)]
struct PredictionPoint {
    community_prediction: f64,
}

#[derive(Serialize, Deserialize)]
struct MetaculusPrediction {
    full: f64,
}
