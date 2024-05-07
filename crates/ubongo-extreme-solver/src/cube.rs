use crate::Axial;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cube(pub i32, pub i32, pub i32);

impl Add for Cube {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

impl Sub for Cube {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0, self.1 - other.1, self.2 - other.2)
    }
}

impl From<Axial> for Cube {
    fn from(coord: Axial) -> Self {
        Self(coord.0, -coord.0 - coord.1, coord.1)
    }
}

impl Cube {
    pub fn rotate(self, pivot: Self) -> Self {
        let pivot_to_self = self - pivot;
        pivot + Cube(-pivot_to_self.1, -pivot_to_self.2, -pivot_to_self.0)
    }

    pub fn rotate_many(self, pivot: Self, steps: usize) -> Self {
        let mut pivot_to_self = self - pivot;
        for _ in 0..steps {
            pivot_to_self = Cube(-pivot_to_self.1, -pivot_to_self.2, -pivot_to_self.0)
        }
        pivot_to_self + pivot
    }
}
