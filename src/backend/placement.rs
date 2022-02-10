use crate::{
    backend::{space, Orientation, PlacedNodeData, SpaceBlock, SpaceCell, RESERVE_SPACE},
    graph,
};

mod reserve;
use reserve::reserve_around;

pub fn place_node(
    space: &mut space::Space<SpaceCell>,
    to_place: graph::normalized::Node,
    x_offset: usize,
    y_offset: usize,
    z_pos: usize,
) -> ((usize, usize, usize), PlacedNodeData) {
    match to_place.inner {
        graph::normalized::NodeType::Input { name, .. } => {
            space.set((x_offset, y_offset, z_pos), |_| {
                SpaceCell::Used(SpaceBlock::Redstone)
            });
            space.set((x_offset, y_offset, z_pos + 1), |_| {
                SpaceCell::Used(SpaceBlock::SolidBlock)
            });
            reserve_around(space, (x_offset, y_offset, z_pos), (1, 1, 1), RESERVE_SPACE);
            ((1, 1, 1), PlacedNodeData::Input { name })
        }
        graph::normalized::NodeType::Output { name, .. } => {
            space.set((x_offset, y_offset, z_pos), |_| {
                SpaceCell::Used(SpaceBlock::Redstone)
            });
            space.set((x_offset, y_offset, z_pos + 1), |_| {
                SpaceCell::Used(SpaceBlock::SolidBlock)
            });
            reserve_around(space, (x_offset, y_offset, z_pos), (1, 1, 1), RESERVE_SPACE);
            ((1, 1, 1), PlacedNodeData::Output { name })
        }
        graph::normalized::NodeType::Variable { name } => {
            space.set((x_offset, y_offset, z_pos), |_| {
                SpaceCell::Used(SpaceBlock::Redstone)
            });
            space.set((x_offset, y_offset, z_pos + 1), |_| {
                SpaceCell::Used(SpaceBlock::SolidBlock)
            });
            reserve_around(space, (x_offset, y_offset, z_pos), (1, 1, 1), RESERVE_SPACE);
            ((1, 1, 1), PlacedNodeData::Variable { name })
        }
        graph::normalized::NodeType::Splitter { port_count: ports } => {
            let height = 1 + 2 * ((ports as usize) - 1);
            let input_height = (height - 1) / 2;
            let port_yoff = (0..(ports as usize)).map(|p| p * 2);

            let port_pos = port_yoff.map(|y_off| (x_offset + 2, y_offset + y_off, z_pos));
            let connecting_pos = (0..height).map(|y_off| (x_offset + 1, y_offset + y_off, z_pos));

            let place_pos = std::iter::once((x_offset, y_offset + input_height, z_pos))
                .chain(port_pos.clone())
                .chain(connecting_pos);

            place_pos.clone().for_each(|pos| {
                space.set(pos, |_| SpaceCell::Used(SpaceBlock::Redstone));
                space.set((pos.0, pos.1, pos.2 + 1), |_| {
                    SpaceCell::Used(SpaceBlock::SolidBlock)
                });
            });

            reserve_around(
                space,
                (x_offset, y_offset, z_pos),
                (3, height, 1),
                RESERVE_SPACE,
            );
            (
                (3, height, 1),
                PlacedNodeData::Splitter {
                    input: (x_offset, y_offset + input_height, z_pos),
                    ports: port_pos.collect(),
                },
            )
        }
        graph::normalized::NodeType::Operation { op } => match op {
            graph::normalized::BuiltinOp::Xor => {
                let redstone_pos = [
                    (x_offset, y_offset, z_pos),
                    (x_offset, y_offset + 3, z_pos),
                    (x_offset + 3, y_offset, z_pos),
                    (x_offset + 3, y_offset + 3, z_pos),
                    (x_offset + 4, y_offset, z_pos),
                    (x_offset + 4, y_offset + 1, z_pos),
                    (x_offset + 4, y_offset + 2, z_pos),
                    (x_offset + 4, y_offset + 3, z_pos),
                    (x_offset + 5, y_offset + 1, z_pos),
                    (x_offset + 6, y_offset + 1, z_pos),
                ];
                let solid_pos = [
                    (x_offset + 2, y_offset, z_pos),
                    (x_offset + 2, y_offset + 1, z_pos),
                    (x_offset + 2, y_offset + 2, z_pos),
                    (x_offset + 2, y_offset + 3, z_pos),
                ];
                let repeater_pos = [
                    (x_offset + 1, y_offset, z_pos),
                    (x_offset + 1, y_offset + 3, z_pos),
                ];
                let comparator_pos = [
                    (x_offset + 3, y_offset + 1, z_pos),
                    (x_offset + 3, y_offset + 2, z_pos),
                ];

                for pos in redstone_pos {
                    space.set(pos, |_| SpaceCell::Used(SpaceBlock::Redstone));
                    space.set((pos.0, pos.1, pos.2 + 1), |_| {
                        SpaceCell::Used(SpaceBlock::SolidBlock)
                    });
                }
                for pos in solid_pos {
                    space.set(pos, |_| SpaceCell::Used(SpaceBlock::SolidBlock));
                }
                for pos in repeater_pos {
                    space.set(pos, |_| {
                        SpaceCell::Used(SpaceBlock::Repeater {
                            direction: Orientation::East,
                        })
                    });
                    space.set((pos.0, pos.1, pos.2 + 1), |_| {
                        SpaceCell::Used(SpaceBlock::SolidBlock)
                    });
                }
                for pos in comparator_pos {
                    space.set(pos, |_| {
                        SpaceCell::Used(SpaceBlock::Comparator {
                            direction: Orientation::East,
                            activated: true,
                        })
                    });
                    space.set((pos.0, pos.1, pos.2 + 1), |_| {
                        SpaceCell::Used(SpaceBlock::SolidBlock)
                    });
                }

                reserve_around(space, (x_offset, y_offset, z_pos), (7, 4, 1), RESERVE_SPACE);

                (
                    (7, 4, 1),
                    PlacedNodeData::Entity {
                        in_ports: [(x_offset, y_offset, z_pos), (x_offset, y_offset + 3, z_pos)]
                            .to_vec(),
                        out_ports: [(x_offset + 6, y_offset + 1, z_pos)].to_vec(),
                    },
                )
            }
            graph::normalized::BuiltinOp::And => {
                let redstone_pos = [
                    (x_offset, y_offset, z_pos),
                    (x_offset, y_offset + 2, z_pos),
                    (x_offset + 3, y_offset + 1, z_pos),
                    (x_offset + 4, y_offset + 1, z_pos),
                ];
                let repeater_pos = [
                    (x_offset + 1, y_offset, z_pos),
                    (x_offset + 1, y_offset + 2, z_pos),
                ];
                let torch_pos = [
                    (x_offset + 3, y_offset, z_pos),
                    (x_offset + 3, y_offset + 2, z_pos),
                ];

                let solid_pos = [
                    (x_offset + 2, y_offset, z_pos),
                    (x_offset + 2, y_offset + 1, z_pos),
                    (x_offset + 2, y_offset + 2, z_pos),
                ];

                for pos in redstone_pos {
                    space.set(pos, |_| SpaceCell::Used(SpaceBlock::Redstone));
                    space.set((pos.0, pos.1, pos.2 + 1), |_| {
                        SpaceCell::Used(SpaceBlock::SolidBlock)
                    });
                }
                for pos in repeater_pos {
                    space.set(pos, |_| {
                        SpaceCell::Used(SpaceBlock::Repeater {
                            direction: Orientation::East,
                        })
                    });
                    space.set((pos.0, pos.1, pos.2 + 1), |_| {
                        SpaceCell::Used(SpaceBlock::SolidBlock)
                    });
                }
                for pos in torch_pos {
                    space.set(pos, |_| {
                        SpaceCell::Used(SpaceBlock::TorchOnBlock {
                            direction: Orientation::West,
                        })
                    });
                }
                for pos in solid_pos {
                    space.set(pos, |_| SpaceCell::Used(SpaceBlock::SolidBlock));
                }

                reserve_around(space, (x_offset, y_offset, z_pos), (5, 3, 1), RESERVE_SPACE);

                (
                    (5, 3, 1),
                    PlacedNodeData::Entity {
                        in_ports: [(x_offset, y_offset, z_pos), (x_offset, y_offset + 2, z_pos)]
                            .to_vec(),
                        out_ports: [(x_offset + 4, y_offset + 1, z_pos)].to_vec(),
                    },
                )
            }
            graph::normalized::BuiltinOp::Or => {
                let redstone_pos = [
                    (x_offset, y_offset, z_pos),
                    (x_offset, y_offset + 2, z_pos),
                    (x_offset + 1, y_offset, z_pos),
                    (x_offset + 1, y_offset + 1, z_pos),
                    (x_offset + 1, y_offset + 2, z_pos),
                    (x_offset + 2, y_offset + 1, z_pos),
                ];

                for pos in redstone_pos {
                    space.set(pos, |_| SpaceCell::Used(SpaceBlock::Redstone));
                    space.set((pos.0, pos.1, pos.2 + 1), |_| {
                        SpaceCell::Used(SpaceBlock::SolidBlock)
                    });
                }

                reserve_around(space, (x_offset, y_offset, z_pos), (3, 3, 1), RESERVE_SPACE);

                (
                    (3, 3, 1),
                    PlacedNodeData::Entity {
                        in_ports: [(x_offset, y_offset, z_pos), (x_offset, y_offset + 2, z_pos)]
                            .to_vec(),
                        out_ports: [(x_offset + 2, y_offset + 1, z_pos)].to_vec(),
                    },
                )
            }
            other => todo!("Handle: {:?}", other),
        },
    }
}
