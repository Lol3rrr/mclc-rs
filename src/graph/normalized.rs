pub use super::general::Edge;

pub type Graph = super::general::Graph<NodeType>;
pub type Node = super::general::Node<NodeType>;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Input { name: String, number: u32 },
    Output { name: String, number: u32 },
    Variable { name: String },
    Splitter { port_count: u32 },
    Operation { op: BuiltinOp },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinOp {
    Xor,
    And,
    Or,
    Not,
}

impl Graph {
    pub fn optimize(&mut self) {
        let removeable: Vec<_> = self
            .nodes
            .iter()
            .filter(|n| {
                matches!(
                    &n.inner,
                    NodeType::Input { .. } | NodeType::Output { .. } | NodeType::Variable { .. }
                )
            })
            .filter(|n| {
                let inputs = self.edges_to_node(n.id);
                let outputs = self.edges_from_node(n.id);
                inputs.len() == 1 && outputs.len() == 1
            })
            .map(|n| n.id)
            .collect();

        for r_id in removeable {
            let mut inputs = self.edges_to_node(r_id);
            let mut outputs = self.edges_from_node(r_id);

            let input = inputs.remove(0);
            let output = outputs.remove(0);

            let n_edge = Edge::new(
                input.src_id,
                input.src_port,
                output.dest_id,
                output.dest_port,
            );

            self.remove_node(r_id);
            self.remove_edge(&input);
            self.remove_edge(&output);

            self.add_edge(n_edge);
        }
    }
}
