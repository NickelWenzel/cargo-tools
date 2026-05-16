/// Commands are defined in this extra module
/// which supports also native targets so we can
/// run tests that make sure that all commands in package.json are also implemented
pub mod cargo;
pub mod cargo_make;
pub mod pinned;
