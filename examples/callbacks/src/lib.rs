/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

// SIM cards.
pub trait SimCard: Send + Sync {
    fn name(&self) -> String;
}

struct RustySim {}
impl SimCard for RustySim {
    fn name(&self) -> String {
        "rusty!".to_string()
    }
}

// namespace functions.
fn get_sim_cards() -> Vec<Arc<dyn SimCard>> {
    vec![Arc::new(RustySim {})]
}

// A trait for the foreign callback.
// TODO: pass the SimCard.
pub trait OnCallAnswered {
    fn hello(&self, sim: Arc<dyn SimCard>) -> String;
    fn busy(&self, sim: Arc<dyn SimCard>);
    fn text_received(&self, sim: Arc<dyn SimCard>, text: String);
}

#[derive(Debug, Clone)]
struct Telephone;
impl Telephone {
    fn new() -> Self {
        Telephone
    }
    fn call(&self, sim: Arc<dyn SimCard>, domestic: bool, call_responder: Box<dyn OnCallAnswered>) {
        if domestic {
            let _ = call_responder.hello(sim.clone());
        } else {
            call_responder.busy(sim.clone());
            call_responder.text_received(sim.clone(), "Not now, I'm on another call!".into());
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/callbacks.uniffi.rs"));
