use wasm_bindgen::prelude::*;
use web_sys::Element;

#[allow(non_local_definitions)]
mod raw {
    use super::*;

    #[wasm_bindgen(module = "/src/bindings/resize_end.js")]
    extern "C" {
        #[derive(Debug, Clone)]
        pub type State;

        pub fn create(timeout: f64, container: &Element, callback: &Closure<dyn FnMut(f64, f64)>) -> State;
        pub fn destroy(state: &State);
    }
}

pub struct Listener {
    inner: raw::State,
    #[allow(dead_code)]
    callback: Closure<dyn FnMut(f64, f64)>,
}

impl Listener {
    pub fn new(timeout: f64, container: &Element, callback: impl FnMut(f64, f64) + 'static) -> Self {
        let callback = Closure::new(callback);
        let inner = raw::create(timeout, container, &callback);
        Self { inner, callback }
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        raw::destroy(&self.inner);
    }
}
