// mod project_outline;

use std::collections::HashMap;

use cargo_tools::{
    app::cargo::{
        command::{Explicit, Implicit},
        selection::{self, Features, Update},
    },
    runtime::{self, CargoTask, Runtime as _},
};
use iced_headless::Task;

use crate::{
    app::cargo::{
        Grouping, Message, PackageFilter, SettingsUpdate, TargetTypesFilter,
        TargetTypesFilterUpdate, Ui,
        command::{Command, ProjectOutline as PO},
    },
    quick_pick::SelectInput,
    runtime::VsCodeRuntime as Runtime,
    vs_code_api::{JsValueExt, execute_async, log},
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
            Command::Refresh => {
                // Not yet implemented
                Task::none()
            }
            Command::Clean => self.cmd_exec(Implicit::Clean),
            Command::Build => self.cmd_exec(Implicit::Build),
            Command::Run => self.cmd_exec(Implicit::Run),
            Command::Debug => {
                // Not yet implemented
                Task::none()
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
            PO::Debug(_) => {
                // Not yet implemented
                Task::none()
            }
            PO::Bench(package) => self.cmd_exec(Explicit::Bench { package }),
            PO::SelectWorkspaceMemberFilter => todo!(),
            PO::EditWorkspaceMemberFilter(filter) => {
                Task::done(PackageFilter(filter).into_cargo_msg())
            }
            PO::SelectTargetTypeFilter => todo!(),
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
    let platform_targets = match execute_async("rustup target list").await {
        Ok(output) => {
            let output_str = output.as_string().unwrap_or_default();
            output_str
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.ends_with("(installed)") {
                        Some(line.trim_end_matches("(installed)").trim().to_string())
                    } else {
                        None
                    }
                })
                .map(Some)
                .collect::<Vec<_>>()
        }
        Err(e) => {
            log(&format!(
                "Failed to get platform targets from rustup: {}",
                e.to_error_string()
            ));
            return None;
        }
    };

    let input = {
        let mut options = vec![None];
        options.extend(platform_targets);
        let current = vec![current];
        SelectInput { options, current }
    };

    input.select().await.map(Update::SelectedPlatformTarget)
}

async fn install_platform_target() {
    let options = match execute_async("rustup target list").await {
        Ok(output) => {
            let output_str = output.as_string().unwrap_or_default();
            output_str
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.ends_with("(installed)") {
                        None
                    } else {
                        Some(line.to_string())
                    }
                })
                .collect::<Vec<_>>()
        }
        Err(e) => {
            log(&format!(
                "Failed to get platform targets from rustup: {}",
                e.to_error_string()
            ));
            return;
        }
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
    log("'Set rust-analyzer check targets' not yet implemented");
    Option::<Update>::None
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
