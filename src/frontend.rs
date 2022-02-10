use std::{collections::HashMap, ops::Range, sync::Arc};

use crate::graph;

#[derive(Debug)]
pub struct Span {
    area: Range<usize>,
    content: Arc<String>,
}

impl Span {
    pub fn new<S>(content: S) -> Self
    where
        S: Into<String>,
    {
        let c_str = content.into();

        let area = 0..c_str.len();

        Self {
            area,
            content: Arc::new(c_str),
        }
    }

    pub fn content(&self) -> &str {
        &self.content[self.area.clone()]
    }

    #[must_use]
    pub fn sub_span(&self, area: Range<usize>) -> Self {
        Self {
            area,
            content: self.content.clone(),
        }
    }
}

mod semantics;
mod syntax;
mod tokens;

pub fn parse<S>(content: S, target: Option<String>) -> Result<graph::normalized::Graph, ()>
where
    S: Into<String>,
{
    let content_span = Span::new(content);

    let tokens = tokens::tokenize(content_span);

    let syntax = syntax::parse(tokens).unwrap();

    let s_entities = semantics::parse(syntax);

    let target_entity = match target {
        Some(t_name) => s_entities.iter().find(|e| e.name == t_name).unwrap(),
        None => s_entities.get(0).unwrap(),
    };

    let target_e_graph = target_entity.graph();
    let all_e_graphs: HashMap<_, _> = s_entities
        .into_iter()
        .map(|e| (e.name.clone(), e.graph()))
        .collect();

    let target_b_graph = target_e_graph.into_builtin(&all_e_graphs);

    Ok(target_b_graph.into_normalized())
}
