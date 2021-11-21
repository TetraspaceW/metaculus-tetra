mod utils;
use crate::utils::read_q_from_file;
use metaculustetra::index::{Index, WeightedQuestion};

#[test]
fn test_weighted_question() {
    let question = read_q_from_file("probability_example");

    let weighted = WeightedQuestion::create_from_binary(&question, 1.0).unwrap();

    assert_eq!(weighted.weight, 1.0);
    assert_eq!(weighted.zero, 0.0);
    assert_eq!(weighted.get_value(), 0.2);
}

#[test]
fn test_index() {
    let question_1 = read_q_from_file("probability_example");
    let question_2 = read_q_from_file("resolved_probability_example");

    let index = Index {
        questions: vec![
            WeightedQuestion::create_from_binary(&question_1, 1.0).unwrap(),
            WeightedQuestion::create_from_binary(&question_1, 0.5).unwrap(),
            WeightedQuestion::create_from_binary(&question_2, 2.0).unwrap(),
        ]
    };

    assert_eq!(index.get_index_value(), 2.3);
}