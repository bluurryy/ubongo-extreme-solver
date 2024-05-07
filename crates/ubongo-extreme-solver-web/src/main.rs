mod prelude {
    pub fn default<T: Default>() -> T {
        Default::default()
    }

    pub use sycamore::prelude::*;
    #[allow(unused_imports)]
    pub use tracing::{debug, error, info, trace, warn};
    pub type View = sycamore::view::View<DomNode>;
    pub use crate::sycamore_ext::*;
    pub use glam::*;
}

mod components;
use components::App;

mod sycamore_ext;

mod bindings;

#[cfg(test)]
mod tests;

use prelude::*;

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");
    info!(what=%"World", "Hello!");
    sycamore::render(App);
}
