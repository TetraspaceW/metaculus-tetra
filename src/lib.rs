//! A library for interacting with Metaculus.
//!
//! [Repository](https://github.com/TetraspaceW/metaculus-tetra)
//!

use crate::Prediction::{AmbP, DatP, NumP};
use chrono::{NaiveDate, NaiveDateTime};
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

    pub fn get_numeric_prediction_for(&self, id: &str) -> Option<f64> {
        self.get_prediction_for(id)?.get_if_numeric()
    }

    pub fn get_date_prediction_for(&self, id: &str) -> Option<NaiveDateTime> {
        self.get_prediction_for(id)?.get_if_date()
    }

    pub fn get_prediction_for(&self, id: &str) -> Option<Prediction> {
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
    pub fn get_best_prediction(&self) -> Option<Prediction> {
        return match self.get_resolution() {
            None => match self.get_metaculus_prediction() {
                None => self.get_community_prediction(),
                mp => mp,
            },
            r => r,
        };
    }

    pub fn get_community_prediction(&self) -> Option<Prediction> {
        let community_prediction = match self.prediction_timeseries.as_ref()?.last()? {
            PredictionTimeseriesPoint::NumericPTP {
                community_prediction,
            } => NumP(*community_prediction),
            PredictionTimeseriesPoint::RangePTP {
                community_prediction,
            } => self.convert_range_prediction(community_prediction.q2)?,
        };

        Some(community_prediction)
    }

    pub fn get_metaculus_prediction(&self) -> Option<Prediction> {
        let metaculus_prediction = match self.metaculus_prediction.as_ref()? {
            MetaculusPredictionTimeseriesPoint::NumericMPTP { full } => NumP(*full),
            MetaculusPredictionTimeseriesPoint::RangeMPTP { full } => {
                self.convert_range_prediction(full.q2)?
            }
        };

        Some(metaculus_prediction)
    }

    fn convert_range_prediction(&self, prediction: f64) -> Option<Prediction> {
        let scale = self.possibilities.scale.as_ref()?;

        match scale {
            RangeQuestionScale::NumericRangeQuestionScale {
                min,
                max,
                deriv_ratio,
            } => Some(NumP(self.scale_range_prediction(
                prediction,
                *min,
                *max,
                *deriv_ratio,
            ))),
            RangeQuestionScale::DateRangeQuestionScale {
                min,
                max,
                deriv_ratio,
            } => {
                let date_format = "%Y-%m-%d";
                let min_ts = NaiveDate::parse_from_str(min, date_format)
                    .ok()?
                    .and_hms(0, 0, 0)
                    .timestamp() as f64;
                let max_ts = NaiveDate::parse_from_str(max, date_format)
                    .ok()?
                    .and_hms(0, 0, 0)
                    .timestamp() as f64;
                Some(DatP(NaiveDateTime::from_timestamp(
                    self.scale_range_prediction(prediction, min_ts, max_ts, *deriv_ratio) as i64,
                    0,
                )))
            }
        }
    }

    fn scale_range_prediction(&self, prediction: f64, min: f64, max: f64, deriv_ratio: f64) -> f64 {
        return if deriv_ratio == 1.0 {
            prediction * (max - min) + min
        } else {
            (max / min).powf(prediction) * min
        };
    }

    pub fn get_resolution(&self) -> Option<Prediction> {
        return if self.resolution? == -1.0 {
            Some(AmbP)
        } else if self.possibilities.question_type == "continuous" {
            Some(self.convert_range_prediction(self.resolution?)?)
        } else {
            Some(NumP(self.resolution?))
        };
    }
}

#[derive(PartialEq, Debug)]
pub enum Prediction {
    AmbP,
    NumP(f64),
    DatP(NaiveDateTime),
}

impl Prediction {
    pub fn get_if_numeric(&self) -> Option<f64> {
        match self {
            NumP(p) => Some(*p),
            _ => None,
        }
    }

    pub fn get_if_date(&self) -> Option<NaiveDateTime> {
        match self {
            DatP(p) => Some(*p),
            _ => None,
        }
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
#[serde(untagged)]
enum RangeQuestionScale {
    NumericRangeQuestionScale {
        min: f64,
        max: f64,
        deriv_ratio: f64,
    },
    DateRangeQuestionScale {
        min: String,
        max: String,
        deriv_ratio: f64,
    },
}
