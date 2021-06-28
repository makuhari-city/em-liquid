use ndarray::{concatenate, s, Array, Array2, Axis};
use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;
use vote::{TopicInfo, Votes};

const ITERATION: u32 = 10_000;

#[derive(Debug)]
pub struct LiquidDemocracy {
    info: TopicInfo,
}

pub type LDResult = ((HashMap<Uuid, f64>), (HashMap<Uuid, f64>));

impl LiquidDemocracy {
    pub fn new(info: TopicInfo) -> Self {
        Self { info }
    }

    pub fn create_matrix(&self) -> Array2<f64> {
        let mut d_to_p: Array2<f64> = Array::zeros((
            self.info.delegates.len() + self.info.policies.len(),
            self.info.delegates.len(),
        ));

        for (x, (did, _name)) in self.info.delegates.iter().enumerate() {
            let mut votes = self.info.votes.get(did).unwrap().to_owned();

            for (to, v) in votes {
                let y = match self.info.delegates.iter().position(|(p, _)| p == &to) {
                    Some(p) => p,
                    None => {
                        self.info
                            .policies
                            .iter()
                            .position(|(p, _)| p == &to)
                            .unwrap()
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

    pub fn calculate(&self) -> LDResult {
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

        let poll_result: HashMap<Uuid, f64> = self
            .info
            .policies
            .iter()
            .map(|(pid, _)| pid)
            .cloned()
            .zip(results)
            .collect();

        let sum = sum.slice(s![..self.info.delegates.len(), ..self.info.delegates.len()]);
        let sum_row = sum.sum_axis(Axis(1));
        let influence = (sum_row / sum.diag()).to_vec();

        let influence: HashMap<Uuid, f64> = self
            .info
            .delegates
            .iter()
            .map(|(did, _)| did)
            .cloned()
            .zip(influence)
            .collect();

        (poll_result, influence)
    }
}

#[cfg(test)]
mod liquid_test {

    use super::*;

    fn breakfast() -> TopicInfo {
        let minori = (Uuid::new_v4(), "minori".to_string());
        let yasushi = (Uuid::new_v4(), "yasushi".to_string());
        let ray = (Uuid::new_v4(), "ray".to_string());

        let delegates: Vec<(Uuid, String)> =
            vec![minori.to_owned(), yasushi.to_owned(), ray.to_owned()]
                .iter()
                .cloned()
                .collect();

        let bread = (Uuid::new_v4(), "bread".to_string());
        let rice = (Uuid::new_v4(), "rice".to_string());

        let policies: Vec<(Uuid, String)> = vec![bread.to_owned(), rice.to_owned()]
            .iter()
            .cloned()
            .collect();

        let minori_votes = [
            (yasushi.0.to_owned(), 0.1),
            (ray.0.to_owned(), 0.1),
            (rice.0.to_owned(), 0.1),
            (bread.0.to_owned(), 0.7),
        ]
        .iter()
        .cloned()
        .collect();

        let yasushi_votes = [
            (minori.0.to_owned(), 0.2),
            (ray.0.to_owned(), 0.3),
            (rice.0.to_owned(), 0.5),
        ]
        .iter()
        .cloned()
        .collect();

        let ray_votes = [
            (minori.0.to_owned(), 0.4),
            (yasushi.0.to_owned(), 0.4),
            (bread.0.to_owned(), 0.2),
        ]
        .iter()
        .cloned()
        .collect();

        let votes: Votes = vec![
            (minori.0.to_owned(), minori_votes),
            (yasushi.0.to_owned(), yasushi_votes),
            (ray.0.to_owned(), ray_votes),
        ]
        .iter()
        .cloned()
        .collect();

        TopicInfo {
            title: "what to eat for breakfast".to_string(),
            id: Uuid::new_v4(),
            delegates,
            policies,
            votes,
        }
    }

    #[test]
    fn matrix_shape() {
        let breakfast = breakfast();
        let liq = LiquidDemocracy::new(breakfast.to_owned());
        let matrix = liq.create_matrix();

        assert_eq!(matrix.shape(), &[5, 5]);
    }

    #[test]
    fn simple() {
        let bf = breakfast();
        let liq = LiquidDemocracy::new(bf.to_owned());

        let (result, influence) = liq.calculate();

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
