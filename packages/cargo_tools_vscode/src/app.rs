pub mod cargo_make_ui;
pub mod cargo_settings_ui;

use cargo_make_ui::VsCodeCargoMakeUi;
use cargo_settings_ui::VsCodeCargoSettingsUi;
use cargo_tools::app::AppServices;

use crate::context::VsCodeContext;
use crate::runtime::VsCodeRuntime;

pub struct VsCodeAppServices;

impl AppServices for VsCodeAppServices {
    type RuntimeT = VsCodeRuntime;
    type ContextT = VsCodeContext;
    type CargoMakeUiT = VsCodeCargoMakeUi;
    type CargoSettingsUiT = VsCodeCargoSettingsUi;
}
