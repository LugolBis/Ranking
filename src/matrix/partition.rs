use crate::maths::random;

struct GroupParition {
    nodes: Vec<u64>,
}

pub struct Partition {
    groups: Vec<GroupParition>,
}

impl Partition {
    pub fn new(node_count: u64, group_count: u64) -> Partition {
        let mut groups = Vec::new();
        for _ in 0..group_count {
            groups.push(GroupParition { nodes: Vec::new() });
        }
        for i in 0..node_count {
            let index = (random() * ((group_count - 1) as f64)).round() as usize;
            groups.get_mut(index).unwrap().nodes.push(i);
        }
        Partition { groups }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::matrix::partition::Partition;

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
}
