use cargo_tools::cargo::command::{BuildSubTarget, RunSubTarget};
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct Icon {
    icon: &'static str,
    color: &'static str,
}

#[wasm_bindgen]
impl Icon {
    #[wasm_bindgen(getter)]
    pub fn icon(&self) -> String {
        self.icon.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn color(&self) -> String {
        self.color.to_string()
    }
}

// Core Cargo concepts with distinct chart colors
pub const PROJECT: Icon = Icon {
    icon: "repo",
    color: "charts.blue",
};
pub const PACKAGE: Icon = Icon {
    icon: "package",
    color: "charts.orange",
};
pub const WORKSPACE: Icon = Icon {
    icon: "organization",
    color: "charts.purple",
};

// Target types with vibrant, distinguishable colors
pub const BIN_TARGET: Icon = Icon {
    icon: "file-binary",
    color: "charts.green",
};
pub const LIB_TARGET: Icon = Icon {
    icon: "library",
    color: "charts.blue",
};
pub const EXAMPLE_TARGET: Icon = Icon {
    icon: "lightbulb",
    color: "charts.yellow",
};
pub const TEST_TARGET: Icon = Icon {
    icon: "beaker",
    color: "charts.purple",
};
pub const BENCH_TARGET: Icon = Icon {
    icon: "dashboard",
    color: "charts.red",
};
pub const UNKNOWN_TARGET: Icon = Icon {
    icon: "file",
    color: "charts.foreground",
};

// Actions with consistent, vibrant color coding
pub const BUILD_ACTION: Icon = Icon {
    icon: "tools",
    color: "charts.blue",
};
pub const RUN_ACTION: Icon = Icon {
    icon: "play",
    color: "charts.green",
};
pub const DEBUG_ACTION: Icon = Icon {
    icon: "debug-alt",
    color: "charts.orange",
};
pub const TEST_ACTION: Icon = Icon {
    icon: "beaker",
    color: "charts.purple",
};
pub const BENCH_ACTION: Icon = Icon {
    icon: "dashboard",
    color: "charts.red",
};
pub const CLEAN_ACTION: Icon = Icon {
    icon: "trash",
    color: "charts.red",
};

// Configuration and settings with chart colors
pub const PROFILE_CONFIG: Icon = Icon {
    icon: "settings-gear",
    color: "charts.yellow",
};
pub const PLATFORM_CONFIG: Icon = Icon {
    icon: "device-desktop",
    color: "charts.cyan",
};
pub const TARGET_CONFIG: Icon = Icon {
    icon: "target",
    color: "charts.green",
};
pub const FEATURES_CONFIG: Icon = Icon {
    icon: "symbol-misc",
    color: "charts.purple",
};

// States and status with meaningful chart colors
pub const SELECTED_STATE: Icon = Icon {
    icon: "check",
    color: "charts.green",
};
pub const UNSELECTED_STATE: Icon = Icon {
    icon: "circle-outline",
    color: "charts.foreground",
};
pub const WARNING_STATE: Icon = Icon {
    icon: "warning",
    color: "charts.yellow",
};
pub const ERROR_STATE: Icon = Icon {
    icon: "error",
    color: "charts.red",
};

// Makefile specific with chart colors
pub const MAKEFILE_CATEGORY: Icon = Icon {
    icon: "folder",
    color: "charts.cyan",
};
pub const MAKEFILE_TASK: Icon = Icon {
    icon: "gear",
    color: "charts.blue",
};

// Utility actions with chart color distinctions
pub const REFRESH_ACTION: Icon = Icon {
    icon: "refresh",
    color: "charts.foreground",
};
pub const ADD_ACTION: Icon = Icon {
    icon: "add",
    color: "charts.green",
};
pub const REMOVE_ACTION: Icon = Icon {
    icon: "remove",
    color: "charts.red",
};
pub const EDIT_ACTION: Icon = Icon {
    icon: "edit",
    color: "charts.orange",
};
pub const PIN_ACTION: Icon = Icon {
    icon: "pin",
    color: "charts.yellow",
};
pub const UNPIN_ACTION: Icon = Icon {
    icon: "pinned",
    color: "charts.foreground",
};

// Documentation and info with chart colors
pub const DOCS_ACTION: Icon = Icon {
    icon: "book",
    color: "charts.cyan",
};
pub const FILTER_ACTION: Icon = Icon {
    icon: "filter",
    color: "charts.purple",
};
pub const LIST_ACTION: Icon = Icon {
    icon: "list-unordered",
    color: "charts.foreground",
};

impl Icon {
    pub fn build_target(build_target: &Option<BuildSubTarget>) -> Self {
        let Some(build_target) = build_target else {
            return TARGET_CONFIG;
        };

        match build_target {
            BuildSubTarget::Bin(_) => RUN_ACTION,
            BuildSubTarget::Example(_) => EXAMPLE_TARGET,
            BuildSubTarget::Lib(_) => LIB_TARGET,
            BuildSubTarget::Bench(_) => BENCH_TARGET,
        }
    }

    pub fn run_target(build_target: &Option<RunSubTarget>) -> Self {
        let Some(build_target) = build_target else {
            return TARGET_CONFIG;
        };

        match build_target {
            RunSubTarget::Bin(_) => RUN_ACTION,
            RunSubTarget::Example(_) => EXAMPLE_TARGET,
        }
    }
}
