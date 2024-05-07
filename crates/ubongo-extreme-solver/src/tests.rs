use std::path::Path;

use expect_test::expect;

use crate::Game;

use rand::{seq::SliceRandom, Rng};

use super::*;

fn random_piece() -> Vec<Axial> {
    let mut rng = rand::thread_rng();
    let mut piece = vec![];

    for _ in 0..rng.gen_range(2..12) {
        piece.push(Axial(rng.gen_range(-100..100), rng.gen_range(-100..100)));
    }

    piece
}

#[test]
fn canonical() {
    let mut rng = rand::thread_rng();

    for _ in 0..1_000 {
        let original = random_piece();

        for _ in 1..6 {
            let mut shuffled = original.clone();
            shuffled.shuffle(&mut rng);

            let mut original_canon = original.clone();
            let mut shuffled_canon = shuffled.clone();

            canonicalize_place(&mut original_canon);
            canonicalize_place(&mut shuffled_canon);

            let mut shuffled_canon_again = shuffled_canon.clone();
            canonicalize_place(&mut shuffled_canon_again);

            // println!("{i}~: {rotated:?}");
            // println!("{i}*: {rotated_canon:?}");
            assert_eq!(original_canon, shuffled_canon);
            assert_eq!(shuffled_canon, shuffled_canon_again);
        }
    }
}

fn index_to_name(i: usize) -> char {
    char::from_u32(97 + i as u32).unwrap()
}

fn debug_mutator(pieces: Vec<Vec<Axial>>, f: fn(&mut [Axial])) {
    let mut modified = vec![];

    for (i, piece) in pieces.iter().enumerate() {
        let name = index_to_name(i);
        println!("{name}:   {piece:?}");

        let mut canon = piece.clone();
        f(&mut canon);
        modified.push(canon);
    }

    let mut keys = vec![];

    for (i, piece) in modified.iter().enumerate() {
        let name = index_to_name(i);
        println!("{name}_m: {piece:?}");

        let key = piece.iter().copied().map(Axial::key).collect::<Vec<_>>();
        keys.push(key);
    }

    for (i, key) in keys.iter().enumerate() {
        let name = index_to_name(i);
        println!("{name}_k: {key:?}");
    }
}

#[test]
fn canonical_debug() {
    debug_mutator(
        vec![
            vec![Axial(2, 0), Axial(1, 0), Axial(0, 0), Axial(1, 1), Axial(0, 1)],
            vec![Axial(0, 0), Axial(1, 0), Axial(2, 0), Axial(0, 1), Axial(1, 1)],
        ],
        canonicalize_place,
    );
}

const MAX_STEPS: usize = 1_000_000;

fn test_file(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("data/{name}.json"));
    let content = std::fs::read_to_string(path).unwrap();
    let game: Game = serde_json::from_str(&content).unwrap();
    let mut solver = game.solver();
    let step_count = (&mut solver).take(MAX_STEPS).count();

    if step_count >= MAX_STEPS {
        panic!("steps count exceeded {MAX_STEPS}");
    }

    println!("this took {step_count} steps");
    serde_json::to_string(&solver.solutions).unwrap()
}

fn test_perms(len: usize, piece: Vec<Axial>) {
    println!("===============================================================");
    println!("    {piece:?}");
    let permutations = piece_permutations(piece);

    for (i, perm) in permutations.iter().enumerate() {
        println!("{i:2}: {perm:?}");
    }

    assert_eq!(len, permutations.len());
}

#[test]
fn permutations() {
    test_perms(12, vec![Axial(0, 0), Axial(1, 0), Axial(2, 0), Axial(2, 1)]);
    test_perms(3, vec![Axial(0, 0), Axial(1, 0), Axial(0, 1), Axial(1, 1)]);
    test_perms(3, vec![Axial(0, 0), Axial(1, 0), Axial(2, 0)]);
    test_perms(3, vec![Axial(0, 0), Axial(1, 0)]);
    test_perms(1, vec![Axial(0, 0)]);
}

#[test]
fn canon_simple() {
    let mut a = vec![Axial(0, 0), Axial(1, 0), Axial(0, 1), Axial(1, 1)];
    let mut b = vec![Axial(0, 1), Axial(1, 1), Axial(0, 0), Axial(1, 0)];
    canonicalize_shape(&mut a);
    canonicalize_shape(&mut b);
    assert_eq!(a, b);
}

#[test]
fn b4() {
    let json = test_file("b4");
    let expected = expect!["[[[[3,2],[0,3],[1,3],[2,3]],[[0,0],[1,0],[2,0],[0,1],[1,1]],[[4,0],[3,1],[4,1],[4,2],[3,3]],[[3,0],[2,1],[0,2],[1,2],[2,2]]]]"];
    expected.assert_eq(&json);
}

#[test]
fn b38y() {
    let json = test_file("b38y");
    let expected = expect!["[[[[-2,4],[-3,5],[-2,5],[-3,6]],[[0,0],[-1,1],[-2,2],[-1,2]],[[0,1],[0,2],[0,3],[-1,3],[-1,4]],[[-3,3],[-2,3],[-3,4],[-4,5],[-4,6]]],[[[-4,5],[-3,5],[-4,6],[-3,6]],[[0,1],[0,2],[0,3],[-1,3]],[[0,0],[-1,1],[-2,2],[-1,2],[-2,3]],[[-3,3],[-3,4],[-2,4],[-1,4],[-2,5]]],[[[-4,5],[-3,5],[-4,6],[-3,6]],[[0,0],[0,1],[-1,1],[0,2]],[[-2,2],[-1,2],[0,3],[-2,3],[-1,3]],[[-3,3],[-3,4],[-2,4],[-1,4],[-2,5]]],[[[-4,5],[-3,5],[-4,6],[-3,6]],[[0,0],[0,1],[-1,1],[0,2]],[[-3,3],[-2,3],[-3,4],[-2,4],[-2,5]],[[-2,2],[-1,2],[0,3],[-1,3],[-1,4]]]]"];
    expected.assert_eq(&json);
}
