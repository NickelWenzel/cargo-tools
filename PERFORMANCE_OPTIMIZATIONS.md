# Performance optimization candidates

## Context

This is a general performance pass over the hybrid Rust/WASM VS Code extension. The extension is driven by an Elm-style `update()` loop: filter previews, quick-pick selections, and file-change events flow through `Extension::update()` as messages. The most important costs for perceived performance are work repeated for every message or keystroke, subprocesses triggered by file watchers, extension-state writes that cross the WASM/JavaScript boundary, and the size and startup cost of the release WASM module.

Metadata and task parsing are not rerun by ordinary UI interactions, but they are triggered by file watchers. Those watcher notifications are currently neither debounced nor coalesced, so a single atomic save can start redundant parsing or subprocess work. The recommendations below are ordered by expected value and separate low-risk runtime changes from build-profile experiments that require measurement.

## Recommended changes

### 1. Filter Rust tracing and avoid serializing quick-pick contents

`packages/cargo_tools_vscode/src/extension/mod.rs` installs `VSCodeLogger` without a level filter:

```rust
tracing_subscriber::registry().with(VSCodeLogger).init();
```

`VSCodeLogger` does not override `Layer::enabled`, so every tracing event reaches `MessageVisitor` and crosses the JavaScript boundary. In particular, the `debug!` call in `Extension::update()` formats every message, including filter-preview messages. Changing a call from `info!` to `debug!` therefore has no release-runtime benefit until a subscriber filter is installed.

Use the `LevelFilter` already available with the current `tracing-subscriber` features. Keep debug diagnostics in development builds and INFO-or-higher diagnostics in release builds:

```rust
use tracing_subscriber::filter::LevelFilter;

let level = if cfg!(debug_assertions) {
    LevelFilter::DEBUG
} else {
    LevelFilter::INFO
};

tracing_subscriber::registry()
    .with(level)
    .with(VSCodeLogger)
    .init();
```

Do not add `EnvFilter` for this change: its feature is not enabled, and adding runtime parsing/configuration would increase complexity and WASM size for no current requirement.

There is a second logging hot spot in `packages/cargo_tools_vscode/src/quick_pick.ts`:

```ts
log.info(`Show quick pick for ${JSON.stringify(items)}`);
```

This serializes every quick-pick item and writes the complete payload at INFO level whenever a picker opens. Replace it with a payload-free debug message such as ``log.debug(`Show quick pick with ${items.length} items`)``, or remove it. Merely changing `info` to `debug` while retaining `JSON.stringify` is insufficient because the template expression is evaluated before the logger sees the message.

### 2. Debounce file-watcher event bursts

`FileWatcher` in `packages/cargo_tools_vscode/src/runtime.ts` forwards every create, change, and delete notification immediately. Editors that save via a temporary file or atomic rename may emit several events for one logical save. Each notification can trigger expensive work, including `cargo metadata` and cargo-make discovery subprocesses.

Add a fixed 100 ms trailing-edge debounce inside `FileWatcher`:

- Route all three watcher callbacks through one scheduling method.
- Cancel the previous timer whenever another event arrives within the window.
- Invoke `onChanged` once after 100 ms without another event.
- Clear any pending timer in `dispose()` as well as disposing the VS Code watcher/listeners.

Keeping the debounce in the shared TypeScript wrapper applies it consistently to Cargo manifests, `.cargo/config.toml`, and `Makefile.toml` without changing Rust component interfaces. Events arriving after the debounce window must still schedule a new refresh.

### 3. Do not persist transient filter previews

Live filter callbacks currently update component settings and immediately call `persist_state_vs_code`. A typed task-name filter is dispatched to both the cargo-make and alias components, so each keystroke can cause two JSON serializations and two `workspaceState.update` calls in addition to the intended tree refresh. Multi-select category and target-type previews have the same pattern.

Preserve live tree previews, but distinguish preview updates from committed settings updates:

- Preview messages update in-memory filter state and refresh the relevant tree without persistence.
- When a quick pick is accepted, its final value is sent as a committed update and persisted once per owning settings object.
- When a quick pick is canceled, the original value is restored through a committed update and persisted.
- Apply this to workspace-member, shared task-name, cargo-make category, and outline target-type filters.

This keeps the current UX while removing storage writes from the keystroke/selection hot path. It also avoids introducing a timer or delayed-write task into the Rust update loop.

### 4. Lowercase filter input once per filtering pass

- `MakefileTask::keep` in `packages/cargo_tools/src/cargo_make.rs` recomputes `task_filter.to_lowercase()` for every task on every preview update.
- `XtaskAlias::keep` in `packages/cargo_tools/src/xtask.rs` does the same for aliases.

Lowercase the filter once in `MakefileTasks::filtered` and `XtaskAliases::filtered`, then pass the normalized `&str` into `keep`. Continue lowercasing each candidate name so matching behavior remains unchanged. Do not introduce trimming or ASCII-only matching as part of this optimization, because either would alter existing behavior.

### 5. Borrow selected packages while building quick-pick options

`Config::selected_package` in `packages/cargo_tools/src/cargo/config.rs` returns an owned `Package` by deep-cloning its targets and features. The build-target, run-target, benchmark-target, and feature-option methods only read that package.

Return `Option<&Package>` instead and update those callers to borrow it. Clone only the strings placed in the final option vectors. In the workspace-wide feature branch, iterate over `&package.features`, sort and deduplicate references, and clone only the resulting feature names.

### 6. Keep metadata parsing cleanup narrowly scoped

`Package::from_metadata` uses `sorted_by_key(|pkg| pkg.name.clone())`, which allocates a cloned key for each package. Replace it with a comparator over borrowed names, using either `sorted_by(|a, b| a.name.cmp(&b.name))` or a collected `Vec` followed by `sort_by`.

Do **not** add the previously proposed `HashSet` for `workspace_members` without benchmark evidence. Metadata conversion is a cold path, workspace package counts are usually small, and allocating a second collection may not amortize the current linear membership checks. The sort-key change is also a micro-optimization and should remain lower priority than items 1-5.

### 7. Treat WASM release-profile tuning as an experiment

The workspace has no explicit `[profile.release]`. Production packaging therefore uses Cargo's release defaults. A smaller installed WASM can reduce disk I/O, compilation, and instantiation work during activation, but size-oriented optimization may reduce steady-state execution speed and increase build time.

Measure a fresh release package first, then test this conservative profile while retaining runtime-oriented optimization:

```toml
[profile.release]
lto = "fat"
codegen-units = 1
strip = "symbols"
opt-level = 3
```

Test `opt-level = "s"` as a separate variant. Adopt it only if the additional reduction in embedded WASM and VSIX size improves measured activation without a meaningful regression in filter, tree-generation, or parsing scenarios. Do not describe profile tuning as unconditionally low-risk: LTO and one codegen unit increase release build time, while `"s"` explicitly trades some runtime optimization for size.

#### Packaging measurements

Measurements were collected on 2026-07-21 with a warmed build cache. Each duration is wall-clock time for `cargo xt-pkg`; sizes are exact bytes from the resulting `cargo-tools.vsix` and uncompressed `dist/cargo_tools_vscode_bg.wasm`.

| Variant | VSIX bytes | WASM bytes | `cargo xt-pkg` |
| --- | ---: | ---: | ---: |
| Baseline, before the optimization series | 538,134 | 1,147,830 | 15.39 s |
| `lto = "fat"`, one codegen unit, stripped symbols, `opt-level = 3` | 493,639 | 1,077,641 | 11.41 s |
| Same profile with temporary `opt-level = "s"` | 454,463 | 1,001,451 | 10.75 s |

The retained `opt-level = 3` profile reduced the measured VSIX by 44,495 bytes (8.3%) and the embedded WASM by 70,189 bytes (6.1%) relative to the baseline. The `"s"` variant was restored after measurement: although it was smaller, activation time and representative runtime behavior could not be profiled in an Extension Development Host in this environment, so there is not enough evidence to accept its runtime tradeoff.

The installed `wasm-opt` also reported that bulk-memory operations were not enabled while validating these builds. The webpack packaging pipeline still exited successfully and produced each VSIX, but it retained the pre-`wasm-opt` WASM. These figures therefore compare the emitted package artifacts and should be repeated after the Binaryen/`wasm-opt` toolchain is updated. Extension activation time was not available in this non-GUI environment.

## Explicitly out of scope

- Incremental tree diffing. Filtering changes the visible node set, and replacing full `_onDidChangeTreeData.fire()` refreshes would be a larger architectural change. The preview-persistence optimization above intentionally preserves current tree refresh behavior.
- Converting small `Vec::find` and `.position()` lookups in tree and pinned-task code to `HashMap`. Typical package and task counts do not justify the additional indexing and synchronization complexity without profiling evidence.
- Caching cargo metadata or task parser results independently of file watchers. Debouncing redundant watcher bursts is the smaller first step; further caching should be driven by measurements after that change.

## Verification and measurement

1. Run `cargo lint`, `cargo xt-test`, and `cargo compile` after the runtime and core-library changes.
2. In an Extension Development Host, exercise task-name, alias, workspace-member, category, and target-type filters. Confirm live preview, acceptance, and cancellation behavior remain unchanged.
3. Instrument or inspect `set_state` calls while interacting with filters. Preview keystrokes/selections must not persist; accepting or canceling must produce only the final settings writes.
4. Save `Cargo.toml`, `.cargo/config.toml`, and `Makefile.toml` using both normal and atomic-save editors. A burst inside 100 ms must produce one refresh, while a later event must still produce another refresh.
5. Verify debug builds retain debug logs and release builds suppress DEBUG/TRACE before formatting or crossing into JavaScript. INFO, WARN, and ERROR events must remain visible in release builds. Confirm quick-pick logs no longer contain serialized item payloads.
6. Run `cargo xt-pkg` before and after each release-profile variant. Record the VSIX size, the uncompressed embedded `dist/cargo_tools_vscode_bg.wasm` size, and activation time from VS Code's running-extension diagnostics or an Extension Host profile.
7. Adopt release-profile changes only when the measurements show an improvement. Keep the baseline results with the implementation change so future dependency upgrades can be compared consistently.
