use crate::Cube;
use serde::{Deserialize, Serialize};
use std::{
    hash::Hash,
    ops::{Add, AddAssign, Sub, SubAssign},
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Axial(pub i32, pub i32);

#[allow(clippy::derived_hash_with_manual_eq)]
impl Hash for Axial {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().hash(state);
    }
}

impl Add for Axial {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0.wrapping_add(other.0), self.1.wrapping_add(other.1))
    }
}

impl AddAssign for Axial {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_add(rhs.0);
        self.1 = self.1.wrapping_add(rhs.1);
    }
}

impl Sub for Axial {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0, self.1 - other.1)
    }
}

impl SubAssign for Axial {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_sub(rhs.0);
        self.1 = self.1.wrapping_sub(rhs.1);
    }
}

impl From<Cube> for Axial {
    fn from(coord: Cube) -> Self {
        Self(coord.0, coord.2)
    }
}

impl Axial {
    pub const ZERO: Axial = Axial::splat(0);
    pub const MIN: Axial = Axial::splat(i32::MIN);
    pub const MAX: Axial = Axial::splat(i32::MAX);

    #[must_use]
    pub const fn splat(i: i32) -> Self {
        Self(i, i)
    }

    #[must_use]
    pub fn flip(self) -> Self {
        Self(self.0 + self.1, -self.1)
    }

    #[must_use]
    pub fn rotate(self, pivot: Self) -> Self {
        Cube::from(self).rotate(pivot.into()).into()
    }

    #[must_use]
    pub fn rotate_many(self, pivot: Self, steps: usize) -> Self {
        Cube::from(self).rotate_many(pivot.into(), steps).into()
    }

    #[must_use]
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0), self.1.min(other.1))
    }

    #[must_use]
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0), self.1.max(other.1))
    }

    #[must_use]
    pub fn key(self) -> u64 {
        unsafe { std::mem::transmute(self) }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AxialAabb {
    pub min: Axial,
    pub max: Axial,
}

impl AxialAabb {
    pub fn size(self) -> Axial {
        Axial(self.max.0.wrapping_sub(self.min.0), self.max.1.wrapping_sub(self.min.1))
    }
}
