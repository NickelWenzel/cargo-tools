use cargo_tools_vscode::commands::{cargo, cargo_make, pinned};

fn all_cargo_commands() -> [&'static str; cargo::NUMBER_CMDS] {
    use cargo_tools_vscode::commands::cargo::*;
    [
        CARGO_TOOLS_SELECT_PROFILE,
        CARGO_TOOLS_SELECT_PACKAGE,
        CARGO_TOOLS_SELECT_BUILD_TARGET,
        CARGO_TOOLS_SELECT_RUN_TARGET,
        CARGO_TOOLS_SELECT_BENCHMARK_TARGET,
        CARGO_TOOLS_SELECT_PLATFORM_TARGET,
        CARGO_TOOLS_INSTALL_PLATFORM_TARGET,
        CARGO_TOOLS_SET_RUST_ANALYZER_CHECK_TARGETS,
        CARGO_TOOLS_BUILD_DOCS,
        CARGO_TOOLS_SELECT_FEATURES,
        CARGO_TOOLS_REFRESH,
        CARGO_TOOLS_CLEAN,
        CARGO_TOOLS_PROJECT_STATUS_BUILD,
        CARGO_TOOLS_PROJECT_STATUS_RUN,
        CARGO_TOOLS_PROJECT_STATUS_DEBUG,
        CARGO_TOOLS_PROJECT_STATUS_TEST,
        CARGO_TOOLS_PROJECT_STATUS_BENCH,
        CARGO_TOOLS_PROJECT_STATUS_TOGGLE_FEATURE,
        CARGO_TOOLS_PROJECT_OUTLINE_SELECT_PACKAGE,
        CARGO_TOOLS_PROJECT_OUTLINE_UNSELECT_PACKAGE,
        CARGO_TOOLS_PROJECT_OUTLINE_SET_BUILD_TARGET,
        CARGO_TOOLS_PROJECT_OUTLINE_UNSET_BUILD_TARGET,
        CARGO_TOOLS_PROJECT_OUTLINE_SET_RUN_TARGET,
        CARGO_TOOLS_PROJECT_OUTLINE_UNSET_RUN_TARGET,
        CARGO_TOOLS_PROJECT_OUTLINE_SET_BENCHMARK_TARGET,
        CARGO_TOOLS_PROJECT_OUTLINE_UNSET_BENCHMARK_TARGET,
        CARGO_TOOLS_PROJECT_OUTLINE_BUILD_WORKSPACE,
        CARGO_TOOLS_PROJECT_OUTLINE_TEST_WORKSPACE,
        CARGO_TOOLS_PROJECT_OUTLINE_CLEAN_WORKSPACE,
        CARGO_TOOLS_PROJECT_OUTLINE_BUILD_PACKAGE,
        CARGO_TOOLS_PROJECT_OUTLINE_TEST_PACKAGE,
        CARGO_TOOLS_PROJECT_OUTLINE_CLEAN_PACKAGE,
        CARGO_TOOLS_PROJECT_OUTLINE_BUILD_TARGET,
        CARGO_TOOLS_PROJECT_OUTLINE_RUN_TARGET,
        CARGO_TOOLS_PROJECT_OUTLINE_DEBUG_TARGET,
        CARGO_TOOLS_PROJECT_OUTLINE_BENCH_TARGET,
        CARGO_TOOLS_PROJECT_OUTLINE_SET_WORKSPACE_MEMBER_FILTER,
        CARGO_TOOLS_PROJECT_OUTLINE_EDIT_WORKSPACE_MEMBER_FILTER,
        CARGO_TOOLS_PROJECT_OUTLINE_SHOW_TARGET_TYPE_FILTER,
        CARGO_TOOLS_PROJECT_OUTLINE_EDIT_TARGET_TYPE_FILTER,
        CARGO_TOOLS_PROJECT_OUTLINE_CLEAR_ALL_FILTERS,
        CARGO_TOOLS_PROJECT_OUTLINE_TOGGLE_WORKSPACE_MEMBER_GROUPING,
        CARGO_TOOLS_PROJECT_OUTLINE_TOGGLE_FEATURE,
    ]
}

const fn all_cargo_make_commands() -> [&'static str; cargo_make::NUMBER_CMDS] {
    use cargo_tools_vscode::commands::cargo_make::*;
    [
        CARGO_TOOLS_MAKEFILE_RUNTASK,
        CARGO_TOOLS_MAKEFILE_SELECTANDRUNTASK,
        CARGO_TOOLS_MAKEFILE_SELECTTASKFILTER,
        CARGO_TOOLS_MAKEFILE_SELECTCATEGORYFILTER,
        CARGO_TOOLS_MAKEFILE_CLEARALLFILTERS,
        CARGO_TOOLS_MAKEFILE_PINTASK,
    ]
}

const fn all_pinned_commands() -> [&'static str; pinned::NUMBER_CMDS] {
    use cargo_tools_vscode::commands::pinned::*;
    [
        CARGO_TOOLS_PINNED_ADD,
        CARGO_TOOLS_PINNED_REMOVE,
        CARGO_TOOLS_PINNED_EXECUTE,
        CARGO_TOOLS_PINNED_EXECUTE1,
        CARGO_TOOLS_PINNED_EXECUTE2,
        CARGO_TOOLS_PINNED_EXECUTE3,
        CARGO_TOOLS_PINNED_EXECUTE4,
        CARGO_TOOLS_PINNED_EXECUTE5,
    ]
}

fn all_cargo_commands_from_cargo_tools() -> Vec<&'static str> {
    let mut cmds = all_cargo_commands().into_iter().collect::<Vec<_>>();
    cmds.extend(all_cargo_make_commands());
    cmds.extend(all_pinned_commands());
    cmds
}

fn all_commands_from_package_json() -> Vec<String> {
    let package_json = include_str!("../../../package.json");
    let json: serde_json::Value =
        serde_json::from_str(package_json).expect("Failed to parse package.json");

    json["contributes"]["commands"]
        .as_array()
        .expect("commands should be an array")
        .iter()
        .filter_map(|cmd| cmd["command"].as_str().map(|s| s.to_string()))
        .collect()
}

#[test]
fn all_commands_are_registered() {
    let cargo_commands_from_cargo_tools = all_cargo_commands_from_cargo_tools();
    let commands_from_package_json = all_commands_from_package_json();

    for cmd in commands_from_package_json {
        assert!(
            cargo_commands_from_cargo_tools.contains(&cmd.as_str()),
            "Command '{cmd}' from all_commands() was not registered."
        );
    }
}
