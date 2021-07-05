use ndarray::{concatenate, s, Array, Array2, Axis};
use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;
use vote::{TopicData, VoteData};

const ITERATION: u32 = 10_000;

#[derive(Debug)]
pub struct LiquidDemocracy {
    info: VoteData,
}

pub type LDResult = ((HashMap<Uuid, f64>), (HashMap<Uuid, f64>));

impl LiquidDemocracy {
    pub fn new(info: VoteData) -> Self {
        Self { info }
    }

    pub fn create_matrix(&self) -> Array2<f64> {
        let num_delegates = self.info.delegates.len();
        let num_policies = self.info.policies.len();

        let mut d_to_p: Array2<f64> = Array::zeros((num_delegates + num_policies, num_delegates));

        for (x, did) in self.info.delegates.iter().enumerate() {
            println!("{}", did);
            let votes = self.info.votes.get(did).unwrap().to_owned();

            for (to, v) in votes {
                let y = match self.info.delegates.iter().position(|p| p == &to) {
                    Some(p) => p,
                    None => {
                        self.info.policies.iter().position(|p| p == &to).unwrap()
                            + self.info.delegates.len()
                    }
                };

                d_to_p[[y, x]] = v;
            }
        }

        let p_to_d: Array2<f64> =
            Array::zeros((self.info.policies.len(), self.info.delegates.len()));
        let p_to_p: Array2<f64> = Array::eye(self.info.policies.len());

        let left: Array2<f64> = concatenate![Axis(1), p_to_d, p_to_p];
        concatenate![Axis(1), d_to_p, left.t()]
    }

    pub async fn calculate(&self) -> LDResult {
        let matrix = self.create_matrix();

        let edge = matrix.shape()[0];
        let mut a = Array::eye(edge);
        let mut sum = Array::eye(edge);

        for _ in 0..ITERATION {
            a = a.dot(&matrix);
            sum += &a;
        }

        let a = a.slice(s![.., 0..self.info.delegates.len()]);
        let results = a
            .sum_axis(Axis(1))
            .slice(s![self.info.delegates.len()..])
            .to_vec();

        let poll_result: HashMap<Uuid, f64> =
            self.info.policies.iter().cloned().zip(results).collect();

        let sum = sum.slice(s![..self.info.delegates.len(), ..self.info.delegates.len()]);
        let sum_row = sum.sum_axis(Axis(1));
        let influence = (sum_row / sum.diag()).to_vec();

        let influence: HashMap<Uuid, f64> =
            self.info.delegates.iter().cloned().zip(influence).collect();

        (poll_result, influence)
    }
}

#[cfg(test)]
mod liquid_test {

    use super::*;

    fn breakfast() -> TopicData {
        let mut topic = TopicData::new("breakfast", "what to eat in the morning");

        let minori = topic.add_new_delegate("minori").unwrap();
        let yasushi = topic.add_new_delegate("yasushi").unwrap();
        let ray = topic.add_new_delegate("ray").unwrap();

        let bread = topic.add_new_policy("bread").unwrap();
        let rice = topic.add_new_policy("rice").unwrap();

        topic.cast_vote_to(&minori, &yasushi, 0.1);
        topic.cast_vote_to(&minori, &ray, 0.1);
        topic.cast_vote_to(&minori, &rice, 0.1);
        topic.cast_vote_to(&minori, &bread, 0.7);

        topic.cast_vote_to(&yasushi, &minori, 0.2);
        topic.cast_vote_to(&yasushi, &ray, 0.3);
        topic.cast_vote_to(&yasushi, &rice, 0.5);

        topic.cast_vote_to(&ray, &minori, 0.4);
        topic.cast_vote_to(&ray, &yasushi, 0.4);
        topic.cast_vote_to(&ray, &bread, 0.2);

        topic
    }

    #[test]
    fn matrix_shape() {
        let breakfast = breakfast();

        let info: VoteData = breakfast.into();
        let liq = LiquidDemocracy::new(info);
        let matrix = liq.create_matrix();

        assert_eq!(matrix.shape(), &[5, 5]);
    }

    #[actix_rt::test]
    async fn simple() {
        let bf = breakfast();
        let info: VoteData = bf.to_owned().into();
        let liq = LiquidDemocracy::new(info);

        let (result, influence) = liq.calculate().await;

        let bread = bf
            .get_id_by_title("bread")
            .and_then(|uid| result.get(&uid))
            .unwrap();

        let rice = bf
            .get_id_by_title("rice")
            .and_then(|uid| result.get(&uid))
            .unwrap();

        assert!(rice < bread);

        let minori = bf
            .get_id_by_name("minori")
            .and_then(|uid| influence.get(&uid))
            .unwrap();

        let yasushi = bf
            .get_id_by_name("yasushi")
            .and_then(|uid| influence.get(&uid))
            .unwrap();

        assert!(minori > yasushi);
    }
}
