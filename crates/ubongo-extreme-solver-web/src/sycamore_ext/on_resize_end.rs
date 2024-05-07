use std::any::Any;
use sycamore::prelude::*;

use wasm_bindgen::JsCast;
use web_sys::Element;

type ResizeListener = crate::bindings::resize_end::Listener;

pub fn on_resize_end_callback<G: GenericNode>(node: &G, timeout: f64, f: impl FnMut(f64, f64) + 'static) {
    let node = match (node as &dyn Any).downcast_ref::<DomNode>() {
        Some(node) => node.to_web_sys(),
        None => panic!("aaah"),
    };

    let element: Element = node.dyn_into().expect("`on_resize_end` requires an `Element` not just any `Node`");
    create_signal(ResizeListener::new(timeout, &element, f));
}

pub fn on_resize_end<G: GenericNode>(node: &G, timeout: f64) -> Signal<[f64; 2]> {
    let signal = create_signal([f64::NAN; 2]);
    on_resize_end_callback(node, timeout, move |x, y| signal.set([x, y]));
    signal
}
