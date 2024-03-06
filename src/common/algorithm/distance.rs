use crate::common::{max_3, min_3};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DistanceFormula {
    Manhattan,
    Diagonal,
    Euclidean,
    EuclideanSq,
    Chebyshev,
}

pub struct Distance;

impl Distance {
    pub fn get(formula: DistanceFormula, a: [i32; 3], b: [i32; 3]) -> f32 {
        match formula {
            DistanceFormula::Manhattan => Self::manhattan(a, b),
            DistanceFormula::Diagonal => Self::diagonal(a, b),
            DistanceFormula::Euclidean => Self::euclidean(a, b),
            DistanceFormula::EuclideanSq => Self::euclidean_sq(a, b),
            DistanceFormula::Chebyshev => Self::chebyshev(a, b),
        }
    }

    pub fn manhattan(a: [i32; 3], b: [i32; 3]) -> f32 {
        ((a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs()) as f32
    }

    pub fn diagonal(a: [i32; 3], b: [i32; 3]) -> f32 {
        let dx = (a[0] - b[0]).abs();
        let dy = (a[1] - b[1]).abs();
        let dz = (a[2] - b[2]).abs();

        (dx + dy + dz) as f32 - (0.59 * min_3(dx, dy, dz) as f32)
    }

    pub fn chebyshev(a: [i32; 3], b: [i32; 3]) -> f32 {
        let dx = (a[0] - b[0]).abs();
        let dy = (a[1] - b[1]).abs();
        let dz = (a[2] - b[2]).abs();

        max_3(dx, dy, dz) as f32
    }

    pub fn euclidean_sq(a: [i32; 3], b: [i32; 3]) -> f32 {
        let dx = (a[0] - b[0]).abs();
        let dy = (a[1] - b[1]).abs();
        let dz = (a[2] - b[2]).abs();

        (dx * dx + dy * dy + dz * dz) as f32
    }

    pub fn euclidean(a: [i32; 3], b: [i32; 3]) -> f32 {
        Self::euclidean_sq(a, b).sqrt()
    }
}
