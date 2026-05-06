use std::collections::HashSet;

use crate::maths::random;

pub struct GroupParition {
    nodes: HashSet<u64>,
}

impl GroupParition {
    pub fn contains(&self, value: u64) -> bool {
        self.nodes.contains(&value)
    }
}

pub struct Partition {
    groups: Vec<GroupParition>,
}

impl Partition {
    pub fn new(node_count: u64, group_count: u64) -> Partition {
        let mut groups = Vec::new();
        for _ in 0..group_count {
            groups.push(GroupParition {
                nodes: HashSet::new(),
            });
        }
        for i in 0..node_count {
            let index = (random() * ((group_count - 1) as f64)).round() as usize;
            groups.get_mut(index).unwrap().nodes.insert(i);
        }
        Partition { groups }
    }

    pub fn fusion_stationary_distributions(
        &self,
        stationary_distributions: Vec<Vec<f64>>,
    ) -> Vec<f64> {
        let mut node_count = 0;
        for group in self.groups.iter() {
            node_count += group.nodes.len();
        }
        let mut fusioned_stationary_distribution = vec![0_f64; node_count];
        for group_index in 0..self.groups.len() {
            for column in self.groups[group_index].nodes.iter() {
                fusioned_stationary_distribution[*column as usize] =
                    stationary_distributions[group_index][*column as usize];
            }
        }
        fusioned_stationary_distribution
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::matrix::partition::{GroupParition, Partition};

    #[test]
    fn create_partition() {
        let partition = Partition::new(100, 4);
        assert_eq!(partition.groups.len(), 4);
        let mut values = HashSet::new();
        for group in partition.groups {
            for value in group.nodes {
                if values.contains(&value) {
                    panic!();
                }
                values.insert(value);
            }
        }
        assert_eq!(values.len(), 100);
    }

    #[test]
    fn fusion_distributions() {
        let distributions: Vec<Vec<f64>> = vec![vec![0.2, 0.0, 0.2, 0.0], vec![0.0, 0.2, 0.0, 0.2]];
        let mut first_group = HashSet::new();
        first_group.insert(0);
        first_group.insert(2);
        let mut second_group = HashSet::new();
        second_group.insert(1);
        second_group.insert(3);
        let partition = Partition {
            groups: vec![
                GroupParition { nodes: first_group },
                GroupParition {
                    nodes: second_group,
                },
            ],
        };
        assert_eq!(
            partition.fusion_stationary_distributions(distributions),
            vec![0.2, 0.2, 0.2, 0.2]
        );
    }
}
