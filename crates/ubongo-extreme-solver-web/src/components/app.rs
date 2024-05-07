use crate::prelude::default;
use std::collections::BTreeMap;

use chrono::Duration;
use gloo::timers::future::sleep;
use indexmap::{indexmap, IndexMap};
use sycamore::{generic_node::GenericNodeElements, web::html};
use ubongo_extreme_solver::{canonicalize_place, Axial, Game, Solver};

use crate::prelude::*;

macro_rules! examples {
    ($($name:literal),*) => {
        indexmap! {
            $(
                $name => include_str!(concat!("../../../ubongo-extreme-solver/data/", $name, ".json")),
            )*
        }
    };
}

#[derive(Props)]
pub struct PropertyProps {
    pub label: &'static str,
    pub close: ReadSignal<bool>,
    pub children: Children<DomNode>,
}

#[component]
pub fn Property(props: PropertyProps) -> View {
    let PropertyProps { label, close, children } = props;
    let children = children.call();

    view! {
        li(class=if close.get() { "hidden" } else { "" }) {
            span(class="label") { (label) }
            div(class="value") { (children) }
        }
    }
}

#[derive(Props)]
pub struct TextEditProps {
    pub value: Signal<String>,
    pub attributes: Attributes<DomNode>,
}

#[component]
pub fn TextEdit(props: TextEditProps) -> View {
    const MIN_ROWS: usize = 2;
    const MIN_COLS: usize = 25;

    let TextEditProps { value, attributes } = props;

    let rows = create_selector(move || value.with(|v| v.lines().count().max(MIN_ROWS).to_string()));
    let cols = create_selector(move || value.with(|v| v.lines().map(|line| line.len()).max().unwrap_or(MIN_COLS).max(MIN_COLS).to_string()));

    view! {
        textarea(
            rows=rows,
            cols=cols,
            bind:value=value,
            ..attributes
        )
    }
}

// "M 0 1 L -0.8660254037844386 0.5000000000000001 L -0.8660254037844387 -0.4999999999999998 L -1.2246467991473532e-16 -1 L 0.8660254037844384 -0.5000000000000004 L 0.866025403784439 0.49999999999999933 Z"
fn unit_hexagon_path() -> String {
    fn rotate([x, y]: [f64; 2], angle: f64) -> [f64; 2] {
        [x * angle.cos() - y * angle.sin(), x * angle.sin() + y * angle.cos()]
    }

    use std::fmt::Write;
    let mut s = String::new();

    for i in 0..6 {
        let angle = i as f64 * (std::f64::consts::TAU / 6.0);
        let [x, y] = rotate([0.0, 1.0], angle);
        let cmd = if i == 0 { 'M' } else { 'L' };
        write!(s, "{cmd} {x} {y} ").unwrap();
    }

    s.push('Z');
    s
}

fn axial_to_cartesian(Axial(x, y): Axial) -> DVec2 {
    const SQRT_3: f64 = 1.7320508075688772;
    let x = x as f64;
    let y = y as f64;
    dvec2(SQRT_3 * x + (SQRT_3 / 2.0) * y, (3.0 / 2.0) * y)
}

fn hexagon(pos: DVec2, color: &'static str) -> View {
    use sycamore::builder::prelude::*;

    r#use()
        .attr("href", "#hexagon")
        .attr("fill", color)
        .attr("x", pos.x.to_string())
        .attr("y", pos.y.to_string())
        .view()

    // view! {
    //   r#use(href="#hexagon", fill=color, x=pos.x, y=pos.y)
    // }
}

fn hexagon_dyn(pos: DVec2, color: Signal<&'static str>) -> View {
    use sycamore::builder::prelude::*;

    r#use()
        .attr("href", "#hexagon")
        .dyn_attr("fill", move || Some(color.get()))
        .attr("x", pos.x.to_string())
        .attr("y", pos.y.to_string())
        .view()

    // view! {
    //   r#use(href="#hexagon", fill=color, x=pos.x, y=pos.y)
    // }
}

pub struct SolverTracedSignals {
    is_done: Signal<bool>,
    steps: Signal<usize>,
    solutions_len: Signal<usize>,
    duration: Signal<Duration>,
    boards_views: Signal<Vec<View>>,
    pieces_views: Signal<Vec<View>>,
    item_size: Signal<[f64; 2]>,
    item_aspect: Signal<f64>,
    working_piece_fill_color: Signal<&'static str>,
}

pub struct SolverTraced {
    solver: Solver,
    steps: usize,
    is_done: bool,
    duration: chrono::Duration,
    viewbox: String,
    solutions: usize,
    boards_colors_signals: Vec<Vec<Signal<&'static str>>>,
    board_color_buffer: BTreeMap<u64, &'static str>,
    signals: SolverTracedSignals,
}

impl SolverTraced {
    pub fn new(signals: SolverTracedSignals) -> Self {
        Self {
            solver: Solver::new(default()),
            duration: Duration::zero(),
            boards_colors_signals: Vec::new(),
            board_color_buffer: default(),
            steps: 0,
            solutions: 0,
            is_done: false,
            viewbox: default(),
            signals,
        }
    }

    pub fn set_game(&mut self, mut game: Game) {
        canonicalize_place(&mut game.board);
        let mut aabb = Aabb::from_points_axial(game.board.iter().copied());
        aabb = aabb.expand(PADDING);

        self.viewbox = aabb.viewbox();
        self.board_color_buffer = BTreeMap::new();

        for &coord in &game.board {
            self.board_color_buffer.insert(coord.key(), COLOR_DEFAULT);
        }

        let mut pieces = vec![];
        let mut pieces_aabb = Aabb::EMPTY;

        for piece in &game.pieces {
            for &coord in piece {
                let point = axial_to_cartesian(coord);
                pieces_aabb = pieces_aabb.expand_to(point);
            }
        }

        let pieces_aabb_size = pieces_aabb.expand(PADDING).size();

        for (piece_i, piece) in game.pieces.iter().enumerate() {
            let mut hexagons = vec![];
            let mut aabb = Aabb::EMPTY;

            for &coord in piece {
                let point = axial_to_cartesian(coord);
                aabb = aabb.expand_to(point);
                let color = COLORS[piece_i % COLORS.len()];
                hexagons.push(hexagon(point, color));
            }

            let aabb = Aabb::from_origin_size(aabb.origin(), pieces_aabb_size);

            use sycamore::builder::prelude::*;

            let view = div().c(svg().attr("viewBox", aabb.viewbox()).c(View::new_fragment(hexagons))).view();

            pieces.push(view);
        }

        self.solver = Solver::new(game);
        self.duration = Duration::zero();
        self.boards_colors_signals = Vec::new();
        self.steps = 0;
        self.solutions = 0;
        self.is_done = false;
        self.signals.is_done.set(false);
        self.signals.boards_views.take();
        self.signals.pieces_views.set(pieces);
        self.signals.duration.set(Duration::zero());
        self.signals.steps.set(0);
        self.signals.solutions_len.set(0);
        self.signals.item_aspect.set(aabb.aspect());
        self.something_changed();
    }

    fn clear_board_color_buffer(&mut self) {
        for value in self.board_color_buffer.values_mut() {
            *value = COLOR_DEFAULT;
        }
    }

    fn apply_board_color_buffer(&mut self, i: usize) {
        let signals = &self.boards_colors_signals[i];

        for (color, signal) in self.board_color_buffer.values().copied().zip(signals) {
            if !std::ptr::eq(color, signal.get_untracked()) {
                signal.set(color);
            }
        }
    }

    fn make_board(&self, hexagons: Vec<View>) -> View {
        use sycamore::builder::prelude::*;

        let size = self.signals.item_size;
        let style = move || {
            let [width, height] = size.get();
            Some(format!("min-width:{width}px;min-height:{height}px;max-width:{width}px;max-height:{height}px"))
        };

        div()
            .dyn_attr("style", style)
            .c(svg().attr("viewBox", self.viewbox.clone()).c(View::new_fragment(hexagons)))
            .view()
    }

    fn something_changed(&mut self) {
        self.signals.duration.set(self.duration);
        self.signals.steps.set(self.steps);
        self.signals.solutions_len.set(self.solver.solutions.len());
        self.signals.is_done.set(self.is_done);

        let mut board_count = self.solver.solutions.len();

        if !self.is_done {
            self.steps += 1;
            board_count += 1;
        }

        while board_count > self.boards_colors_signals.len() {
            let mut hexagons = vec![];
            let mut colors = Vec::new();

            for &coord in &self.solver.game.board {
                let signal = create_signal(COLOR_DEFAULT);
                hexagons.push(hexagon_dyn(axial_to_cartesian(coord), signal));
                colors.push(signal);
            }

            self.boards_colors_signals.push(colors);
            self.signals.boards_views.update(|v| v.push(self.make_board(hexagons)));
        }

        while board_count < self.boards_colors_signals.len() {
            self.boards_colors_signals.pop();
            self.signals.boards_views.update(|v| v.pop());
        }

        while self.solutions < self.solver.solutions.len() {
            self.clear_board_color_buffer();

            for (piece_i, piece) in self.solver.solutions[self.solutions].iter().enumerate() {
                for coord in piece {
                    if let Some(color) = self.board_color_buffer.get_mut(&coord.key()) {
                        *color = COLORS[piece_i % COLORS.len()];
                    }
                }
            }

            self.apply_board_color_buffer(self.solutions);
            self.solutions += 1;
        }

        if !self.is_done {
            self.clear_board_color_buffer();

            for (piece_i, piece) in self.solver.pieces.iter().enumerate().take(self.solver.work_idx) {
                let is_last = piece_i == self.solver.work_idx - 1;

                for coord in piece {
                    if let Some(color) = self.board_color_buffer.get_mut(&coord.key()) {
                        let fill_color = COLORS[piece_i % COLORS.len()];

                        *color = if is_last {
                            self.signals.working_piece_fill_color.set(fill_color);
                            "url(#working-piece-fill)"
                        } else {
                            fill_color
                        };
                    }
                }
            }

            self.apply_board_color_buffer(self.solutions);
        }
    }

    pub fn step(&mut self) {
        self.step_silent();
        self.something_changed();
    }

    pub fn step_silent(&mut self) {
        let start = chrono::Local::now();
        self.is_done = self.solver.next().is_none();
        let now = chrono::Local::now();
        self.duration += now - start;
    }
}

const COLORS: &[&str] = &[
    "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd", "#8c564b", "#e377c2", "#7f7f7f", "#bcbd22", "#17becf",
];
const COLOR_DEFAULT: &str = "#ccc";
const PADDING: f64 = 1.5;

#[component]
pub fn App() -> View {
    let examples: Signal<IndexMap<&'static str, &'static str>> = create_signal(examples!["b18b", "b18g", "b18r", "b18y", "b38y", "b4"]);

    let examples_keys = create_memo(move || examples.with(|v| v.keys().copied().collect::<Vec<_>>()));
    let example_key = create_signal(String::from("b38y"));
    let example_value = create_memo(move || {
        examples.with(|examples| example_key.with(|example_key| examples.get(&**example_key).map(|&v| String::from(v)).unwrap_or_default()))
    });
    let game_json = create_signal(String::new());

    create_effect(move || game_json.set(example_value.get_clone()));

    let view = create_signal(false);
    let game = create_signal(Game::default());

    create_effect(move || match game_json.with(|v| json5::from_str::<Game>(v)) {
        Ok(g) => game.set(g),
        Err(error) => error!(%error),
    });

    let steps = create_signal(0usize);
    let duration = create_signal(Duration::zero());
    let boards_views = create_signal(Default::default());
    let pieces_views = create_signal(Default::default());
    let is_done = create_signal(false);
    let item_size = create_signal([100.0f64; 2]);
    let item_aspect = create_signal(1.0);
    let solutions_len = create_signal(0);
    let solutions_is_empty = create_selector(move || solutions_len.get() == 0);
    let working_piece_fill_color = create_signal("#000");

    let solver = create_signal(SolverTraced::new(SolverTracedSignals {
        boards_views,
        pieces_views,
        is_done,
        steps,
        solutions_len,
        duration,
        item_size,
        item_aspect,
        working_piece_fill_color,
    }));

    create_effect(move || {
        let game = game.get_clone();
        solver.update(|s| s.set_game(game));
    });

    let delay = create_signal(10.0);
    let player = create_task();

    create_effect({
        let player = player.clone();
        move || {
            let is_running = player.is_running().get();
            if is_running {
                info!("RUNNING");
            } else {
                info!("STOPPING");
            }
        }
    });

    let close = create_signal(false);

    let resize_listener = DomNode::element::<html::div>();
    resize_listener.set_attribute("class".into(), "resize-listener".into());

    let item_count = create_selector(move || boards_views.with(|v| v.len()));
    let container_size = on_resize_end(&resize_listener, 100.0);

    create_effect(move || {
        let [container_width, container_height] = container_size.get();
        let item_count = item_count.get();
        let item_aspect = item_aspect.get();
        let layout = absolute_grid_layout(container_width, container_height, item_count, item_aspect, 10_000);
        debug!(?layout, container_width, container_height, item_count, item_aspect, "resized");
        item_size.set([layout.item_width.floor(), layout.item_height.floor()]);
    });

    let resize_listener = View::new_node(resize_listener);

    let step = move |_| {
        warn!("STEPPING");
        solver.update(|s| s.step());
    };

    let play = {
        let player = player.clone();
        move |_| {
            player.spawn_local(async move {
                while !solver.with(|s| s.is_done) {
                    solver.update(|s| s.step());
                    sleep(std::time::Duration::from_millis(delay.get_untracked() as u64)).await;
                }
            });
        }
    };

    let pause = {
        let player = player.clone();
        move |_| {
            player.abort();
            solver.update(|s| s.something_changed());
        }
    };

    let player = create_signal(player);

    create_effect(move || {
        let close = close.get();
        debug!("close changed to: {close}");
    });

    view! {
        svg(id="templates", viewBox="-10 -10 20 20") {
            defs {
                path(id="hexagon", d=unit_hexagon_path())
                pattern(id="working-piece-fill", height="2", width="2", patternUnits="userSpaceOnUse", patternTransform="translate(1,0) rotate(60) scale(0.17)") {
                    rect(fill=COLOR_DEFAULT, width="2", height="2")
                    rect(fill=*working_piece_fill_color.get(), width="1", height="2")
                }
            }
        }
        div(id="grid") {
            div(id="grid-boards") {
                (resize_listener)
                (View::new_fragment(boards_views.get_clone()))
            }
            div(id="grid-pieces", class=if solutions_is_empty.get() { "" } else { "hidden" }) {
                (View::new_fragment(pieces_views.get_clone()))
            }
        }
        div(id="ui") {
            ul {
                Property(label="input", close=*close) {
                    div(class="row") {
                        select(bind:value=example_key) {
                            Keyed(
                                iterable=examples_keys,
                                key=|name| *name,
                                view=|name|  {
                                    view! {
                                        option(value=name) { (name) }
                                    }
                                },
                            )
                        }
                        button(on:click=move |_| view.set(!view.get_untracked())) {
                            (if view.get() { "Hide" } else { "View & Edit" })
                        }
                        button(on:click=move |_| example_key.update(|_| ()), disabled=example_value.with(|a| game_json.with(|b| a == b)) ) {
                            "⟲"
                        }
                    }
                    TextEdit(
                        value=game_json,
                        attr:class=if view.get() { "" } else { "hidden" },
                    )
                }
                Property(label="controls", close=*close) {
                    div(class="row") {
                        button(
                            disabled=player.with(|s| s.is_running()).get() || is_done.get(),
                            on:click=step,
                        ) { "Step" }
                        button(
                            disabled=player.with(|s| s.is_running()).get() || is_done.get(),
                            on:click=play,
                        ) { "Play" }
                        button(
                            disabled=!player.with(|s| s.is_running()).get() || is_done.get(),
                            on:click=pause,
                        ) { "Pause" }
                        button(on:click=move |_| game.update(|_| ())) {
                            "⟲"
                        }
                    }
                }
                Property(label="delay", close=*close) {
                    div(class="row") {
                        input(type="number", min="0", bind:valueAsNumber=delay) span { "ms" }
                    }
                }
                Property(label="steps", close=*close) {
                    div(class="row") {
                        (steps.get())
                    }
                }
                Property(label="solutions", close=*close) {
                    div(class="row") {
                        (solutions_len.get())
                    }
                }
                li(class="close-row", on:click=move |_| close.set(!close.get())) {
                    (if close.get() { "show" } else { "close" })
                }
            }
        }
        div(id="manifest") {
            (MANIFEST)
        }
    }
}

#[cfg(debug_assertions)]
const MANIFEST: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"), " (debug)");

#[cfg(not(debug_assertions))]
const MANIFEST: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AbsoluteGridLayout {
    pub rows: usize,
    pub columns: usize,
    pub item_width: f64,
    pub item_height: f64,
}

pub fn absolute_grid_layout(container_width: f64, container_height: f64, item_count: usize, item_aspect: f64, max_iterations: usize) -> AbsoluteGridLayout {
    let item_aspect_inv = 1.0 / item_aspect;

    let mut rows = 1;
    let mut columns = item_count;
    let mut item_width = container_width / columns as f64;

    for _ in 0..max_iterations {
        let next_rows = rows + 1;
        let next_columns = item_count.div_ceil(next_rows);
        let next_item_width = container_width / next_columns as f64;
        let next_item_height = next_item_width * item_aspect_inv;

        if (next_item_height * next_rows as f64) > container_height {
            break;
        }

        rows = next_rows;
        columns = next_columns;
        item_width = next_item_width;
    }

    let mut item_height;

    item_width = item_width.min(container_width);
    item_height = item_width * item_aspect_inv;

    item_height = item_height.min(container_height);
    item_width = item_height * item_aspect;

    AbsoluteGridLayout {
        rows,
        columns,
        item_width,
        item_height,
    }
}

#[derive(Debug, Clone, Copy)]
struct Aabb {
    pub min: DVec2,
    pub max: DVec2,
}

impl Aabb {
    pub const EMPTY: Self = Self {
        min: DVec2::splat(f64::INFINITY),
        max: DVec2::splat(f64::NEG_INFINITY),
    };

    #[must_use]
    pub fn origin(self) -> DVec2 {
        (self.min + self.max) * 0.5
    }

    #[must_use]
    pub fn size(self) -> DVec2 {
        self.max - self.min
    }

    #[must_use]
    pub fn from_origin_extents(origin: DVec2, extents: DVec2) -> Self {
        Self {
            min: origin - extents,
            max: origin + extents,
        }
    }

    #[must_use]
    pub fn from_origin_size(origin: DVec2, size: DVec2) -> Self {
        Self::from_origin_extents(origin, size * 0.5)
    }

    #[must_use]
    pub fn from_points(points: impl IntoIterator<Item = DVec2>) -> Self {
        let mut this = Self::EMPTY;

        for point in points {
            this = this.expand_to(point);
        }

        this
    }

    #[must_use]
    pub fn from_points_axial(points: impl IntoIterator<Item = Axial>) -> Self {
        Self::from_points(points.into_iter().map(axial_to_cartesian))
    }

    #[must_use]
    pub fn expand(mut self, value: f64) -> Self {
        self.min -= value;
        self.max += value;
        self
    }

    #[must_use]
    pub fn expand_to(mut self, point: DVec2) -> Self {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
        self
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn expand_with(mut self, other: Self) -> Self {
        self = self.expand_to(other.min);
        self = self.expand_to(other.max);
        self
    }

    #[must_use]
    pub fn aspect(self) -> f64 {
        let size = self.size();
        size.x / size.y
    }

    #[must_use]
    pub fn viewbox(self) -> String {
        let size = self.size();
        let Self { min, .. } = self;
        format!("{} {} {} {}", min.x, min.y, size.x, size.y)
    }
}
