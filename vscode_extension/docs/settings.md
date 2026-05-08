# Settings Reference

Configure Cargo Tools by adding settings to VS Code's `settings.json` (user or workspace level). All settings are under the `cargoTools` namespace.

## Cargo Invocation

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `cargoTools.cargoCommand` | `string` | `"cargo"` | Command to invoke instead of `cargo`. If the value contains whitespace, the first word is used as the command and the remaining words are prepended as arguments. Useful for wrappers such as `cross`. |
| `cargoTools.extraEnv` | `object` | `{}` | Additional environment variables set for every cargo command. Merged with the shell environment. |
| `cargoTools.buildArgs` | `string[]` | `[]` | Additional arguments appended to every `cargo build` invocation. |

## Run and Debug

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `cargoTools.runCommandOverride` | `string` | `""` | Override the command used for run operations. When empty, `cargo run` is used. Example: `"cargo watch -x run"`. |
| `cargoTools.run.extraArgs` | `string[]` | `[]` | Additional arguments appended to every run or debug invocation. Arguments after `--` are passed to the binary. |
| `cargoTools.run.extraEnv` | `object` | `{}` | Additional environment variables set for run and debug operations. Merged with `cargoTools.extraEnv`. |

## Test and Benchmark

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `cargoTools.testCommandOverride` | `string` | `""` | Override the command used for test operations. When empty, `cargo test` is used. Example: `"cargo nextest run"`. |
| `cargoTools.test.extraArgs` | `string[]` | `[]` | Additional arguments appended to every test or benchmark invocation. |
| `cargoTools.test.extraEnv` | `object` | `{}` | Additional environment variables set for test and benchmark operations. Merged with `cargoTools.extraEnv`. |

## rust-analyzer Integration

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `cargoTools.useRustAnalyzerEnvAndArgs` | `boolean` | `false` | When enabled, Cargo Tools reads `rust-analyzer.cargo.extraArgs`, `rust-analyzer.cargo.extraEnv`, `rust-analyzer.runnables.extraArgs`, and `rust-analyzer.runnables.extraTestBinaryArgs` and incorporates them when constructing cargo commands. |
| `cargoTools.updateRustAnalyzerTarget` | `boolean` | `false` | When enabled, changing the **Platform Target** selection also updates `rust-analyzer.cargo.target`, keeping the analyzer's target in sync. |

## Example Configurations

### Using cargo-nextest

```json
{
  "cargoTools.testCommandOverride": "cargo nextest run"
}
```

### Cross-compilation with automatic rust-analyzer sync

```json
{
  "cargoTools.updateRustAnalyzerTarget": true
}
```

Use **Cargo Tools: Select Platform Target** to switch target triples. Both Cargo Tools and rust-analyzer stay in sync automatically.

### Custom cargo wrapper

```json
{
  "cargoTools.cargoCommand": "cross"
}
```

### Verbose builds with additional run arguments

```json
{
  "cargoTools.buildArgs": ["--verbose"],
  "cargoTools.run.extraArgs": ["--", "--my-flag"],
  "cargoTools.extraEnv": {
    "RUST_LOG": "debug"
  }
}
```

### Inheriting rust-analyzer environment

```json
{
  "cargoTools.useRustAnalyzerEnvAndArgs": true
}
```

When enabled, extra arguments and environment variables defined in your rust-analyzer settings are automatically applied to all cargo invocations from Cargo Tools.
