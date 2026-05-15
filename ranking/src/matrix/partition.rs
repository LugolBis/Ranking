use std::collections::HashMap;

use crate::maths::RngSeq;

const PARTITION_SEED: u64 = 42;

#[derive(Clone, Debug)]
pub struct GroupPartition {
    nodes: HashMap<u64, usize>,
}

impl GroupPartition {
    pub fn new(nodes: HashMap<u64, usize>) -> GroupPartition {
        GroupPartition { nodes }
    }

    pub fn contains(&self, value: u64) -> bool {
        self.nodes.contains_key(&value)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn index(&self, value: u64) -> usize {
        self.nodes[&value]
    }
}

#[derive(Debug)]
pub struct Partition {
    groups: Vec<GroupPartition>,
}

impl Partition {
    pub fn new(node_count: u64, group_count: u64) -> Partition {
        let mut groups = Vec::new();
        for _ in 0..group_count {
            groups.push(GroupPartition {
                nodes: HashMap::new(),
            });
        }
        let mut rndseq = RngSeq::from(PARTITION_SEED);
        for i in 0..node_count {
            let index = (rndseq.next() * ((group_count - 1) as f64)).round() as usize;
            let value = groups[index].nodes.len();
            groups[index].nodes.insert(i, value);
        }
        Partition { groups }
    }

    pub fn fusion_stationary_distributions(
        &self,
        stationary_distributions: &Vec<Vec<f64>>,
    ) -> Vec<f64> {
        let mut node_count = 0;
        for group in self.groups.iter() {
            node_count += group.nodes.len();
        }
        let mut fusioned_stationary_distribution = vec![0_f64; node_count];
        for (group_index, group) in self.groups.iter().enumerate() {
            for column in group.nodes.iter() {
                fusioned_stationary_distribution[*(column.0) as usize] =
                    stationary_distributions[group_index][*(column.1)];
            }
        }
        fusioned_stationary_distribution
    }

    pub fn divide_stationary_distribution(
        &self,
        stationary_distribution: &Vec<f64>,
    ) -> Vec<Vec<f64>> {
        let mut stationary_distributions = Vec::new();
        for group in self.groups.iter() {
            let mut new_stationary_distribution = vec![0.0; group.len()];
            for (column_index, position) in group.nodes.iter() {
                new_stationary_distribution[*position] =
                    stationary_distribution[TryInto::<usize>::try_into(*column_index).unwrap()];
            }
            stationary_distributions.push(new_stationary_distribution);
        }
        stationary_distributions
    }

    pub fn groups(&self) -> &Vec<GroupPartition> {
        &self.groups
    }

    pub fn group_containing(&self, value: u64) -> Option<usize> {
        for (i, group) in self.groups.iter().enumerate() {
            if group.contains(value) {
                return Some(i);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::matrix::partition::{GroupPartition, Partition};

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
        let distributions: Vec<Vec<f64>> = vec![vec![0.1, 0.3], vec![0.2, 0.4]];
        let mut first_group = HashMap::new();
        first_group.insert(0, 0);
        first_group.insert(2, 1);
        let mut second_group = HashMap::new();
        second_group.insert(1, 0);
        second_group.insert(3, 1);
        let partition = Partition {
            groups: vec![
                GroupPartition { nodes: first_group },
                GroupPartition {
                    nodes: second_group,
                },
            ],
        };
        assert_eq!(
            partition.fusion_stationary_distributions(&distributions),
            vec![0.1, 0.2, 0.3, 0.4]
        );
    }

    #[test]
    fn divide_distributions() {
        let partition = Partition::new(4, 2);
        let distribution: Vec<f64> = vec![0.1, 0.3, 0.2, 0.4];
        assert_eq!(
            distribution,
            partition.fusion_stationary_distributions(
                &partition.divide_stationary_distribution(&distribution)
            )
        )
    }
}
