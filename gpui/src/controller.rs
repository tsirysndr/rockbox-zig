use crate::state::AppState;
use gpui::{Entity, Global};

pub struct Controller {
    pub state: Entity<AppState>,
}

impl Controller {
    pub fn new(state: Entity<AppState>) -> Self {
        Controller { state }
    }
}

impl Global for Controller {}
