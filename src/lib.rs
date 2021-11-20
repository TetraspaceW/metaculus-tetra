use log::info;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json;

pub struct Metaculus<'a> {
    domain: &'a str,
    client: Client,
}

impl Metaculus<'_> {
    pub fn new() -> Metaculus<'static> {
        Metaculus {
            domain: "www",
            client: Client::new(),
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
        let response = self.client.get(url).send().ok()?.text().ok()?;
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
        let community_prediction = match self.prediction_timeseries.as_ref()?.last()? {
            PredictionTimeseriesPoint::NumericPTP {
                community_prediction,
            } => *community_prediction,
            PredictionTimeseriesPoint::RangePTP {
                community_prediction,
            } => community_prediction.q2,
        };

        Some(community_prediction)
    }

    pub fn get_metaculus_prediction(&self) -> Option<f64> {
        let metaculus_prediction = match self.metaculus_prediction.as_ref()? {
            MetaculusPrediction::NumericMP { full } => *full,
            MetaculusPrediction::RangeMP { full } => full.q2,
        };

        Some(metaculus_prediction)
    }

    pub fn get_resolution(&self) -> Option<f64> {
        Some(self.resolution?)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum PredictionTimeseriesPoint {
    NumericPTP {
        community_prediction: f64,
    },
    RangePTP {
        community_prediction: RangeCommunityPrediction,
    },
}

#[derive(Serialize, Deserialize)]
struct RangeCommunityPrediction {
    q2: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum MetaculusPrediction {
    NumericMP { full: f64 },
    RangeMP { full: RangeMetaculusPrediction },
}

#[derive(Serialize, Deserialize)]
struct RangeMetaculusPrediction {
    q2: f64,
}
