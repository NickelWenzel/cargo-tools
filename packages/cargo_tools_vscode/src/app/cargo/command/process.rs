// mod project_outline;

use cargo_tools::app::cargo::{
    command::{Explicit, Implicit},
    selection::{self, Features, Update},
    ui::Task::*,
};
use iced_headless::Task as IcedTask;

use crate::{
    app::{
        CargoMsg,
        cargo::{
            Grouping, PackageFilter, SettingsUpdate, TargetTypesFilter, TargetTypesFilterUpdate,
            Ui, UiMessage,
            command::{Command, ProjectOutline as PO},
        },
    },
    quick_pick::SelectInput,
    vs_code_api::{JsValueExt, execute_async, log},
};

trait IntoCargoMessage {
    fn into_cargo_msg(self) -> CargoMsg;
}

impl IntoCargoMessage for selection::Update {
    fn into_cargo_msg(self) -> CargoMsg {
        CargoMsg::Selection(self)
    }
}

impl IntoCargoMessage for cargo_tools::app::cargo::ui::Task {
    fn into_cargo_msg(self) -> CargoMsg {
        CargoMsg::Task(self)
    }
}

impl IntoCargoMessage for PackageFilter {
    fn into_cargo_msg(self) -> CargoMsg {
        CargoMsg::Custom(UiMessage::Settings(SettingsUpdate::PackageFilter(self)))
    }
}

impl IntoCargoMessage for TargetTypesFilter {
    fn into_cargo_msg(self) -> CargoMsg {
        CargoMsg::Custom(UiMessage::Settings(SettingsUpdate::TargetTypesFilter(self)))
    }
}

impl IntoCargoMessage for Grouping {
    fn into_cargo_msg(self) -> CargoMsg {
        CargoMsg::Custom(UiMessage::Settings(SettingsUpdate::Grouping(self)))
    }
}

impl Ui {
    pub(crate) fn process_cmd(&self, cmd: Command) -> IcedTask<CargoMsg> {
        match cmd {
            Command::SelectProfile => {
                let input = self.data.profiles();
                run_task(async move { input.select().await.map(Update::SelectedProfile) })
            }
            Command::SelectPackage => {
                let input = self.data.packages();
                run_task(async move { input.select().await.map(Update::SelectedPackage) })
            }
            Command::SelectBuildTarget => {
                let input = self.data.build_target_options();
                run_task(async move { input?.select().await.map(Update::SelectedBuildTarget) })
            }
            Command::SelectRunTarget => {
                let input = self.data.run_target_options();
                run_task(async move { input?.select().await.map(Update::SelectedRunTarget) })
            }
            Command::SelectBenchmarkTarget => {
                let input = self.data.bench_target_options();
                run_task(async move { input?.select().await.map(Update::SelectedBenchmarkTarget) })
            }
            Command::SelectPlatformTarget => {
                let current = self.data.selection.platform_target.clone();
                run_task(async move { select_platform_target(current.clone()).await })
            }
            Command::InstallPlatformTarget => run_task(install_platform_target()),
            Command::SetRustAnalyzerCheckTargets => {
                IcedTask::done(set_rust_analyzer_check_targets())
                    .and_then(IcedTask::done)
                    .map(IntoCargoMessage::into_cargo_msg)
            }
            Command::BuildDocs => IcedTask::done(ExplicitCommand(Explicit::Doc).into_cargo_msg()),
            Command::SelectFeatures => {
                let input = self.data.feature_options();
                run_task(async move { select_features(input).await })
            }
            Command::Refresh => {
                // Not yet implemented
                IcedTask::none()
            }
            Command::Clean => IcedTask::done(ImplicitCommand(Implicit::Clean).into_cargo_msg()),
            Command::Build => IcedTask::done(ImplicitCommand(Implicit::Build).into_cargo_msg()),
            Command::Run => IcedTask::done(ImplicitCommand(Implicit::Run).into_cargo_msg()),
            Command::Debug => {
                // Not yet implemented
                IcedTask::none()
            }
            Command::Test => IcedTask::done(ImplicitCommand(Implicit::Test).into_cargo_msg()),
            Command::Bench => IcedTask::done(ImplicitCommand(Implicit::Bench).into_cargo_msg()),
            Command::ProjectOutline(cmd) => self.process_outline_cmd(cmd),
        }
    }

    pub(crate) fn process_outline_cmd(&self, cmd: PO) -> IcedTask<CargoMsg> {
        match cmd {
            PO::Select(update) => IcedTask::done(update.into_cargo_msg()),
            PO::Unselect(update) => IcedTask::done(update.into_cargo_msg()),
            PO::Build(target) => {
                IcedTask::done(ExplicitCommand(Explicit::Build(target)).into_cargo_msg())
            }
            PO::Test(package) => {
                IcedTask::done(ExplicitCommand(Explicit::Test { package }).into_cargo_msg())
            }
            PO::Clean(package) => {
                IcedTask::done(ExplicitCommand(Explicit::Clean { package }).into_cargo_msg())
            }
            PO::Run(target) => {
                IcedTask::done(ExplicitCommand(Explicit::Run(Some(target))).into_cargo_msg())
            }
            PO::Debug(_) => {
                // Not yet implemented
                IcedTask::none()
            }
            PO::Bench(package) => {
                IcedTask::done(ExplicitCommand(Explicit::Bench { package }).into_cargo_msg())
            }
            PO::SelectWorkspaceMemberFilter => todo!(),
            PO::EditWorkspaceMemberFilter(filter) => {
                IcedTask::done(PackageFilter(filter).into_cargo_msg())
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
                IcedTask::done(filter.into_cargo_msg())
            }
            PO::ClearAllFilters => {
                let member_filter = IcedTask::done(PackageFilter::default().into_cargo_msg());
                let types_filter = IcedTask::done(TargetTypesFilter::default().into_cargo_msg());

                IcedTask::batch([member_filter, types_filter])
            }
            PO::ToggleWorkspaceMemberGrouping => {
                IcedTask::done(self.settings.grouping.toggle().into_cargo_msg())
            }
        }
    }
}

fn run_task(
    fut: impl Future<Output = Option<impl IntoCargoMessage + 'static>> + 'static,
) -> IcedTask<CargoMsg> {
    IcedTask::future(fut)
        .and_then(IcedTask::done)
        .map(IntoCargoMessage::into_cargo_msg)
}

async fn select_platform_target(current: Option<String>) -> Option<impl IntoCargoMessage> {
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

async fn install_platform_target() -> Option<impl IntoCargoMessage> {
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
            return None;
        }
    };

    let input = SelectInput {
        options,
        current: Vec::new(),
    };

    input.select().await.map(AddPlatformTarget)
}

fn set_rust_analyzer_check_targets() -> Option<impl IntoCargoMessage> {
    log("'Set rust-analyzer check targets' not yet implemented");
    Option::<Update>::None
}

async fn select_features(input: Option<SelectInput<String>>) -> Option<impl IntoCargoMessage> {
    let selected_features = input?.select_multiple().await?;
    let features = if selected_features.iter().any(|f| f == "All Features") {
        Features::All
    } else {
        Features::Some(selected_features)
    };

    Some(Update::SelectedFeatures(features))
}
