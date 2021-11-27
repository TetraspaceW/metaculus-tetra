//!
//! A module for creating indices. Indices are the weighted sums of multiple questions that relate
//! to a single topic, intended to give an overall view of how the topic is doing, as described
//! [here](https://www.metaculus.com/questions/935/platform-feature-suggestions/#comment-69686).
//!

use crate::date_utils::DateUtils;
use crate::RangeQuestionScale::{DateRangeQuestionScale, NumericRangeQuestionScale};
use crate::{AmbP, DatP, Metaculus, NumP, Question};
use chrono::NaiveDateTime;

///
/// An index, containing a list of weighted questions.
///
pub struct Index {
    /// List of questions to consider as part of the index.
    pub questions: Vec<WeightedQuestion>,
}

/// A question along with the weight to assign to it in an index.
pub struct WeightedQuestion {
    pub question: Question,
    pub weight: f64,
    ///
    /// The zero point for the question.
    ///
    /// For linear continuous questions, this is subtracted from the prediction before multiplying
    /// by the weight.
    ///
    /// For logarithmic continuous questions with `linearise_if_log == true`, the prediction is divided
    /// by this before multiplying by the weight.
    ///
    pub zero: f64,
    ///
    /// `true` to take the natural logarithm of the prediction when evaluating the index if it is
    /// on a logarithmic scale, `false` to leave it as-is.
    ///
    pub linearise_if_log: bool,
}

impl Index {
    ///
    /// Get the current value of the index by summing over the questions and multiplying by the
    /// weights.
    ///
    pub fn get_index_value(&self) -> f64 {
        self.get_index_value_before(NaiveDateTime::latest_prediction_date())
    }

    ///
    /// Get the value of the index as it was on the given `date` by summing over the questions and
    /// multiplying by the weights.
    ///
    pub fn get_index_value_before(&self, date: NaiveDateTime) -> f64 {
        self.questions
            .iter()
            .map(|q| q.get_value_before(date))
            .sum::<f64>()
    }
}

impl WeightedQuestion {
    ///
    /// Makes a new `WeightedQuestion` from a [Question], with a given weight and a zero point of
    /// `0.0`, if the question is a binary question.
    ///
    pub fn create_from_binary(question: &Question, weight: f64) -> Option<WeightedQuestion> {
        if question.is_binary() {
            Some(WeightedQuestion {
                question: question.clone(),
                weight,
                zero: 0.0,
                linearise_if_log: false,
            })
        } else {
            None
        }
    }

    ///
    /// Makes a new `WeightedQuestion` from a [Question], with a given weight and a zero point equal
    /// to the smallest input value on Metaculus, if the question is a continuous numerical question.
    ///
    pub fn create_from_range(question: &Question, weight: f64) -> Option<WeightedQuestion> {
        Some(WeightedQuestion {
            question: question.clone(),
            weight,
            zero: match question.possibilities.scale.as_ref()? {
                NumericRangeQuestionScale { min, .. } => *min,
                _ => None?,
            },
            linearise_if_log: true,
        })
    }

    ///
    /// Makes a new `WeightedQuestion` from a [Question], with a given weight and a zero point equal
    /// to the smallest input value on Metaculus, if the question is a continuous date question.
    ///
    /// For dates, the base unit for the value of the index is the number of seconds from the zero
    /// point.
    ///
    pub fn create_from_date(question: &Question, weight: f64) -> Option<WeightedQuestion> {
        Some(WeightedQuestion {
            question: question.clone(),
            weight,
            zero: match question.possibilities.scale.as_ref()? {
                DateRangeQuestionScale { min, .. } => NaiveDateTime::date_to_timestamp(min)?,
                _ => None?,
            },
            linearise_if_log: true,
        })
    }

    ///
    /// Get the value that the current prediction on this weighted question alone contributes to its
    /// index.
    ///
    pub fn get_value(&self) -> f64 {
        self.get_value_before(NaiveDateTime::latest_prediction_date())
    }

    ///
    /// Get the value that the prediction on this weighted question at the given `date` contributes
    /// to its index.
    ///
    pub fn get_value_before(&self, date: NaiveDateTime) -> f64 {
        match self.question.get_best_prediction_before(date) {
            None => 0.0,
            Some(p) => match p {
                AmbP => 0.0,
                NumP(p) => {
                    if self.linearise_if_log && self.question.is_logarithmic() {
                        (p / self.zero).ln() * self.weight
                    } else {
                        (p - self.zero) * self.weight
                    }
                }
                DatP(p) => {
                    if self.linearise_if_log && self.question.is_logarithmic() {
                        (p.timestamp() as f64 / self.zero).ln() * self.weight
                    } else {
                        (p.timestamp() as f64 - self.zero) * self.weight
                    }
                }
            },
        }
    }
}

pub trait MetaculusIndexCreator {
    fn create_index_from_questions(&self, ids: Vec<&str>, weights: Vec<f64>) -> Index;
}

impl MetaculusIndexCreator for Metaculus<'_> {
    ///
    /// Creates an [Index] from a list of question `ids`, each of which have the given weight,
    /// ignoring questions which cannot be received or parsed successfully.
    ///
    fn create_index_from_questions(&self, ids: Vec<&str>, weights: Vec<f64>) -> Index {
        let questions = self.get_questions(ids).iter()
            .zip(weights)
            .map(|pair| {
                let q = (*pair.0).as_ref()?;
                let weight = pair.1;
                Some(
                    WeightedQuestion::create_from_binary(&q, weight).unwrap_or(
                        WeightedQuestion::create_from_range(&q, weight)
                            .unwrap_or(WeightedQuestion::create_from_date(&q, weight)?),
                    ),
                )
            })
            .filter_map(|wq| wq )
            .collect();

        Index { questions }
    }
}
