use std::collections::HashMap;

use super::normalized;

pub use super::general::Edge;

pub type Node = super::general::Node<NodeType>;
pub type Graph = super::general::Graph<NodeType>;

#[derive(Debug)]
pub enum BuiltinOp {
    And,
    Not,
    Xor,
    Or,
}

#[derive(Debug)]
pub enum NodeType {
    Input { name: String, number: u32 },
    Output { name: String, number: u32 },
    Variable { name: String },
    BuiltinOp { op: BuiltinOp },
}

impl Graph {
    pub fn into_normalized(self) -> normalized::Graph {
        let nodes: Vec<_> = self
            .nodes
            .into_iter()
            .map(|n| {
                let id = n.id;
                let inner = match n.inner {
                    NodeType::Input { name, number } => {
                        normalized::NodeType::Input { name, number }
                    }
                    NodeType::Output { name, number } => {
                        normalized::NodeType::Output { name, number }
                    }
                    NodeType::Variable { name } => normalized::NodeType::Variable { name },
                    NodeType::BuiltinOp { op } => {
                        let tmp_op = match op {
                            BuiltinOp::And => normalized::BuiltinOp::And,
                            BuiltinOp::Not => normalized::BuiltinOp::Not,
                            BuiltinOp::Xor => normalized::BuiltinOp::Xor,
                            BuiltinOp::Or => normalized::BuiltinOp::Or,
                        };
                        normalized::NodeType::Operation { op: tmp_op }
                    }
                };

                normalized::Node::new(id, inner)
            })
            .collect();
        let edges: Vec<_> = self
            .edges
            .into_iter()
            .map(|e| normalized::Edge::new(e.src_id, e.src_port, e.dest_id, e.dest_port))
            .collect();

        let mut src_connections: HashMap<(u32, u32), Vec<(u32, u32)>> = HashMap::new();
        for edge in edges.iter() {
            let key = (edge.src_id, edge.src_port);
            let value = (edge.dest_id, edge.dest_port);

            match src_connections.get_mut(&key) {
                Some(prev) => {
                    prev.push(value);
                }
                None => {
                    src_connections.insert(key, vec![value]);
                }
            };
        }

        let mut graph = normalized::Graph::new(nodes, edges);

        loop {
            let multiple_targets_res = src_connections.iter().find(|(_, dest)| dest.len() > 1);
            let (src, targets) = match multiple_targets_res {
                Some((s, t)) => (*s, t.clone()),
                None => break,
            };

            // First remove all the previous Edges
            for target in targets.iter() {
                let tmp_e = normalized::Edge::new(src.0, src.1, target.0, target.1);
                graph.remove_edge(&tmp_e);
            }

            // Insert Splitter
            let n_id = graph.max_id() + 1;
            let splitter_node = normalized::Node::new(
                n_id,
                normalized::NodeType::Splitter {
                    port_count: targets.len() as u32,
                },
            );
            graph.add_node(splitter_node);

            let src_edge = normalized::Edge::new(src.0, src.1, n_id, 0);
            graph.add_edge(src_edge);

            for (index, target) in targets.into_iter().enumerate() {
                let n_edge = normalized::Edge::new(n_id, index as u32, target.0, target.1);
                graph.add_edge(n_edge);

                src_connections.insert((n_id, index as u32), vec![(target.0, target.1)]);
            }

            src_connections.insert(src, vec![(n_id, 0)]);
        }

        graph
    }
}
