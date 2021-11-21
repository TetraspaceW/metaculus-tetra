use std::fs::File;
use std::io::BufReader;
use metaculustetra::Question;

pub fn read_q_from_file(filename: &str) -> Question {
    let file = File::open(format!("tests/{}.json", filename)).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}