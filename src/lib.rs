//! A library for interacting with Metaculus.
//!
//! [Repository](https://github.com/TetraspaceW/metaculus-tetra)
//!

pub mod index;

use chrono::{NaiveDate, NaiveDateTime};
use log::info;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use crate::Prediction::{AmbP, DatP, NumP};

///
/// An API client for retrieving Metaculus question data . Contains the domain (e.g. `www`,
/// `pandemic`, `ai`) of the Metaculus instance and the blocking reqwest client.
///
/// # Example
///
/// ``` rust
/// use metaculustetra::Metaculus;
/// use reqwest::blocking::Client;
/// // Standard Metaculus client, accesses https://www.metaculus.com
/// let m = Metaculus::standard();
/// // Pandemic Metaculus client, accesses https://pandemic.metaculus.com
/// let mp = Metaculus { domain: "pandemic", client: Client::new() };
/// ```
pub struct Metaculus<'a> {
    pub domain: &'a str,
    pub client: Client,
}

impl Metaculus<'_> {
    pub fn standard() -> Metaculus<'static> {
        Metaculus {
            domain: "www",
            client: Client::new(),
        }
    }

    ///
    /// Returns the numeric prediction for the question `id` if it is a numerical question, and
    /// `None` otherwise.
    ///
    pub fn get_numeric_prediction_for(&self, id: &str) -> Option<f64> {
        self.get_prediction_for(id)?.get_if_numeric()
    }

    ///
    /// Returns the date prediction for the question `id` if it is a numerical question, and `None`
    /// otherwise.
    ///
    pub fn get_date_prediction_for(&self, id: &str) -> Option<NaiveDateTime> {
        self.get_prediction_for(id)?.get_if_date()
    }

    ///
    /// Returns the best prediction available (prioritising the actual resolution, then the Metaculus
    /// prediction, then the community prediction) far the question with id `id` as a [Prediction],
    /// if the question exists and has any predictions.
    ///
    pub fn get_prediction_for(&self, id: &str) -> Option<Prediction> {
        self.get_question(id)?.get_best_prediction()
    }

    ///
    /// Returns the question with id `id` as a [Question] if it exists.
    ///
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

///
/// Data on a single Metaculus question.
///
#[derive(Serialize, Deserialize, Clone)]
pub struct Question {
    pub title_short: String,
    prediction_timeseries: Option<Vec<PredictionTimeseriesPoint>>,
    metaculus_prediction: Option<MetaculusPredictionTimeseriesPoint>,
    resolution: Option<f64>,
    possibilities: QuestionPossibilities,
}

impl Question {
    ///
    /// Returns the best prediction available (prioritising the actual resolution, then the
    /// Metaculus prediction, then the community prediction) for the question as a [Prediction], if
    /// the question has any predictions.
    ///
    /// # Example
    /// ```rust
    /// use metaculustetra::Prediction::NumP;
    /// use std::fs::File;
    /// use std::io::BufReader;
    /// use metaculustetra::Question;
    ///
    /// // Load .json response for a file
    /// let file = File::open("tests/resolved_probability_example.json").unwrap();
    /// let reader = BufReader::new(file);
    /// let question: Question = serde_json::from_reader(reader).unwrap();
    /// // The question resolved at yes, which is 1.0 (100%).
    /// assert_eq!(question.get_resolution(), Some(NumP(1.0)));
    /// // The community median prediction was 99%.
    /// assert_eq!(question.get_community_prediction(), Some(NumP(0.99)));
    /// // The Metaculus prediction was 98.7% with a few more decimal places.
    /// assert_eq!(question.get_metaculus_prediction(), Some(NumP(0.986684322309624)));
    /// // The best available prediction is the resolution, 100%.
    /// assert_eq!(question.get_best_prediction(), Some(NumP(1.0)));
    /// ```
    ///
    pub fn get_best_prediction(&self) -> Option<Prediction> {
        return match self.get_resolution() {
            None => match self.get_metaculus_prediction() {
                None => self.get_community_prediction(),
                mp => mp,
            },
            r => r,
        };
    }

    ///
    /// Returns the community median prediction, if it exists.
    ///
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

    ///
    /// Returns the Metaculus prediction, if it exists and is available.
    ///
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

    ///
    /// Returns the question resolution, if it exists. This will be a [Prediction::AmbP]
    ///
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

///
/// An aggregated overall prediction on a Metaculus question.
///
#[derive(PartialEq, Debug)]
pub enum Prediction {
    /// Represents an Ambiguous resolution.
    AmbP,
    /// Represents a numeric prediction, either a probability (0.0 - 1.0) or continuous.
    NumP(f64),
    /// Represents a date prediction.
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

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum PredictionTimeseriesPoint {
    NumericPTP {
        community_prediction: f64,
    },
    RangePTP {
        community_prediction: RangeCommunityPrediction,
    },
}

#[derive(Serialize, Deserialize, Clone)]
struct RangeCommunityPrediction {
    q2: f64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum MetaculusPredictionTimeseriesPoint {
    NumericMPTP { full: f64 },
    RangeMPTP { full: RangeMetaculusPrediction },
}

#[derive(Serialize, Deserialize, Clone)]
struct RangeMetaculusPrediction {
    q2: f64,
}

#[derive(Serialize, Deserialize, Clone)]
struct QuestionPossibilities {
    #[serde(rename = "type")]
    question_type: String,
    scale: Option<RangeQuestionScale>,
    format: Option<String>
}

#[derive(Serialize, Deserialize, Clone)]
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
