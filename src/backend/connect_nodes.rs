use std::vec;

use crate::{
    backend::{
        astar, space::Space, PlacedNode, PlacedNodeData, SpaceBlock, SpaceCell, RESERVE_SPACE,
    },
    graph,
};

pub fn connect_nodes(
    space: &mut Space<SpaceCell>,
    graph: &graph::normalized::Graph,
    placed_nodes: &[PlacedNode],
) {
    let edges = &graph.edges;

    for edge in edges {
        place_edge(space, edge, placed_nodes);
    }
}

fn place_edge(space: &mut Space<SpaceCell>, edge: &graph::normalized::Edge, nodes: &[PlacedNode]) {
    let src_id = edge.src_id;
    let src_node = nodes
        .iter()
        .find(|PlacedNode(_, id, _)| *id == src_id)
        .unwrap();
    let src_pos = match &src_node.2 {
        PlacedNodeData::Input { .. } => src_node.0,
        PlacedNodeData::Output { .. } => src_node.0,
        PlacedNodeData::Variable { .. } => src_node.0,
        PlacedNodeData::Splitter { ports, .. } => *ports.get(edge.src_port as usize).unwrap(),
        PlacedNodeData::Entity { out_ports, .. } => *out_ports.get(edge.src_port as usize).unwrap(),
    };

    let dest_id = edge.dest_id;
    let dest_node = nodes
        .iter()
        .find(|PlacedNode(_, id, _)| *id == dest_id)
        .unwrap();
    let dest_pos = match &dest_node.2 {
        PlacedNodeData::Input { .. } => dest_node.0,
        PlacedNodeData::Output { .. } => dest_node.0,
        PlacedNodeData::Variable { .. } => dest_node.0,
        PlacedNodeData::Splitter { input, .. } => *input,
        PlacedNodeData::Entity { in_ports, .. } => *in_ports.get(edge.dest_port as usize).unwrap(),
    };

    let unreserve_src_pos_x = src_pos.0..(src_pos.0 + RESERVE_SPACE + 1);
    let unreserve_dest_pos_x = (dest_pos.0.saturating_sub(RESERVE_SPACE + 1))..dest_pos.0;

    let unreserve_src_pos = unreserve_src_pos_x.map(|x| (x, src_pos.1, src_pos.2));
    let unreserve_dest_pos = unreserve_dest_pos_x
        .flat_map(|x| [(x, dest_pos.1, dest_pos.2), (x, dest_pos.1, dest_pos.2 + 1)]);

    let unreserve_pos = unreserve_src_pos.chain(unreserve_dest_pos);

    for pos in unreserve_pos {
        space.set(pos, |_| SpaceCell::Empty);
    }

    let search_s_pos = (src_pos.0 + 1, src_pos.1, src_pos.2);
    let search_d_pos = (dest_pos.0 - 1, dest_pos.1, dest_pos.2);

    let mut path = astar::path(
        space,
        search_s_pos,
        search_d_pos,
        |(src_x, src_y, src_z), (dest_x, dest_y, dest_z)| {
            let result = (dest_x as i64 - src_x as i64).abs()
                + (dest_y as i64 - src_y as i64).abs()
                + (dest_z as i64 - src_z as i64).abs();

            result as i64
        },
        |s, pos| neighbours(s, pos, search_d_pos),
    );

    path.push(src_pos);
    path.push(dest_pos);

    place_path(space, path);
}

fn neighbours(
    s: &Space<SpaceCell>,
    pos: (usize, usize, usize),
    dest: (usize, usize, usize),
) -> Vec<((usize, usize, usize), i64)> {
    let base = base_neighbours(s, pos);

    if let Some(v) = base.iter().find(|(p, _)| p == &dest) {
        return vec![*v];
    }

    base.into_iter()
        .filter(|(pos, _)| matches!(s.get(*pos), SpaceCell::Empty))
        .filter(|((x, y, z), _)| matches!(s.get((*x, *y, z + 1)), SpaceCell::Empty))
        .filter(|((x, y, z), _)| {
            matches!(
                s.get((*x, *y, z - 1)),
                SpaceCell::Empty | SpaceCell::Reserved
            )
        })
        .collect()
}

fn base_neighbours(
    s: &Space<SpaceCell>,
    pos: (usize, usize, usize),
) -> Vec<((usize, usize, usize), i64)> {
    let base_cords = [
        (pos.0 + 1, pos.1, pos.2),
        (pos.0 - 1, pos.1, pos.2),
        (pos.0, pos.1 + 1, pos.2),
        (pos.0, pos.1.saturating_sub(1), pos.2),
    ];

    let lower_cords = base_cords.into_iter().map(|(x, y, z)| (x, y, z + 1));

    let top_free = if pos.2 > 0 {
        matches!(s.get((pos.0, pos.1, pos.2 - 1)), SpaceCell::Empty)
    } else {
        false
    };

    let upper_cords = base_cords
        .into_iter()
        .filter(|_| top_free)
        .map(|(x, y, z)| (x, y, z - 1));

    base_cords
        .into_iter()
        .map(|p| (p, 1))
        .chain(lower_cords.map(|p| (p, 2)))
        .chain(upper_cords.map(|p| (p, 2)))
        .collect()
}

fn place_path(s: &mut Space<SpaceCell>, path: Vec<(usize, usize, usize)>) {
    for pos in path {
        s.set(pos, |_| SpaceCell::Used(SpaceBlock::Redstone));
        s.set((pos.0, pos.1, pos.2 + 1), |_| {
            SpaceCell::Used(SpaceBlock::SolidBlock)
        });

        let sourinding = [
            (pos.0 + 1, pos.1, pos.2),
            (pos.0.saturating_sub(1), pos.1, pos.2),
            (pos.0, pos.1 + 1, pos.2),
            (pos.0, pos.1 - 1, pos.2),
            (pos.0, pos.1, pos.2 + 1),
            (pos.0 + 1, pos.1, pos.2 - 1),
            (pos.0.saturating_sub(1), pos.1, pos.2 - 1),
            (pos.0, pos.1 + 1, pos.2 - 1),
            (pos.0, pos.1 - 1, pos.2 - 1),
            (pos.0 + 1, pos.1, pos.2 + 1),
            (pos.0.saturating_sub(1), pos.1, pos.2 + 1),
            (pos.0, pos.1 + 1, pos.2 + 1),
            (pos.0, pos.1 - 1, pos.2 + 1),
        ];
        for s_pos in sourinding {
            s.set(s_pos, |prev| match prev {
                SpaceCell::Empty => SpaceCell::Reserved,
                SpaceCell::Reserved => SpaceCell::Reserved,
                other => other.clone(),
            });
        }
    }
}
