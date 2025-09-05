* ~~make project outline view filtrable (name of crate, type of target)~~ ✅ COMPLETED
  * ✅ **Real-time workspace member filter** - Inline search box with immediate updates (VSCode-style)
  * ✅ **Target type filter** - Checkbox menu for bin, lib, test, example, bench
  * ✅ **Filter buttons in view title bar** - Quick access to filter functions
  * ✅ **Clear all filters functionality** - One-click reset
  * ✅ **Clean UI** - Inline filter input within tree view, no extra nodes
  * ✅ **Debounced updates** - Smooth performance with 300ms debounce
* use rust-analyzer settings where it makes sense (cargo paths, additional arguments etc)
* set rust-analyzer settings through convenience commands
* persist selections via extension context and appropriate keys analoguous to vscode-cmake-tools
* cargo make (Makefile.toml) support
    * discover tasks
    * task view by category
    * package selection
* create extension icon (rust logo with wrench on the lower right corner)
* extension documentation
* polish and release