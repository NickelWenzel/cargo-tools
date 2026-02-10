// mod project_outline;

use std::{collections::HashMap, iter};

use cargo_tools::{
    cargo::{
        command::{BuildSubTarget, BuildTarget, Explicit, Implicit, RunSubTarget, RunTarget},
        selection::{self, Features, Update},
    },
    profile::Profile,
    runtime::{self, CargoTask, Runtime as _},
};
use futures::SinkExt;
use iced_headless::Task;
use serde_wasm_bindgen::to_value;
use std::path::PathBuf;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    extension::cargo::{
        Grouping, Message, PackageFilter, SettingsUpdate, TargetTypesFilter,
        TargetTypesFilterUpdate, Ui,
        command::{Command, ProjectOutline as PO},
    },
    quick_pick::{SelectInput, ToQuickPickItem},
    runtime::VsCodeRuntime as Runtime,
    vs_code_api::{JsValueExt, debug, execute_async, host_platform, log, show_quick_pick_type},
};

trait IntoMessage {
    fn into_cargo_msg(self) -> Message;
}

impl IntoMessage for selection::Update {
    fn into_cargo_msg(self) -> Message {
        Message::SelectionChanged(self)
    }
}

impl IntoMessage for PackageFilter {
    fn into_cargo_msg(self) -> Message {
        Message::SettingsChanged(SettingsUpdate::PackageFilter(self))
    }
}

impl IntoMessage for TargetTypesFilter {
    fn into_cargo_msg(self) -> Message {
        Message::SettingsChanged(SettingsUpdate::TargetTypesFilter(self))
    }
}

impl IntoMessage for Grouping {
    fn into_cargo_msg(self) -> Message {
        Message::SettingsChanged(SettingsUpdate::Grouping(self))
    }
}

impl Ui {
    pub(crate) fn process_cmd(&self, cmd: Command) -> Task<Message> {
        match cmd {
            Command::SelectProfile => {
                let input = self.data.profiles();
                done(async move { input.select().await.map(Update::SelectedProfile) })
            }
            Command::SelectPackage => {
                let input = self.data.packages();
                done(async move { input.select().await.map(Update::SelectedPackage) })
            }
            Command::SelectBuildTarget => {
                let input = self.data.build_target_options();
                done(async move { input?.select().await.map(Update::SelectedBuildTarget) })
            }
            Command::SelectRunTarget => {
                let input = self.data.run_target_options();
                done(async move { input?.select().await.map(Update::SelectedRunTarget) })
            }
            Command::SelectBenchmarkTarget => {
                let input = self.data.bench_target_options();
                done(async move { input?.select().await.map(Update::SelectedBenchmarkTarget) })
            }
            Command::SelectPlatformTarget => {
                let current = self.data.selection.platform_target.clone();
                done(async move { select_platform_target(current.clone()).await })
            }
            Command::InstallPlatformTarget => Task::future(install_platform_target()).discard(),
            Command::SetRustAnalyzerCheckTargets => Task::done(set_rust_analyzer_check_targets())
                .and_then(Task::done)
                .map(IntoMessage::into_cargo_msg),
            Command::BuildDocs => self.cmd_exec(Explicit::Doc),
            Command::SelectFeatures => {
                let input = self.data.feature_options();
                done(async move { select_features(input).await })
            }
            Command::Refresh => Task::done(Message::ManifestChanged),
            Command::Clean => self.cmd_exec(Implicit::Clean),
            Command::Build => self.cmd_exec(Implicit::Build),
            Command::Run => self.cmd_exec(Implicit::Run),
            Command::Debug => {
                let Explicit::Debug(Some(target)) =
                    Implicit::Debug.to_explicit(&self.data.selection)
                else {
                    return Task::none();
                };
                self.debug(target)
            }
            Command::Test => self.cmd_exec(Implicit::Test),
            Command::Bench => self.cmd_exec(Implicit::Bench),
            Command::ProjectOutline(cmd) => self.process_outline_cmd(cmd),
        }
    }

    pub(crate) fn process_outline_cmd(&self, cmd: PO) -> Task<Message> {
        match cmd {
            PO::Select(update) => Task::done(update.into_cargo_msg()),
            PO::Unselect(update) => Task::done(update.into_cargo_msg()),
            PO::Build(target) => self.cmd_exec(Explicit::Build(target)),
            PO::Test(package) => self.cmd_exec(Explicit::Test { package }),
            PO::Clean(package) => self.cmd_exec(Explicit::Clean { package }),
            PO::Run(target) => self.cmd_exec(Explicit::Run(Some(target))),
            PO::Debug(target) => self.debug(target),
            PO::Bench(package) => self.cmd_exec(Explicit::Bench { package }),
            PO::SelectWorkspaceMemberFilter => self.select_workspace_member_filter(),
            PO::EditWorkspaceMemberFilter(filter) => {
                Task::done(PackageFilter(filter).into_cargo_msg())
            }
            PO::SelectTargetTypeFilter => todo!(), // TODO
            PO::EditTargetTypeFilter(update) => {
                let mut filter = self.settings.target_types_filter.clone();
                match update {
                    TargetTypesFilterUpdate::Bin(on) => filter.bin = on,
                    TargetTypesFilterUpdate::Lib(on) => filter.lib = on,
                    TargetTypesFilterUpdate::Example(on) => filter.example = on,
                    TargetTypesFilterUpdate::Benchmarks(on) => filter.benchmarks = on,
                    TargetTypesFilterUpdate::Features(on) => filter.features = on,
                };
                Task::done(filter.into_cargo_msg())
            }
            PO::ClearAllFilters => {
                let member_filter = Task::done(PackageFilter::default().into_cargo_msg());
                let types_filter = Task::done(TargetTypesFilter::default().into_cargo_msg());

                Task::batch([member_filter, types_filter])
            }
            PO::ToggleWorkspaceMemberGrouping => {
                Task::done(self.settings.grouping.toggle().into_cargo_msg())
            }
        }
    }

    fn cmd_exec(&self, cmd: impl ToTask) -> Task<Message> {
        Task::future(Runtime::exec_task(cmd.into_task(&self.data.selection))).discard()
    }

    fn debug(&self, target: RunTarget) -> Task<Message> {
        let Some(run_target) = target.target.as_ref() else {
            return Task::none();
        };
        let build_sub_target = match run_target {
            RunSubTarget::Bin(t) => BuildSubTarget::Bin(t.clone()),
            RunSubTarget::Example(t) => BuildSubTarget::Example(t.clone()),
        };
        let build_target = BuildTarget {
            package: target.package.clone(),
            target: Some(build_sub_target),
        };
        let mut selection = self.data.selection.clone();
        selection.profile = Profile::Dev; // For now always use standard dev profile
        // TODO: make it possible to run shell commands with env arguments
        let build_debug_task =
            Explicit::Build(Some(build_target)).to_task(&selection, &Runtime::get_configuration());

        let Some(target_dir) = self
            .data
            .metadata
            .metadata
            .as_ref()
            .map(|metadata| metadata.target_directory.to_string())
        else {
            return Task::none();
        };

        let target_exe_path = exec_path(run_target, &self.data.selection, &target_dir);

        Task::future(async move {
            Runtime::exec_task(build_debug_task).await;

            if let Err(e) = debug(&target_exe_path, &target.package).await {
                log(&format!("Error while dbugging: {}", e.to_error_string()));
            }
        })
        .discard()
    }

    fn select_workspace_member_filter(&self) -> Task<Message> {
        let current = self.settings.package_filter.clone().into_string();
        let Some(Ok(options)) = self.data.metadata.metadata.as_ref().map(|m| {
            m.workspace_packages()
                .into_iter()
                .map(|p| to_value(&p.name.to_string().to_item(false)))
                .collect::<Result<Array, _>>()
        }) else {
            return Task::none();
        };

        if options.length() < 2 {
            return Task::none();
        }

        let cmd_tx = self.cmd_tx.clone();

        Task::future(async move {
            // Closure only needs to live while the quickpick is active
            let filter_update = Closure::new(move |filter: String| {
                let mut tx = cmd_tx.clone();
                spawn_local(async move {
                    log(&format!("Sending workspace member filter '{filter}'"));
                    if let Err(e) = tx
                        .send(PO::EditWorkspaceMemberFilter(filter).to_cmd())
                        .await
                    {
                        log(&format!("Failed to queue msg: {}", e));
                    }
                });
            });

            let filter = show_quick_pick_type(current.clone(), options, &filter_update)
                .await
                .map(|f| f.as_string().unwrap_or(current.clone()))
                .unwrap_or(current);

            PackageFilter(filter)
        })
        .map(PackageFilter::into_cargo_msg)
    }
}

fn done(fut: impl Future<Output = Option<impl IntoMessage + 'static>> + 'static) -> Task<Message> {
    Task::future(fut)
        .and_then(Task::done)
        .map(IntoMessage::into_cargo_msg)
}

trait ToTask {
    fn into_task(self, selection: &selection::State) -> CargoTask;
}

impl ToTask for Implicit {
    fn into_task(self, selection: &selection::State) -> CargoTask {
        self.to_task(selection, &Runtime::get_configuration())
    }
}

impl ToTask for Explicit {
    fn into_task(self, selection: &selection::State) -> CargoTask {
        self.to_task(selection, &Runtime::get_configuration())
    }
}

async fn select_platform_target(current: Option<String>) -> Option<impl IntoMessage> {
    let options = match platform_targets().await {
        Some(targets) => targets.into_iter().filter_map(|t| {
            (!t.ends_with("(installed)"))
                .then_some(Some(t.trim_end_matches("(installed)").trim().to_string()))
        }),
        None => return None,
    };

    let input = {
        SelectInput {
            options: iter::once(None).chain(options).collect(),
            current: vec![current],
        }
    };

    input.select().await.map(Update::SelectedPlatformTarget)
}

async fn install_platform_target() {
    let options = match platform_targets().await {
        Some(targets) => targets
            .into_iter()
            .filter(|t| !t.ends_with("(installed)"))
            .collect(),
        None => return,
    };

    let input = SelectInput {
        options,
        current: Vec::new(),
    };

    let Some(target) = input.select().await else {
        return;
    };

    Runtime::exec_task(CargoTask::Cargo(runtime::Task {
        cmd: "rustup".to_string(),
        args: vec!["target".to_string(), "add".to_string(), target],
        env: HashMap::new(),
    }))
    .await
}

fn set_rust_analyzer_check_targets() -> Option<impl IntoMessage> {
    // TODO
    log("'Set rust-analyzer check targets' not yet implemented");
    Option::<Update>::None
}

async fn platform_targets() -> Option<Vec<String>> {
    let rustup_args = Vec::from_iter(["target", "list"].map(ToString::to_string));
    match execute_async("rustup", rustup_args).await {
        Ok(output) => output
            .as_string()
            .map(|s| s.lines().map(|l| l.trim().to_string()).collect()),
        Err(e) => {
            log(&format!(
                "Failed to get platform targets from rustup: {}",
                e.to_error_string()
            ));
            None
        }
    }
}

async fn select_features(input: Option<SelectInput<String>>) -> Option<impl IntoMessage> {
    let selected_features = input?.select_multiple().await?;
    let features = if selected_features.iter().any(|f| f == "All Features") {
        Features::All
    } else {
        Features::Some(selected_features)
    };

    Some(Update::SelectedFeatures(features))
}

fn exec_path(target: &RunSubTarget, selection: &selection::State, target_dir: &str) -> String {
    let path_components = iter::once(target_dir.to_string())
        .chain(iter::once("debug".to_string())) // For now always assume debug profile
        .chain(selection.platform_target.as_ref().map(|t| t.to_string()))
        .chain(match target {
            RunSubTarget::Bin(bin) => vec![bin.clone()],
            RunSubTarget::Example(example) => {
                vec!["example".to_string(), example.to_string()]
            }
        })
        .chain((host_platform() == "windows").then_some(".exe".to_string()));

    PathBuf::from_iter(path_components)
        .to_string_lossy()
        .to_string()
}
