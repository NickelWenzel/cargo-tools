# Commands Reference

All commands are in the **Cargo Tools** category and can be invoked from the Command Palette (`Ctrl+Shift+P`) unless noted otherwise. Commands marked *context menu only* are available exclusively via tree view context menus or inline buttons.

## Quick Actions

| Command ID | Title | Default Keybinding | Description |
|-----------|-------|--------------------|-------------|
| `cargo-tools.projectStatus.build` | Build | `F7` | Build the active target with the current profile, package, platform target, and features |
| `cargo-tools.projectStatus.run` | Run | `Ctrl+Shift+F5` | Run the active run target |
| `cargo-tools.projectStatus.debug` | Debug | `Shift+F5` | Start a debug session for the active run target |
| `cargo-tools.projectStatus.test` | Test | — | Run tests for the selected package |
| `cargo-tools.projectStatus.bench` | Benchmark | — | Run benchmarks for the selected benchmark target |

## Configuration Commands

| Command ID | Title | Description |
|-----------|-------|-------------|
| `cargo-tools.selectProfile` | Select Build Profile | Choose the active cargo build profile |
| `cargo-tools.selectPackage` | Select Package | Choose the active workspace member |
| `cargo-tools.selectBuildTarget` | Select Build Target | Choose the target used for build operations |
| `cargo-tools.selectRunTarget` | Select Run Target | Choose the target used for run and debug operations |
| `cargo-tools.selectBenchmarkTarget` | Select Benchmark Target | Choose the target used for benchmark operations |
| `cargo-tools.selectPlatformTarget` | Select Platform Target | Choose the compilation target triple |
| `cargo-tools.installPlatformTarget` | Install Platform Target | Install the selected platform target via `rustup target add` |
| `cargo-tools.setRustAnalyzerCheckTargets` | Set rust-analyzer check targets | Configure target platforms for rust-analyzer analysis |
| `cargo-tools.selectFeatures` | Select Features | Enable or disable cargo features for the active package |
| `cargo-tools.refresh` | Refresh | Reload workspace metadata |
| `cargo-tools.clean` | Clean Build Artifacts | Run `cargo clean` |
| `cargo-tools.buildDocs` | Build Documentation | Run `cargo doc` with current configuration |

## Project Outline Commands

### Workspace member actions *(context menu only)*

| Command ID | Title | Description |
|-----------|-------|-------------|
| `cargo-tools.projectOutline.selectPackage` | Select Package | Set this workspace member as the active package |
| `cargo-tools.projectOutline.unselectPackage` | Unselect Package | Remove the active package selection |
| `cargo-tools.projectOutline.buildPackage` | Build Package | `cargo build -p <package>` |
| `cargo-tools.projectOutline.testPackage` | Test Package | `cargo test -p <package>` |
| `cargo-tools.projectOutline.cleanPackage` | Clean Package | `cargo clean -p <package>` |

### Workspace root actions *(context menu only)*

| Command ID | Title | Description |
|-----------|-------|-------------|
| `cargo-tools.projectOutline.buildWorkspace` | Build Workspace | `cargo build --workspace` |
| `cargo-tools.projectOutline.testWorkspace` | Test Workspace | `cargo test --workspace` |
| `cargo-tools.projectOutline.cleanWorkspace` | Clean Workspace | `cargo clean` |

### Target selection *(context menu only)*

| Command ID | Title | Description |
|-----------|-------|-------------|
| `cargo-tools.projectOutline.setBuildTarget` | Set as Build Target | Use this target for build operations |
| `cargo-tools.projectOutline.unsetBuildTarget` | Unset Build Target | Clear the build target selection |
| `cargo-tools.projectOutline.setRunTarget` | Set as Run Target | Use this target for run and debug operations |
| `cargo-tools.projectOutline.unsetRunTarget` | Unset Run Target | Clear the run target selection |
| `cargo-tools.projectOutline.setBenchmarkTarget` | Set as Benchmark Target | Use this target for benchmark operations |
| `cargo-tools.projectOutline.unsetBenchmarkTarget` | Unset Benchmark Target | Clear the benchmark target selection |

### Target actions *(context menu only)*

| Command ID | Title | Description |
|-----------|-------|-------------|
| `cargo-tools.projectOutline.buildTarget` | Build Target | Build this specific target |
| `cargo-tools.projectOutline.runTarget` | Run Target | Run this executable target |
| `cargo-tools.projectOutline.debugTarget` | Debug Target | Start a debug session for this target |
| `cargo-tools.projectOutline.benchTarget` | Benchmark Target | Run benchmarks for this target |

### View controls

| Command ID | Title | Description |
|-----------|-------|-------------|
| `cargo-tools.projectOutline.setWorkspaceMemberFilter` | Filter Workspace Members | Show only selected workspace members |
| `cargo-tools.projectOutline.editWorkspaceMemberFilter` | Edit Member Filter | Modify the active workspace member filter |
| `cargo-tools.projectOutline.showTargetTypeFilter` | Filter Target Types | Show only selected target types |
| `cargo-tools.projectOutline.clearAllFilters` | Clear All Filters | Remove all active filters |
| `cargo-tools.projectOutline.toggleWorkspaceMemberGrouping` | Toggle Workspace Member Grouping | Switch between grouping by workspace member or target type |

## cargo-make Commands

| Command ID | Title | Default Keybinding | Description |
|-----------|-------|--------------------|-------------|
| `cargo-tools.makefile.selectAndRunTask` | Run Makefile Task... | — | Pick and run a task from the Command Palette |
| `cargo-tools.makefile.runTask` | Run Makefile Task | — | Run the selected task *(context menu only)* |
| `cargo-tools.makefile.pinTask` | Pin Task | — | Add the selected task to the Pinned Makefile Tasks view *(context menu only)* |
| `cargo-tools.makefile.selectTaskFilter` | Filter Tasks | — | Filter tasks by name |
| `cargo-tools.makefile.selectCategoryFilter` | Filter Categories | — | Filter tasks by category |
| `cargo-tools.makefile.clearAllFilters` | Clear Filters | — | Remove all active task filters |
| `cargo-tools.pinnedMakefileTasks.add` | Add Task | — | Add a task to the pinned list |
| `cargo-tools.pinnedMakefileTasks.execute` | Execute Task | — | Run the selected pinned task *(context menu only)* |
| `cargo-tools.pinnedMakefileTasks.remove` | Remove Task | — | Remove a task from the pinned list *(context menu only)* |
| `cargo-tools.pinnedMakefileTasks.execute1` | Execute 1st Pinned Task | `Ctrl+Alt+1` | Run the 1st pinned task |
| `cargo-tools.pinnedMakefileTasks.execute2` | Execute 2nd Pinned Task | `Ctrl+Alt+2` | Run the 2nd pinned task |
| `cargo-tools.pinnedMakefileTasks.execute3` | Execute 3rd Pinned Task | `Ctrl+Alt+3` | Run the 3rd pinned task |
| `cargo-tools.pinnedMakefileTasks.execute4` | Execute 4th Pinned Task | `Ctrl+Alt+4` | Run the 4th pinned task |
| `cargo-tools.pinnedMakefileTasks.execute5` | Execute 5th Pinned Task | `Ctrl+Alt+5` | Run the 5th pinned task |
