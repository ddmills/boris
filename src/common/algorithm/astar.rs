use ordered_float::*;
use std::collections::HashMap;

use crate::common::PriorityQueue;

pub struct AStarSettings<F, T, C, K>
where
    T: std::cmp::Eq + std::hash::Hash + Copy,
    F: Fn(T, T) -> f32,
    C: Fn(T, T) -> f32,
    K: Fn(T) -> Vec<T>,
{
    pub start: T,
    pub goal: T,
    pub cost: C,
    pub heuristic: F,
    pub neighbors: K,
    pub max_depth: u32,
}

pub struct AStarResult<T> {
    pub is_success: bool,
    pub path: Vec<T>,
    pub cost: f32,
}

pub fn astar<F, T, C, K>(settings: AStarSettings<F, T, C, K>) -> AStarResult<T>
where
    T: std::cmp::Eq + std::hash::Hash + Copy,
    F: Fn(T, T) -> f32,
    C: Fn(T, T) -> f32,
    K: Fn(T) -> Vec<T>,
{
    let mut depth = 0;
    let mut open = PriorityQueue::new();
    let mut from = HashMap::new();
    let mut costs = HashMap::new();

    let mut result = AStarResult {
        is_success: false,
        path: vec![],
        cost: 0.,
    };

    if (settings.cost)(settings.start, settings.goal) == 0. {
        return result;
    }

    open.put(settings.start, OrderedFloat(0.));
    costs.insert(settings.start, OrderedFloat(0.));

    while !open.is_empty() {
        depth += 1;

        if depth >= settings.max_depth {
            break;
        }

        let current = open.pop().unwrap();

        if current == settings.goal {
            result.is_success = true;
            break;
        }

        let neighbors = (settings.neighbors)(current);

        for next in neighbors {
            let cost = if next == settings.goal {
                0.
            } else {
                (settings.cost)(current, next)
            };

            if cost == f32::INFINITY {
                continue;
            }

            let new_cost = costs.get(&current).unwrap() + cost;

            if !costs.contains_key(&next) || new_cost < *costs.get(&next).unwrap() {
                costs.insert(next, new_cost);

                // todo: use a min priority queue and remove hard-coded float here
                let priority =
                    OrderedFloat(100000.0) - new_cost * (settings.heuristic)(next, settings.goal);

                open.put(next, priority);
                from.insert(next, current);
            }
        }
    }

    if !result.is_success {
        return result;
    }

    result.path.push(settings.goal);
    result.cost = **costs.get(&settings.goal).unwrap();

    let mut previous_pos = &settings.goal; // = from.get(&settings.goal).unwrap();

    while from.contains_key(previous_pos) {
        let f = from.get(previous_pos).unwrap();
        result.path.push(*f);
        previous_pos = f;
    }

    // note: path is returned in reverse order
    result
}
