use crate::backend::{space, SpaceCell};

pub fn reserve_around(
    space: &mut space::Space<SpaceCell>,
    pos: (usize, usize, usize),
    size: (usize, usize, usize),
    reserve_size: usize,
) {
    let (x, y, z) = pos;
    let (width, depth, _) = size;

    let row = (x.saturating_sub(reserve_size))..(x.saturating_add(width + reserve_size));
    let top_row = row.clone().map(|x_pos| (x_pos, y - 1, z));
    let bottom_row = row.map(|x_pos| (x_pos, y + depth, z));

    let column = (y.saturating_sub(2))..(y.saturating_add(depth + 1));
    let left_column = column
        .clone()
        .map(|y_pos| (x.saturating_sub(reserve_size), y_pos, z));
    let right_column = column.map(|y_pos| (x.saturating_add(width + reserve_size), y_pos, z));

    let raw_sides = left_column.chain(right_column);

    let sides = raw_sides
        .clone()
        .chain(raw_sides.map(|(x, y, z)| (x, y, z + 1)));

    let to_reserve = top_row.chain(bottom_row).chain(sides);

    to_reserve.for_each(|pos| {
        space.set(pos, |_| SpaceCell::Reserved);
    });
}
