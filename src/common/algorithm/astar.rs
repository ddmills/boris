use ordered_float::*;
use std::collections::HashMap;

use super::{Distance, DistanceFormula};
use crate::common::PriorityQueue;

pub struct AStarSettings<F>
where
    F: Fn([i32; 3], [i32; 3]) -> f32,
{
    pub start: [i32; 3],
    pub goal: [i32; 3],
    pub allow_diagonals: bool,
    pub cost: F,
    pub max_depth: u32,
}

pub struct AStarResult {
    pub is_success: bool,
    pub path: Vec<[i32; 3]>,
    pub cost: f32,
}

pub fn astar<F: Fn([i32; 3], [i32; 3]) -> f32>(settings: AStarSettings<F>) -> AStarResult {
    let heuristic = if settings.allow_diagonals {
        DistanceFormula::Diagonal
    } else {
        DistanceFormula::Manhattan
    };
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

        let neighbors = if settings.allow_diagonals {
            neighbors_diagonal(current)
        } else {
            neighbors(current)
        };

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

                let priority = OrderedFloat(100000.0)
                    - new_cost * Distance::get(heuristic, next, settings.goal);

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

fn neighbors(point: [i32; 3]) -> Vec<[i32; 3]> {
    let [x, y, z] = point;

    vec![
        [x + 1, y, z],
        [x - 1, y, z],
        [x, y + 1, z],
        [x, y - 1, z],
        [x, y, z + 1],
        [x, y, z - 1],
    ]
}

fn neighbors_diagonal(point: [i32; 3]) -> Vec<[i32; 3]> {
    let [x, y, z] = point;
    vec![
        [x - 1, y + 1, z - 1],
        [x, y + 1, z - 1],
        [x + 1, y + 1, z - 1],
        [x - 1, y + 1, z],
        [x, y + 1, z],
        [x + 1, y + 1, z],
        [x - 1, y + 1, z + 1],
        [x, y + 1, z + 1],
        [x + 1, y + 1, z + 1],
        [x - 1, y, z - 1],
        [x, y, z - 1],
        [x + 1, y, z - 1],
        [x - 1, y, z],
        [x + 1, y, z],
        [x - 1, y, z + 1],
        [x, y, z + 1],
        [x + 1, y, z + 1],
        [x - 1, y - 1, z - 1],
        [x, y - 1, z - 1],
        [x + 1, y - 1, z - 1],
        [x - 1, y - 1, z],
        [x, y - 1, z],
        [x + 1, y - 1, z],
        [x - 1, y - 1, z + 1],
        [x, y - 1, z + 1],
        [x + 1, y - 1, z + 1],
    ]
}
