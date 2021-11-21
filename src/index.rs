use crate::RangeQuestionScale::{DateRangeQuestionScale, NumericRangeQuestionScale};
use crate::{AmbP, DatP, NumP, Question};

pub struct Index {
    pub questions: Vec<WeightedQuestion>,
}

pub struct WeightedQuestion {
    pub question: Question,
    pub weight: f64,
    pub zero: f64,
    pub linearise_if_log: bool,
}

impl Index {
    pub fn get_index_value(&self) -> f64 {
        self.questions.iter().map(|q| q.get_value()).sum::<f64>()
    }
}

impl WeightedQuestion {
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

    pub fn create_from_date(question: &Question, weight: f64) -> Option<WeightedQuestion> {
        Some(WeightedQuestion {
            question: question.clone(),
            weight,
            zero: match question.possibilities.scale.as_ref()? {
                DateRangeQuestionScale { min, .. } => Question::date_to_timestamp(min)?,
                _ => None?,
            },
            linearise_if_log: true,
        })
    }

    pub fn get_value(&self) -> f64 {
        match self.question.get_best_prediction() {
            None => 0.0,
            Some(p) => match p {
                AmbP => 0.0,
                NumP(p) => {
                    if self.linearise_if_log && self.question.is_logarithmic() {
                        ((p / self.zero).ln() - 1.0) * self.weight
                    } else {
                        (p - self.zero) * self.weight
                    }
                }
                DatP(_) => {
                    panic!("Only numeric predictions are currently supported in indices.")
                }
            },
        }
    }
}
