use std::path::Path;

use criterion::{criterion_group, criterion_main, Criterion};
use ubongo_extreme_solver::Game;

fn test_file(name: &str) -> Game {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("data/{name}.json"));
    let content = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&content).unwrap()
}

macro_rules! files {
	($($name:ident)*) => {
		$(
			fn $name(c: &mut Criterion) {
				let game = test_file(stringify!($name));

				c.bench_function(stringify!($name), |b| {
					b.iter(|| {
						let mut solver = game.clone().solver();
						while solver.next().is_some() { }
						solver
					})
				});
			}
		)*
	};
}

files!(b4 b38y);

criterion_group!(benches, b4, b38y);
criterion_main!(benches);
