use crate::{cargo_tools::CargoTools, vs_code_cargo_tools::state_manager::VSCodeStateManager};

mod state_manager;

pub struct VSCodeCargoTools(CargoTools<VSCodeStateManager>);
