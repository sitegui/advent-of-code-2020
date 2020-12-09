use crate::data::Data;

#[derive(Debug, Default)]
struct GroupForm {
    num_people: i32,
    yeses: [i32; 26],
}

#[derive(Debug, Default)]
struct QuestionsTally {
    any_yes: i64,
    all_yes: i64,
}

pub fn solve() -> (i64, i64) {
    let data = Data::read(6);

    let mut total_tally = QuestionsTally::default();
    for paragraph in data.paragraphs() {
        let mut form = GroupForm::default();
        for line in paragraph {
            form.begin_person();
            for &question in line {
                form.answer_yes(question);
            }
        }
        let tally = form.tally_answers();
        total_tally.any_yes += tally.any_yes;
        total_tally.all_yes += tally.all_yes;
    }

    (total_tally.any_yes, total_tally.all_yes)
}

impl GroupForm {
    fn begin_person(&mut self) {
        self.num_people += 1;
    }

    fn answer_yes(&mut self, question: u8) {
        self.yeses[(question - b'a') as usize] += 1;
    }

    fn tally_answers(&self) -> QuestionsTally {
        self.yeses
            .iter()
            .fold(QuestionsTally::default(), |mut tally, &num_yeses| {
                if num_yeses > 0 {
                    tally.any_yes += 1;
                }
                if num_yeses == self.num_people {
                    tally.all_yes += 1;
                }
                tally
            })
    }
}
