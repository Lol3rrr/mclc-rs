#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub src_id: u32,
    pub src_port: u32,
    pub dest_id: u32,
    pub dest_port: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node<T> {
    pub id: u32,
    pub inner: T,
}

#[derive(Debug, Clone)]
pub struct Graph<T> {
    pub nodes: Vec<Node<T>>,
    pub edges: Vec<Edge>,
}

impl<T> Node<T> {
    pub fn new(id: u32, inner: T) -> Self {
        Self { id, inner }
    }
}

impl Edge {
    pub fn new(src_id: u32, src_port: u32, dest_id: u32, dest_port: u32) -> Self {
        Self {
            src_id,
            src_port,
            dest_id,
            dest_port,
        }
    }
}

impl<T> Graph<T> {
    /// Creates a new Graph from the given Set of Nodes and Edges
    pub fn new(nodes: Vec<Node<T>>, edges: Vec<Edge>) -> Self {
        Self { nodes, edges }
    }

    /// Returns all the Edges that are directed at the Node with the given ID
    pub fn edges_to_node(&self, target_id: u32) -> Vec<Edge> {
        self.edges
            .iter()
            .filter(|e| e.dest_id == target_id)
            .cloned()
            .collect()
    }

    pub fn edges_from_node(&self, src_id: u32) -> Vec<Edge> {
        self.edges
            .iter()
            .filter(|e| e.src_id == src_id)
            .cloned()
            .collect()
    }

    /// Shifts all the IDs in the Graph by the given Offset
    pub fn offset_ids(&mut self, offset: u32) {
        for e in self.edges.iter_mut() {
            e.src_id += offset;
            e.dest_id += offset;
        }

        for n in self.nodes.iter_mut() {
            n.id += offset;
        }
    }

    pub fn get_node(&self, id: u32) -> Option<&Node<T>> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn add_node(&mut self, node: Node<T>) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    pub fn remove_node(&mut self, node: u32) {
        let index = match self.nodes.iter().enumerate().find(|(_, n)| n.id == node) {
            Some((i, _)) => i,
            None => return,
        };

        self.nodes.remove(index);
    }

    pub fn remove_edge(&mut self, edge: &Edge) {
        let index = match self.edges.iter().enumerate().find(|(_, e)| e == &edge) {
            Some((i, _)) => i,
            None => return,
        };

        self.edges.remove(index);
    }

    pub fn max_id(&self) -> u32 {
        self.nodes.iter().map(|n| n.id).max().unwrap()
    }
}

impl<T> Graph<T>
where
    T: Clone + PartialEq,
{
    pub fn nodes_with_predecessors(&self) -> Vec<(Node<T>, Vec<Node<T>>)> {
        let mut result = Vec::new();
        for target_node in self.nodes.iter() {
            let node = target_node.clone();

            let mut preds = Vec::new();

            for src in self
                .edges_to_node(target_node.id)
                .into_iter()
                .filter_map(|e| self.get_node(e.src_id).cloned())
            {
                if !preds.contains(&src) {
                    preds.push(src);
                }
            }

            result.push((node, preds));
        }

        result
    }
}
