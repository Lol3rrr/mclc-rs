use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

pub struct OpenSet<C> {
    values: HashMap<C, i64, ahash::RandomState>,
    inner: BTreeMap<i64, Vec<C>>,
}

impl<C> OpenSet<C>
where
    C: Eq + Hash + Clone,
{
    pub fn new(initial: (C, i64)) -> Self {
        Self {
            values: [(initial.0.clone(), initial.1)].into_iter().collect(),
            inner: [(initial.1, vec![initial.0])].into_iter().collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn pop(&mut self) -> C {
        let smallest = self.inner.iter_mut().next().unwrap();
        let item = smallest.1.remove(0);
        if smallest.1.is_empty() {
            let value = *smallest.0;
            self.inner.remove(&value);
        }
        self.values.remove(&item);

        item
    }

    pub fn update(&mut self, item: C, cost: i64) {
        match self.values.get(&item) {
            Some(v) => {
                let prev_cost = *v;

                let prev_list = self.inner.get_mut(&prev_cost).unwrap();
                let index = prev_list
                    .iter()
                    .enumerate()
                    .find(|(_, c)| c == &&item)
                    .map(|(i, _)| i)
                    .unwrap();

                prev_list.remove(index);
            }
            None => {}
        };

        let list = self.inner.entry(cost).or_insert(Vec::new());
        if !list.contains(&item) {
            list.push(item);
        }
    }
}
