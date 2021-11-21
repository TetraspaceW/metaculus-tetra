use crate::{DatP, NumP, AmbP, Question};
use crate::RangeQuestionScale::NumericRangeQuestionScale;

pub struct Index {
    pub questions: Vec<WeightedQuestion>
}

pub struct WeightedQuestion {
    pub question: Question,
    pub weight: f64,
    pub zero: f64
}

impl Index {
    pub fn get_index_value(&self) -> f64 {
        self.questions.iter().map(|q| {
            q.get_value()
        }).sum::<f64>()
    }
}

impl WeightedQuestion {
    pub fn create_from_binary(question: &Question, weight: f64) -> Option<WeightedQuestion> {
        Some(WeightedQuestion {
            question: question.clone(),
            weight,
            zero: 0.0
        })
    }

    pub fn create_from_range(question: &Question, weight: f64) -> Option<WeightedQuestion> {
        Some(WeightedQuestion {
            question: question.clone(),
            weight,
            zero: match question.possibilities.scale.as_ref()? {
                NumericRangeQuestionScale { min, .. } => { *min }
                _ => { return None }
            }
        })
    }

    pub fn get_value(&self) -> f64 {
        match self.question.get_best_prediction() {
            None => { 0.0 }
            Some(p) => match p {
                AmbP => { 0.0 }
                NumP(p) => {
                    (p - self.zero) * self.weight
                }
                DatP(_) => {
                    panic!("Only numeric predictions are currently supported in indices.")
                }
            }
        }
    }
}