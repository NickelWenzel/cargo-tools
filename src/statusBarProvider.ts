import * as vscode from 'vscode';
import { CargoWorkspace } from './cargoWorkspace';
import { CargoProfile } from './cargoProfile';

export class StatusBarProvider {
    private profileStatusBarItem: vscode.StatusBarItem;
    private targetStatusBarItem: vscode.StatusBarItem;

    constructor(private workspace: CargoWorkspace) {
        // Create status bar items with proper priority spacing (following CMake Tools pattern)
        this.profileStatusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 3.25);
        this.targetStatusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 3.24);

        // Set up commands
        this.profileStatusBarItem.command = 'cargo-tools.selectProfile';
        this.targetStatusBarItem.command = 'cargo-tools.selectTarget';

        // Set up tooltips
        this.profileStatusBarItem.tooltip = 'Click to change build profile';
        this.targetStatusBarItem.tooltip = 'Click to change build target';

        // Listen for changes
        workspace.onDidChangeProfile(() => this.updateProfileStatusBar());
        workspace.onDidChangeTarget(() => this.updateTargetStatusBar());

        // Initial update
        this.updateProfileStatusBar();
        this.updateTargetStatusBar();

        // Show the items
        this.profileStatusBarItem.show();
        this.targetStatusBarItem.show();
    }

    private updateProfileStatusBar(): void {
        const profile = this.workspace.currentProfile;
        const displayName = CargoProfile.getDisplayName(profile);

        this.profileStatusBarItem.text = `$(tools) ${displayName}`;
    }

    private updateTargetStatusBar(): void {
        const target = this.workspace.currentTarget;

        if (target) {
            let icon = '$(target)';
            if (target.isExecutable) {
                icon = '$(play)';
            } else if (target.isLibrary) {
                icon = '$(package)';
            }

            this.targetStatusBarItem.text = `${icon} ${target.name}`;
        } else {
            this.targetStatusBarItem.text = '$(target) No target';
        }
    }

    setVisible(visible: boolean): void {
        if (visible) {
            this.profileStatusBarItem.show();
            this.targetStatusBarItem.show();
        } else {
            this.profileStatusBarItem.hide();
            this.targetStatusBarItem.hide();
        }
    }

    dispose(): void {
        this.profileStatusBarItem.dispose();
        this.targetStatusBarItem.dispose();
    }
}
