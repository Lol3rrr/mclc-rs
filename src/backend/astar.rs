use std::{
    collections::HashMap,
    fmt::Debug,
    hash::{BuildHasher, Hash},
};

mod openset;
pub use openset::*;

pub trait Container {
    type Index;
}

fn reconstruct_path<I, S>(came_from: &HashMap<I, I, S>, mut current: I) -> Vec<I>
where
    I: Eq + Hash + Clone,
    S: BuildHasher,
{
    let mut path = vec![current.clone()];

    while came_from.contains_key(&current) {
        current = came_from.get(&current).unwrap().clone();
        path.insert(0, current.clone());
    }

    path
}

pub fn path<C, DF, NF>(
    container: &mut C,
    start: C::Index,
    dest: C::Index,
    mut dist: DF,
    mut neigh: NF,
) -> Vec<C::Index>
where
    C: Container,
    C::Index: Eq + Hash + Clone + Debug,
    DF: FnMut(C::Index, C::Index) -> i64,
    NF: FnMut(&C, C::Index) -> Vec<(C::Index, i64)>,
{
    let mut n_openset = OpenSet::new((start.clone(), dist(start.clone(), dest.clone())));
    let mut came_from: HashMap<C::Index, C::Index, ahash::RandomState> = HashMap::default();
    let mut gscores: HashMap<C::Index, i64, ahash::RandomState> = HashMap::default();
    let mut fscores: HashMap<C::Index, i64, ahash::RandomState> = HashMap::default();

    gscores.insert(start.clone(), 0);
    fscores.insert(start.clone(), dist(start.clone(), dest.clone()));

    while !n_openset.is_empty() {
        let current = n_openset.pop();

        if current == dest {
            return reconstruct_path(&came_from, current);
        }

        let current_gscore = gscores.get(&current).cloned().unwrap();
        let neighbours = neigh(container, current.clone());
        for (neighbour, neighbour_cost) in neighbours {
            let tentative_gscore = current_gscore + neighbour_cost;
            let prev_neighbour_gscore = gscores.get(&neighbour).cloned().unwrap_or(i64::MAX);

            if tentative_gscore < prev_neighbour_gscore {
                came_from.insert(neighbour.clone(), current.clone());
                gscores.insert(neighbour.clone(), tentative_gscore);
                fscores.insert(
                    neighbour.clone(),
                    tentative_gscore + dist(neighbour.clone(), dest.clone()),
                );

                n_openset.update(
                    neighbour.clone(),
                    tentative_gscore + dist(neighbour.clone(), dest.clone()),
                );
            }
        }
    }

    dbg!(start, dest);

    todo!("Could not find a Path between these Points")
}
