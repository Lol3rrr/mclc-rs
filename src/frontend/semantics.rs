use std::collections::HashMap;

use crate::graph;

use super::{syntax, tokens::Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Type_ {
    Bit,
}

#[derive(Debug, Clone)]
pub struct Port {
    name: String,
    ty: Type_,
}

#[derive(Debug, Clone)]
struct EntityHeader {
    name: String,
    in_ports: Vec<Port>,
    out_ports: Vec<Port>,
}

#[derive(Debug, Clone)]
pub struct Variable {
    name: String,
    ty: Type_,
}

#[derive(Debug)]
pub enum BuiltinOp {
    Not,
    And,
    Xor,
    Or,
}

#[derive(Debug)]
pub enum Operand {
    Variable(Variable),
    Port(Port),
}

#[derive(Debug)]
pub enum BehaviourValue {
    BuiltinOp {
        op: BuiltinOp,
        arguments: Vec<Operand>,
    },
    EntityOp {
        op: String,
        arguments: Vec<Operand>,
        port_count: u32,
    },
    Variables {
        vars: Vec<Variable>,
    },
}

#[derive(Debug)]
pub enum Behaviour {
    VarAssign {
        targets: Vec<Variable>,
        value: BehaviourValue,
    },
    PortAssign {
        targets: Vec<Port>,
        value: BehaviourValue,
    },
}

#[derive(Debug)]
pub struct Entity {
    pub name: String,
    in_ports: Vec<Port>,
    out_ports: Vec<Port>,
    behaviour: Vec<Behaviour>,
}

pub fn parse(raw_entities: Vec<syntax::Entity>) -> Vec<Entity> {
    let headers: HashMap<String, EntityHeader> = raw_entities
        .iter()
        .map(parse_entity_header)
        .map(|h| (h.name.clone(), h))
        .collect();

    raw_entities
        .into_iter()
        .map(|e| parse_entity(e, &headers))
        .collect()
}

fn parse_entity_header(raw_entity: &syntax::Entity) -> EntityHeader {
    let name = raw_entity.name.content().to_string();

    let in_ports = parse_ports(&raw_entity.in_ports);
    let out_ports = parse_ports(&raw_entity.out_ports);

    EntityHeader {
        name,
        in_ports,
        out_ports,
    }
}

fn parse_ports(raw: &[(Token, Token)]) -> Vec<Port> {
    raw.iter()
        .map(|(n, ty)| {
            let name = n.1.content().to_string();
            let ty = match ty.1.content() {
                "bit" => Type_::Bit,
                other => panic!("Unknown Type: {:?}", other),
            };

            Port { name, ty }
        })
        .collect()
}

fn parse_value(
    value: syntax::BehaviourValue,
    current_header: &EntityHeader,
    vars: &HashMap<String, Variable>,
    headers: &HashMap<String, EntityHeader>,
) -> (BehaviourValue, Vec<Type_>) {
    match value {
        syntax::BehaviourValue::Operation { name, arguments } => {
            let op_name = name.1.content();

            let arguments: Vec<Operand> = arguments
                .into_iter()
                .map(|a| {
                    let name = a.1.content();

                    if let Some(port) = current_header.in_ports.iter().find(|p| p.name == name) {
                        return Operand::Port(port.clone());
                    }
                    if let Some(var) = vars.get(name) {
                        return Operand::Variable(var.clone());
                    }

                    dbg!(&a);
                    todo!("")
                })
                .collect();

            match op_name {
                "and" => {
                    assert!(arguments.len() == 2);
                    for a in arguments.iter() {
                        assert!(a.ty() == &Type_::Bit);
                    }

                    (
                        BehaviourValue::BuiltinOp {
                            op: BuiltinOp::And,
                            arguments,
                        },
                        vec![Type_::Bit],
                    )
                }
                "not" => {
                    assert!(arguments.len() == 1);
                    for a in arguments.iter() {
                        assert!(a.ty() == &Type_::Bit);
                    }

                    (
                        BehaviourValue::BuiltinOp {
                            op: BuiltinOp::Not,
                            arguments,
                        },
                        vec![Type_::Bit],
                    )
                }
                "xor" => {
                    assert!(arguments.len() == 2);
                    for a in arguments.iter() {
                        assert!(a.ty() == &Type_::Bit);
                    }

                    (
                        BehaviourValue::BuiltinOp {
                            op: BuiltinOp::Xor,
                            arguments,
                        },
                        vec![Type_::Bit],
                    )
                }
                "or" => {
                    assert!(arguments.len() == 2);
                    for a in arguments.iter() {
                        assert!(a.ty() == &Type_::Bit);
                    }

                    (
                        BehaviourValue::BuiltinOp {
                            op: BuiltinOp::Or,
                            arguments,
                        },
                        vec![Type_::Bit],
                    )
                }
                other => {
                    let other_header = headers.get(other).unwrap();

                    assert!(arguments.len() == other_header.in_ports.len());
                    for (a, p) in arguments.iter().zip(other_header.in_ports.iter()) {
                        assert!(a.ty() == &p.ty);
                    }

                    (
                        BehaviourValue::EntityOp {
                            op: other.to_string(),
                            arguments,
                            port_count: other_header.out_ports.len() as u32,
                        },
                        other_header
                            .out_ports
                            .iter()
                            .map(|p| p.ty.clone())
                            .collect(),
                    )
                }
            }
        }
        syntax::BehaviourValue::Variables { vars: raw_vars } => {
            let var_list: Vec<_> = raw_vars
                .into_iter()
                .map(|v| vars.get(v.1.content()).unwrap().clone())
                .collect();

            let types: Vec<_> = var_list.iter().map(|v| v.ty.clone()).collect();

            let value = BehaviourValue::Variables { vars: var_list };

            (value, types)
        }
    }
}

fn parse_entity(raw_entity: syntax::Entity, headers: &HashMap<String, EntityHeader>) -> Entity {
    let name = raw_entity.name.content();
    let current_header = headers.get(name).unwrap().clone();

    let mut behaviour: Vec<Behaviour> = Vec::new();
    let mut vars: HashMap<String, Variable> = HashMap::new();

    for stmnt in raw_entity.behaviour {
        match stmnt {
            syntax::BehaviourStatement::VarAssign { targets, value } => {
                let (b_value, value_types) = parse_value(value, &current_header, &vars, headers);

                assert!(value_types.len() == targets.len());

                let target_vars: Vec<Variable> = targets
                    .into_iter()
                    .zip(value_types.into_iter())
                    .map(|(v, ty)| {
                        let name = v.1.content().to_string();
                        let var = Variable {
                            name: name.clone(),
                            ty,
                        };

                        vars.insert(name, var.clone());
                        var
                    })
                    .collect();

                behaviour.push(Behaviour::VarAssign {
                    targets: target_vars,
                    value: b_value,
                });
            }
            syntax::BehaviourStatement::PortAssign { targets, value } => {
                let (b_value, value_types) = parse_value(value, &current_header, &vars, headers);

                let target_ports: Vec<_> = targets
                    .into_iter()
                    .map(|p_token| {
                        let p_name = p_token.1.content();

                        current_header
                            .out_ports
                            .iter()
                            .find(|p| p.name == p_name)
                            .unwrap()
                            .clone()
                    })
                    .collect();

                target_ports
                    .iter()
                    .zip(value_types.into_iter())
                    .for_each(|(p, t)| {
                        assert!(p.ty == t);
                    });

                behaviour.push(Behaviour::PortAssign {
                    targets: target_ports,
                    value: b_value,
                });
            }
        };
    }

    Entity {
        name: current_header.name,
        in_ports: current_header.in_ports,
        out_ports: current_header.out_ports,
        behaviour,
    }
}

impl Operand {
    pub fn ty(&self) -> &Type_ {
        match self {
            Self::Port(p) => &p.ty,
            Self::Variable(v) => &v.ty,
        }
    }
}

impl BehaviourValue {
    fn to_graph(
        &self,
        node_id: u32,
        in_ports: &HashMap<String, u32>,
        var_ids: &HashMap<String, u32>,
    ) -> (
        Option<graph::entity::Node>,
        Vec<graph::entity::Edge>,
        Vec<(u32, u32)>,
    ) {
        match self {
            Self::BuiltinOp { op, arguments } => {
                let (node_ty, sources) = match op {
                    BuiltinOp::And => (
                        graph::entity::NodeType::BuiltinOp {
                            op: graph::entity::BuiltinOp::And,
                        },
                        vec![(node_id, 0)],
                    ),
                    BuiltinOp::Xor => (
                        graph::entity::NodeType::BuiltinOp {
                            op: graph::entity::BuiltinOp::Xor,
                        },
                        vec![(node_id, 0)],
                    ),
                    BuiltinOp::Not => {
                        todo!()
                    }
                    BuiltinOp::Or => (
                        graph::entity::NodeType::BuiltinOp {
                            op: graph::entity::BuiltinOp::Or,
                        },
                        vec![(node_id, 0)],
                    ),
                };

                let edges = arguments
                    .iter()
                    .enumerate()
                    .map(|(index, arg)| match arg {
                        Operand::Port(p) => {
                            let src_id = in_ports.get(&p.name).unwrap();

                            graph::entity::Edge::new(*src_id, 0, node_id, index as u32)
                        }
                        Operand::Variable(v) => {
                            let src_id = match var_ids.get(&v.name) {
                                Some(i) => i,
                                None => {
                                    panic!("Unknown Variable: {}", v.name)
                                }
                            };

                            graph::entity::Edge::new(*src_id, 0, node_id, index as u32)
                        }
                    })
                    .collect();

                let node = graph::entity::Node::new(node_id, node_ty);
                (Some(node), edges, sources)
            }
            Self::EntityOp {
                op,
                arguments,
                port_count,
            } => {
                let node = graph::entity::Node::new(
                    node_id,
                    graph::entity::NodeType::EntityOp { name: op.clone() },
                );

                let edges: Vec<_> = arguments
                    .iter()
                    .enumerate()
                    .map(|(index, arg)| match arg {
                        Operand::Port(p) => {
                            let src_id = in_ports.get(&p.name).unwrap();

                            graph::entity::Edge::new(*src_id, 0, node_id, index as u32)
                        }
                        Operand::Variable(v) => {
                            let src_id = var_ids.get(&v.name).unwrap();

                            graph::entity::Edge::new(*src_id, 0, node_id, index as u32)
                        }
                    })
                    .collect();

                let outputs: Vec<_> = (0..*port_count).map(|p| (node_id, p)).collect();
                (Some(node), edges, outputs)
            }
            Self::Variables { vars } => {
                let srcs: Vec<_> = vars
                    .iter()
                    .map(|v| {
                        let v_id = var_ids.get(&v.name).unwrap();
                        (*v_id, 0)
                    })
                    .collect();

                (None, Vec::new(), srcs)
            }
        }
    }
}

impl Entity {
    pub fn graph(&self) -> graph::entity::Graph {
        let mut id = 0;

        let mut get_id = || {
            let tmp = id;
            id += 1;
            tmp
        };

        let (input_nodes, in_ports): (Vec<_>, HashMap<_, _>) = self
            .in_ports
            .iter()
            .enumerate()
            .map(|(index, port)| {
                let id = get_id();
                (
                    graph::entity::Node::new(
                        id,
                        graph::entity::NodeType::Input {
                            name: port.name.to_string(),
                            number: index as u32,
                        },
                    ),
                    (port.name.to_string(), id),
                )
            })
            .unzip();
        let (output_nodes, out_ports): (Vec<_>, HashMap<_, _>) = self
            .out_ports
            .iter()
            .enumerate()
            .map(|(index, port)| {
                let id = get_id();
                (
                    graph::entity::Node::new(
                        id,
                        graph::entity::NodeType::Output {
                            name: port.name.to_string(),
                            number: index as u32,
                        },
                    ),
                    (port.name.to_string(), id),
                )
            })
            .unzip();

        let mut var_nodes = HashMap::new();
        let mut b_edges = Vec::new();
        let mut b_nodes = Vec::new();
        for stmnt in self.behaviour.iter() {
            match stmnt {
                Behaviour::VarAssign { targets, value } => {
                    let node_id = get_id();

                    let (n_node, n_edges, outputs) = value.to_graph(node_id, &in_ports, &var_nodes);
                    if let Some(n) = n_node {
                        b_nodes.push(n);
                    }
                    b_edges.extend(n_edges);

                    for (var, src) in targets.iter().zip(outputs.iter()) {
                        let var_id = get_id();
                        var_nodes.insert(var.name.clone(), var_id);

                        b_nodes.push(graph::entity::Node::new(
                            var_id,
                            graph::entity::NodeType::Variable {
                                name: var.name.clone(),
                            },
                        ));
                        b_edges.push(graph::entity::Edge::new(src.0, src.1, var_id, 0));
                    }
                }
                Behaviour::PortAssign { targets, value } => {
                    let node_id = get_id();

                    let (n_node, n_edges, outputs) = value.to_graph(node_id, &in_ports, &var_nodes);
                    if let Some(n) = n_node {
                        b_nodes.push(n);
                    }
                    b_edges.extend(n_edges);

                    for (target, src) in targets.iter().zip(outputs.iter()) {
                        let port_id = out_ports.get(&target.name).unwrap();

                        b_edges.push(graph::entity::Edge::new(src.0, src.1, *port_id, 0));
                    }
                }
            };
        }

        let nodes: Vec<_> = input_nodes
            .into_iter()
            .chain(output_nodes.into_iter())
            .chain(b_nodes.into_iter())
            .collect();

        graph::entity::Graph::new(nodes, b_edges)
    }
}
