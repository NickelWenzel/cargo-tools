import * as vscode from 'vscode';

/**
 * Centralized icon mapping for consistent, colorful icons throughout the extension
 * Using VS Code chart colors for vibrant, distinct visual separation
 */
export class IconMapping {
    // Core Cargo concepts with distinct chart colors
    static readonly PROJECT = new vscode.ThemeIcon('repo', new vscode.ThemeColor('charts.blue'));
    static readonly PACKAGE = new vscode.ThemeIcon('package', new vscode.ThemeColor('charts.orange'));
    static readonly WORKSPACE = new vscode.ThemeIcon('organization', new vscode.ThemeColor('charts.purple'));

    // Target types with vibrant, distinguishable colors
    static readonly BIN_TARGET = new vscode.ThemeIcon('file-binary', new vscode.ThemeColor('charts.green'));
    static readonly LIB_TARGET = new vscode.ThemeIcon('library', new vscode.ThemeColor('charts.blue'));
    static readonly EXAMPLE_TARGET = new vscode.ThemeIcon('lightbulb', new vscode.ThemeColor('charts.yellow'));
    static readonly TEST_TARGET = new vscode.ThemeIcon('beaker', new vscode.ThemeColor('charts.purple'));
    static readonly BENCH_TARGET = new vscode.ThemeIcon('dashboard', new vscode.ThemeColor('charts.red'));
    static readonly UNKNOWN_TARGET = new vscode.ThemeIcon('file', new vscode.ThemeColor('charts.foreground'));

    // Actions with consistent, vibrant color coding
    static readonly BUILD_ACTION = new vscode.ThemeIcon('tools', new vscode.ThemeColor('charts.blue'));
    static readonly RUN_ACTION = new vscode.ThemeIcon('play', new vscode.ThemeColor('charts.green'));
    static readonly DEBUG_ACTION = new vscode.ThemeIcon('debug-alt', new vscode.ThemeColor('charts.orange'));
    static readonly TEST_ACTION = new vscode.ThemeIcon('beaker', new vscode.ThemeColor('charts.purple'));
    static readonly BENCH_ACTION = new vscode.ThemeIcon('dashboard', new vscode.ThemeColor('charts.red'));
    static readonly CLEAN_ACTION = new vscode.ThemeIcon('trash', new vscode.ThemeColor('charts.red'));

    // Configuration and settings with chart colors
    static readonly PROFILE_CONFIG = new vscode.ThemeIcon('settings-gear', new vscode.ThemeColor('charts.yellow'));
    static readonly PLATFORM_CONFIG = new vscode.ThemeIcon('device-desktop', new vscode.ThemeColor('charts.cyan'));
    static readonly TARGET_CONFIG = new vscode.ThemeIcon('target', new vscode.ThemeColor('charts.green'));
    static readonly FEATURES_CONFIG = new vscode.ThemeIcon('symbol-misc', new vscode.ThemeColor('charts.purple'));

    // States and status with meaningful chart colors
    static readonly SELECTED_STATE = new vscode.ThemeIcon('check', new vscode.ThemeColor('charts.green'));
    static readonly UNSELECTED_STATE = new vscode.ThemeIcon('circle-outline', new vscode.ThemeColor('charts.foreground'));
    static readonly WARNING_STATE = new vscode.ThemeIcon('warning', new vscode.ThemeColor('charts.yellow'));
    static readonly ERROR_STATE = new vscode.ThemeIcon('error', new vscode.ThemeColor('charts.red'));

    // Makefile specific with chart colors
    static readonly MAKEFILE_CATEGORY = new vscode.ThemeIcon('folder', new vscode.ThemeColor('charts.cyan'));
    static readonly MAKEFILE_TASK = new vscode.ThemeIcon('gear', new vscode.ThemeColor('charts.blue'));

    // Utility actions with chart color distinctions
    static readonly REFRESH_ACTION = new vscode.ThemeIcon('refresh', new vscode.ThemeColor('charts.foreground'));
    static readonly ADD_ACTION = new vscode.ThemeIcon('add', new vscode.ThemeColor('charts.green'));
    static readonly REMOVE_ACTION = new vscode.ThemeIcon('remove', new vscode.ThemeColor('charts.red'));
    static readonly EDIT_ACTION = new vscode.ThemeIcon('edit', new vscode.ThemeColor('charts.orange'));
    static readonly PIN_ACTION = new vscode.ThemeIcon('pin', new vscode.ThemeColor('charts.yellow'));
    static readonly UNPIN_ACTION = new vscode.ThemeIcon('pinned', new vscode.ThemeColor('charts.foreground'));

    // Documentation and info with chart colors
    static readonly DOCS_ACTION = new vscode.ThemeIcon('book', new vscode.ThemeColor('charts.cyan'));
    static readonly FILTER_ACTION = new vscode.ThemeIcon('filter', new vscode.ThemeColor('charts.purple'));
    static readonly LIST_ACTION = new vscode.ThemeIcon('list-unordered', new vscode.ThemeColor('charts.foreground'));

    /**
     * Get icon for target type
     */
    static getIconForTargetType(type: string): vscode.ThemeIcon {
        switch (type) {
            case 'bin':
                return this.BIN_TARGET;
            case 'lib':
            case 'dylib':
            case 'staticlib':
            case 'cdylib':
            case 'rlib':
            case 'proc-macro':
                return this.LIB_TARGET;
            case 'example':
                return this.EXAMPLE_TARGET;
            case 'test':
                return this.TEST_TARGET;
            case 'bench':
                return this.BENCH_TARGET;
            default:
                return this.UNKNOWN_TARGET;
        }
    }

    /**
     * Convert colorful icon to string format for package.json
     */
    static toIconString(icon: vscode.ThemeIcon): string {
        return `$(${icon.id})`;
    }
}
