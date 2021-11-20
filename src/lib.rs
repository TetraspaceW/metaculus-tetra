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
    metaculus_prediction: Option<MetaculusPredictionTimeseriesPoint>,
    resolution: Option<f64>,
    possibilities: QuestionPossibilities,
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
            } => self.convert_range_prediction(community_prediction.q2)?,
        };

        Some(community_prediction)
    }

    pub fn get_metaculus_prediction(&self) -> Option<f64> {
        let metaculus_prediction = match self.metaculus_prediction.as_ref()? {
            MetaculusPredictionTimeseriesPoint::NumericMPTP { full } => *full,
            MetaculusPredictionTimeseriesPoint::RangeMPTP { full } => {
                self.convert_range_prediction(full.q2)?
            }
        };

        Some(metaculus_prediction)
    }

    fn convert_range_prediction(&self, prediction: f64) -> Option<f64> {
        let scale = self.possibilities.scale.as_ref()?;
        let min = scale.min;
        let max = scale.max;

        Some(prediction * (max - min) + min)
    }

    pub fn get_resolution(&self) -> Option<f64> {
        return if self.possibilities.question_type == "continuous" {
            Some(self.convert_range_prediction(self.resolution?)?)
        } else {
            Some(self.resolution?)
        };
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
enum MetaculusPredictionTimeseriesPoint {
    NumericMPTP { full: f64 },
    RangeMPTP { full: RangeMetaculusPrediction },
}

#[derive(Serialize, Deserialize)]
struct RangeMetaculusPrediction {
    q2: f64,
}

#[derive(Serialize, Deserialize)]
struct QuestionPossibilities {
    #[serde(rename = "type")]
    question_type: String,
    scale: Option<RangeQuestionScale>,
}

#[derive(Serialize, Deserialize)]
struct RangeQuestionScale {
    min: f64,
    max: f64,
}
