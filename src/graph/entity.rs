use std::collections::HashMap;

use super::builtin;

pub use super::general::Edge;

pub type Node = super::general::Node<NodeType>;
pub type Graph = super::general::Graph<NodeType>;

#[derive(Debug, Clone)]
pub enum BuiltinOp {
    Not,
    And,
    Xor,
    Or,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Input { name: String, number: u32 },
    Output { name: String, number: u32 },
    Variable { name: String },
    BuiltinOp { op: BuiltinOp },
    EntityOp { name: String },
}

impl Graph {
    fn inputs(&self) -> Vec<u32> {
        let mut result = Vec::new();

        for n in self.nodes.iter() {
            match &n.inner {
                NodeType::Input { .. } => {}
                _ => continue,
            };

            let edges_to = self.edges_to_node(n.id);

            if !edges_to.is_empty() {
                continue;
            }

            result.push(n.id);
        }

        result
    }

    fn outputs(&self) -> Vec<u32> {
        let mut result = Vec::new();

        for n in self.nodes.iter() {
            match &n.inner {
                NodeType::Output { .. } => {}
                _ => continue,
            };

            let edges_from = self.edges_from_node(n.id);

            if !edges_from.is_empty() {
                continue;
            }

            result.push(n.id);
        }

        result
    }

    pub fn into_builtin(self, entities: &HashMap<String, Graph>) -> super::builtin::Graph {
        let mut nodes = self.nodes;
        let mut edges = self.edges;

        loop {
            let found_result = nodes
                .iter()
                .enumerate()
                .find(|(_, n)| matches!(n.inner, NodeType::EntityOp { .. }));

            let replace_index = match found_result {
                Some((i, _)) => i,
                None => break,
            };

            let to_replace = nodes.remove(replace_index);

            let name = match &to_replace.inner {
                NodeType::EntityOp { name } => name,
                _ => unreachable!(),
            };

            let mut replacement_graph = entities.get(name).unwrap().clone();

            let max_id = nodes.iter().map(|n| n.id).max().unwrap() + 1;
            replacement_graph.offset_ids(max_id);

            let input_ids = replacement_graph.inputs();
            let output_ids = replacement_graph.outputs();

            nodes.extend(replacement_graph.nodes);
            edges.extend(replacement_graph.edges);

            edges
                .iter_mut()
                .filter(|e| e.dest_id == to_replace.id)
                .for_each(|e| {
                    let n_id = input_ids.get(e.dest_port as usize).unwrap();

                    e.dest_id = *n_id;
                    e.dest_port = 0;
                });

            edges
                .iter_mut()
                .filter(|e| e.src_id == to_replace.id)
                .for_each(|e| {
                    let n_id = output_ids.get(e.src_port as usize).unwrap();

                    e.src_id = *n_id;
                    e.src_port = 0;
                });
        }

        let n_nodes: Vec<_> = nodes
            .into_iter()
            .map(|n| {
                let n_type = match n.inner {
                    NodeType::Input { name, number } => builtin::NodeType::Input { name, number },
                    NodeType::Output { name, number } => builtin::NodeType::Output { name, number },
                    NodeType::Variable { name } => builtin::NodeType::Variable { name },
                    NodeType::EntityOp { .. } => panic!("Unexpected Entity Op"),
                    NodeType::BuiltinOp { op } => {
                        let n_op = match op {
                            BuiltinOp::And => builtin::BuiltinOp::And,
                            BuiltinOp::Not => builtin::BuiltinOp::Not,
                            BuiltinOp::Xor => builtin::BuiltinOp::Xor,
                            BuiltinOp::Or => builtin::BuiltinOp::Or,
                        };
                        builtin::NodeType::BuiltinOp { op: n_op }
                    }
                };
                builtin::Node::new(n.id, n_type)
            })
            .collect();

        builtin::Graph::new(n_nodes, edges)
    }
}
