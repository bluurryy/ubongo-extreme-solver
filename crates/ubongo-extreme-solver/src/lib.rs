#![feature(type_alias_impl_trait)]
#![allow(clippy::len_without_is_empty)]

mod axial;
mod cube;

pub use axial::{Axial, AxialAabb};
pub use cube::Cube;

#[cfg(test)]
mod tests;

use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

use std::fmt::Debug;

pub fn translate(coords: &mut [Axial], offset: Axial) {
    for coord in coords {
        *coord += offset;
    }
}

pub fn rotate(coords: &mut [Axial]) {
    for coord in coords {
        *coord = coord.rotate(Axial::ZERO);
    }
}

pub fn rotate_many(coords: &mut [Axial], count: usize) {
    for coord in coords {
        *coord = coord.rotate_many(Axial::ZERO, count);
    }
}

pub fn flip(coords: &mut [Axial]) {
    for coord in coords {
        *coord = coord.flip();
    }
}

pub fn min(coords: &[Axial]) -> Axial {
    let mut min = Axial::MAX;

    for &coord in coords {
        min = min.min(coord);
    }

    min
}

pub fn aabb(coords: &[Axial]) -> AxialAabb {
    let mut min = Axial::MAX;
    let mut max = Axial::MIN;

    for &coord in coords {
        min = min.min(coord);
        max = max.max(coord);
    }

    AxialAabb { min, max }
}

pub fn spot_candidates(board: &[Axial], piece: &[Axial]) -> impl Iterator<Item = Axial> + Clone {
    let board_aabb = aabb(board);
    let piece_aabb = aabb(piece);
    let wiggle = board_aabb.size() - piece_aabb.size();
    let piece_origin = Axial::ZERO - piece_aabb.min;

    let max_x = wiggle.0 + 1;
    let max_y = wiggle.1 + 1;

    let offset = board_aabb.min + piece_origin;
    (0..max_x).flat_map(move |x| (0..max_y).map(move |y| Axial(x, y) + offset))
}

pub type Spots = impl Iterator<Item = Axial> + Clone;

pub fn spots(board: &[Axial], piece: &[Axial]) -> Spots {
    let board_clone = board.to_vec();
    let piece_clone = piece.to_vec();
    spot_candidates(board, piece).filter(move |spot| {
        piece_clone.iter().all(|cell| {
            let translated = *cell + *spot;
            board_clone.contains(&translated)
        })
    })
}

pub fn place(board: &mut Vec<Axial>, piece: &[Axial]) {
    board.retain(|coord| !piece.contains(coord))
}

pub fn canonicalize_place(piece: &mut [Axial]) {
    piece.sort_unstable_by_key(|&axial| axial.key())
}

pub fn canonicalize_shape(piece: &mut [Axial]) {
    let min = min(piece);
    for coord in piece.iter_mut() {
        *coord -= min;
    }
    canonicalize_place(piece);
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Game {
    pub board: Vec<Axial>,
    pub pieces: Vec<Vec<Axial>>,
}

impl Game {
    pub const fn new() -> Self {
        Self {
            board: Vec::new(),
            pieces: Vec::new(),
        }
    }

    pub fn solver(self) -> Solver {
        Solver::new(self)
    }
}

fn piece_permutations(mut piece: Vec<Axial>) -> Vec<Vec<Axial>> {
    canonicalize_shape(&mut piece);
    let mut permutations = vec![piece.clone()];

    let mut rotations = 0;
    let mut is_flipped = false;

    #[allow(clippy::never_loop)]
    loop {
        'check_piece: {
            if rotations < 6 {
                rotate(&mut piece);
                rotations += 1;
                break 'check_piece;
            }

            if !is_flipped {
                flip(&mut piece);
                rotations = 0;
                is_flipped = true;
                break 'check_piece;
            }

            return permutations;
        }

        canonicalize_shape(&mut piece);

        if !permutations.contains(&piece) {
            permutations.push(piece.clone());
        }
    }
}

#[derive(Clone)]
pub struct Placer {
    board: Vec<Axial>,
    spots: Spots,
    permutation_idx: usize,
}

impl Default for Placer {
    fn default() -> Self {
        Self {
            permutation_idx: 0,
            board: Default::default(),
            spots: spots(&[], &[]),
        }
    }
}

impl Placer {
    pub fn new(board: Vec<Axial>, permutations: &[Vec<Axial>]) -> Self {
        Self {
            spots: spots(&board, &permutations[0]),
            board,
            permutation_idx: 0,
        }
    }
}

impl Placer {
    #[must_use]
    fn next_place(&mut self, placed: &mut [Axial], permutations: &[Vec<Axial>]) -> bool {
        while let Some(piece) = permutations.get(self.permutation_idx) {
            if let Some(spot) = self.spots.next() {
                // we found a spot, returning...
                placed.copy_from_slice(piece);
                translate(placed, spot);
                return true;
            }

            self.permutation_idx += 1;

            if let Some(piece) = permutations.get(self.permutation_idx) {
                self.spots = spots(&self.board, piece);
                continue;
            }

            // we have exhausted all possibilities
            break;
        }

        false
    }
}

#[derive(Default, Clone)]
pub struct Solver {
    pub game: Game,
    pub pieces_permutations: Vec<Vec<Vec<Axial>>>,
    pub placers: Vec<Placer>,
    pub pieces: Vec<Vec<Axial>>,
    pub solutions: IndexSet<Vec<Vec<Axial>>>,
    pub work_idx: usize,
}

impl Debug for Solver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Solver")
            .field("game", &self.game)
            .field("solutions", &self.solutions)
            .field("work_idx", &self.work_idx)
            .finish()
    }
}

impl Solver {
    pub fn new(game: Game) -> Self {
        let pieces_permutations: Vec<_> = game.pieces.iter().cloned().map(piece_permutations).collect();
        let mut placers = vec![Default::default(); game.pieces.len()];

        if !game.pieces.is_empty() {
            placers[0] = Placer::new(game.board.clone(), &pieces_permutations[0]);
        }

        Self {
            pieces_permutations,
            placers,
            pieces: game.pieces.clone(),
            solutions: Default::default(),
            work_idx: 0,
            game,
        }
    }
}

impl Iterator for Solver {
    type Item = ();

    fn next(&mut self) -> Option<()> {
        loop {
            if let Some(placer) = self.placers.get_mut(self.work_idx) {
                let placed_piece = &mut self.pieces[self.work_idx];
                let piece_permutations = &self.pieces_permutations[self.work_idx];

                if placer.next_place(placed_piece, piece_permutations) {
                    // we placed our piece, lets move on to the next one
                    self.work_idx += 1;

                    if self.work_idx < self.game.pieces.len() {
                        let mut board = placer.board.clone();
                        place(&mut board, placed_piece);

                        self.placers[self.work_idx] = Placer::new(board, &self.pieces_permutations[self.work_idx]);
                    }

                    // we did something :) lets return
                    break Some(());
                }

                // we failed to place our work piece

                if self.work_idx == 0 {
                    // we exhausted all transformations, all solutions have been found
                    break None;
                }

                // ugh, lets try something else
                self.work_idx -= 1;
                continue;
            }

            // we exhausted all placers, so we got a solution
            self.solutions.insert({
                let mut pieces = self.pieces.clone();
                for piece in &mut pieces {
                    canonicalize_place(piece);
                }
                pieces
            });

            // lets try to find more solutions
            self.work_idx -= 1;
            continue;
        }
    }
}
