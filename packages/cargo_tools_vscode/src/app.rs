pub mod cargo_make_ui;
pub mod cargo_settings_ui;

use cargo_tools::app::AppServices;

use crate::runtime::VsCodeRuntime;

pub struct VsCodeAppServices;

impl AppServices for VsCodeAppServices {
    type RuntimeT = VsCodeRuntime;
}
