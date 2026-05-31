# Commands Reference

All commands are in the **Cargo Tools** category and can be invoked from the Command Palette (`Ctrl+Shift+P`) unless noted otherwise. Commands marked *context menu only* are available exclusively via tree view context menus or inline buttons.

## Quick Actions

| Command ID                        | Title     | Default Keybinding | Description                                                                              |
| --------------------------------- | --------- | ------------------ | ---------------------------------------------------------------------------------------- |
| `cargo-tools.projectStatus.build` | Build     | `F7`               | Build the active target with the current profile, package, platform target, and features |
| `cargo-tools.projectStatus.run`   | Run       | `Ctrl+Shift+F5`    | Run the active run target                                                                |
| `cargo-tools.projectStatus.debug` | Debug     | `Shift+F5`         | Start a debug session for the active run target                                          |
| `cargo-tools.projectStatus.test`  | Test      | —                  | Run tests for the selected package                                                       |
| `cargo-tools.projectStatus.bench` | Benchmark | —                  | Run benchmarks for the selected benchmark target                                         |

## Configuration Commands

| Command ID                                | Title                           | Description                                                  |
| ----------------------------------------- | ------------------------------- | ------------------------------------------------------------ |
| `cargo-tools.selectProfile`               | Select Build Profile            | Choose the active cargo build profile                        |
| `cargo-tools.selectPackage`               | Select Package                  | Choose the active workspace member                           |
| `cargo-tools.selectBuildTarget`           | Select Build Target             | Choose the target used for build operations                  |
| `cargo-tools.selectRunTarget`             | Select Run Target               | Choose the target used for run and debug operations          |
| `cargo-tools.selectBenchmarkTarget`       | Select Benchmark Target         | Choose the target used for benchmark operations              |
| `cargo-tools.selectPlatformTarget`        | Select Platform Target          | Choose the compilation target triple                         |
| `cargo-tools.installPlatformTarget`       | Install Platform Target         | Install the selected platform target via `rustup target add` |
| `cargo-tools.setRustAnalyzerCheckTargets` | Set rust-analyzer check targets | Configure target platforms for rust-analyzer analysis        |
| `cargo-tools.selectFeatures`              | Select Features                 | Enable or disable cargo features for the active package      |
| `cargo-tools.refresh`                     | Refresh                         | Reload workspace metadata                                    |
| `cargo-tools.clean`                       | Clean Build Artifacts           | Run `cargo clean`                                            |
| `cargo-tools.buildDocs`                   | Build Documentation             | Run `cargo doc` with current configuration                   |

## Project Outline Commands

### Workspace member actions *(context menu only)*

| Command ID                                   | Title            | Description                                     |
| -------------------------------------------- | ---------------- | ----------------------------------------------- |
| `cargo-tools.projectOutline.selectPackage`   | Select Package   | Set this workspace member as the active package |
| `cargo-tools.projectOutline.unselectPackage` | Unselect Package | Remove the active package selection             |
| `cargo-tools.projectOutline.buildPackage`    | Build Package    | `cargo build -p <package>`                      |
| `cargo-tools.projectOutline.testPackage`     | Test Package     | `cargo test -p <package>`                       |
| `cargo-tools.projectOutline.cleanPackage`    | Clean Package    | `cargo clean -p <package>`                      |

### Workspace root actions *(context menu only)*

| Command ID                                  | Title           | Description               |
| ------------------------------------------- | --------------- | ------------------------- |
| `cargo-tools.projectOutline.buildWorkspace` | Build Workspace | `cargo build --workspace` |
| `cargo-tools.projectOutline.testWorkspace`  | Test Workspace  | `cargo test --workspace`  |
| `cargo-tools.projectOutline.cleanWorkspace` | Clean Workspace | `cargo clean`             |

### Target selection *(context menu only)*

| Command ID                                        | Title                   | Description                                  |
| ------------------------------------------------- | ----------------------- | -------------------------------------------- |
| `cargo-tools.projectOutline.setBuildTarget`       | Set as Build Target     | Use this target for build operations         |
| `cargo-tools.projectOutline.unsetBuildTarget`     | Unset Build Target      | Clear the build target selection             |
| `cargo-tools.projectOutline.setRunTarget`         | Set as Run Target       | Use this target for run and debug operations |
| `cargo-tools.projectOutline.unsetRunTarget`       | Unset Run Target        | Clear the run target selection               |
| `cargo-tools.projectOutline.setBenchmarkTarget`   | Set as Benchmark Target | Use this target for benchmark operations     |
| `cargo-tools.projectOutline.unsetBenchmarkTarget` | Unset Benchmark Target  | Clear the benchmark target selection         |

### Target actions *(context menu only)*

| Command ID                               | Title            | Description                           |
| ---------------------------------------- | ---------------- | ------------------------------------- |
| `cargo-tools.projectOutline.buildTarget` | Build Target     | Build this specific target            |
| `cargo-tools.projectOutline.runTarget`   | Run Target       | Run this executable target            |
| `cargo-tools.projectOutline.debugTarget` | Debug Target     | Start a debug session for this target |
| `cargo-tools.projectOutline.benchTarget` | Benchmark Target | Run benchmarks for this target        |

### View controls

| Command ID                                                 | Title                            | Description                                                |
| ---------------------------------------------------------- | -------------------------------- | ---------------------------------------------------------- |
| `cargo-tools.projectOutline.setWorkspaceMemberFilter`      | Filter Workspace Members         | Show only selected workspace members                       |
| `cargo-tools.projectOutline.editWorkspaceMemberFilter`     | Edit Member Filter               | Modify the active workspace member filter                  |
| `cargo-tools.projectOutline.showTargetTypeFilter`          | Filter Target Types              | Show only selected target types                            |
| `cargo-tools.projectOutline.clearAllFilters`               | Clear All Filters                | Remove all active filters                                  |
| `cargo-tools.projectOutline.toggleWorkspaceMemberGrouping` | Toggle Workspace Member Grouping | Switch between grouping by workspace member or target type |

## cargo-make Commands

| Command ID                              | Title                | Default Keybinding | Description                                                     |
| --------------------------------------- | -------------------- | ------------------ | --------------------------------------------------------------- |
| `cargo-tools.makefile.selectAndRunTask` | Run Makefile Task... | —                  | Pick and run a task from the Command Palette                    |
| `cargo-tools.makefile.runTask`          | Run Makefile Task    | —                  | Run the selected task *(context menu only)*                     |
| `cargo-tools.makefile.pinTask`          | Pin Task             | —                  | Add the selected task to the Pinned Tasks view *(context menu only)* |

### Tasks view controls

These controls apply to both the cargo-make tasks and the cargo alias sections of the Tasks panel.

| Command ID                                  | Title             | Description                                             |
| ------------------------------------------- | ----------------- | ------------------------------------------------------- |
| `cargo-tools.tasks.selectNameFilter`        | Filter by Name    | Filter both tasks and aliases by name                   |
| `cargo-tools.makefile.selectCategoryFilter` | Filter Categories | Filter cargo-make tasks by category                     |
| `cargo-tools.tasks.clearAllFilters`         | Clear All Filters | Remove all active name and category filters             |

## Cargo Alias Commands

Cargo Tools reads `[alias]` entries from `.cargo/config.toml` and surfaces them as runnable tasks. This covers simple aliases as well as the [xtask pattern](https://github.com/matklad/cargo-xtask), where a workspace member acts as a build tool invoked through a cargo alias (e.g. `cargo xtask compile`).

| Command ID                                    | Title                    | Default Keybinding | Description                                                                              |
| --------------------------------------------- | ------------------------ | ------------------ | ---------------------------------------------------------------------------------------- |
| `cargo-tools.xtask.selectAndRunAlias`         | Run Alias...             | —                  | Pick and run an alias from the Command Palette                                           |
| `cargo-tools.xtask.selectAndRunAliasWithArgs` | Run Alias With Args...   | —                  | Pick an alias and supply extra arguments; each alias shows its `--help` text for context |
| `cargo-tools.xtask.runAlias`                  | Run Alias                | —                  | Run the selected alias *(context menu only)*                                             |
| `cargo-tools.xtask.runAliasWithArgs`          | Run Alias With Args      | —                  | Run the selected alias with extra arguments *(context menu only)*                        |
| `cargo-tools.xtask.pinAlias`                  | Pin Alias                | —                  | Add alias to Pinned Tasks with no fixed arguments *(context menu only)*                  |
| `cargo-tools.xtask.pinAliasWithArgs`          | Pin Alias With Args      | —                  | Add alias to Pinned Tasks with fixed default arguments *(context menu only)*             |

## Pinned Tasks Commands

The Pinned Tasks panel holds both cargo-make tasks and cargo aliases. Items execute in order — tasks first, then aliases — when using the numbered keyboard shortcuts.

| Command ID                          | Title                   | Default Keybinding | Description                                                              |
| ----------------------------------- | ----------------------- | ------------------ | ------------------------------------------------------------------------ |
| `cargo-tools.tasks.pinned.add`      | Add Task                | —                  | Add a cargo-make task or cargo alias to the pinned list                  |
| `cargo-tools.tasks.pinned.execute`  | Execute Task            | —                  | Run the selected pinned cargo-make task *(context menu only)*            |
| `cargo-tools.tasks.pinned.remove`   | Remove Task             | —                  | Remove a pinned cargo-make task from the list *(context menu only)*      |
| `cargo-tools.tasks.pinned.executeAlias` | Execute Alias       | —                  | Run the selected pinned alias *(context menu only)*                      |
| `cargo-tools.tasks.pinned.removeAlias`  | Remove Alias        | —                  | Remove a pinned alias from the list *(context menu only)*                |
| `cargo-tools.tasks.pinned.execute1` | Execute 1st Pinned Task | `Ctrl+Alt+1`       | Run the 1st pinned item (tasks first, then aliases)                      |
| `cargo-tools.tasks.pinned.execute2` | Execute 2nd Pinned Task | `Ctrl+Alt+2`       | Run the 2nd pinned item                                                  |
| `cargo-tools.tasks.pinned.execute3` | Execute 3rd Pinned Task | `Ctrl+Alt+3`       | Run the 3rd pinned item                                                  |
| `cargo-tools.tasks.pinned.execute4` | Execute 4th Pinned Task | `Ctrl+Alt+4`       | Run the 4th pinned item                                                  |
| `cargo-tools.tasks.pinned.execute5` | Execute 5th Pinned Task | `Ctrl+Alt+5`       | Run the 5th pinned item                                                  |
