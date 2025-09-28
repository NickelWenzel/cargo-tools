import * as vscode from 'vscode';
import { CargoTarget } from './cargoTarget';

/**
 * This class keeps track of all state that needs to persist between sessions
 * within a single workspace. Objects that wish to persist state should store
 * it here to ensure that we keep state consistent.
 *
 * This uses VSCode's Memento objects to ensure consistency. The user cannot
 * easily modify the contents of a Memento, so we can be sure that the contents
 * won't be torn or invalid, unless we make them that way. This class prevents
 * invalid states.
 */
export class StateManager {
    constructor(
        readonly extensionContext: vscode.ExtensionContext,
        readonly folder: vscode.WorkspaceFolder
    ) { }

    private _get<T>(key: string, folderName: string, isMultiProject: boolean): T | undefined {
        return isMultiProject
            ? this.extensionContext.globalState.get<T>(this.folder.uri.fsPath + `${folderName} ` + key)
            : this.extensionContext.globalState.get<T>(this.folder.uri.fsPath + key);
    }

    private _update(key: string, value: any, folderName: string, isMultiProject: boolean): Thenable<void> {
        return isMultiProject
            ? this.extensionContext.globalState.update(this.folder.uri.fsPath + `${folderName} ` + key, value)
            : this.extensionContext.globalState.update(this.folder.uri.fsPath + key, value);
    }

    // Project Status View State - User selections in the project status tree

    /**
     * The currently selected package in the workspace
     */
    getSelectedPackage(folderName: string, isMultiProject: boolean): string | undefined {
        return this._get<string>('selectedPackage', folderName, isMultiProject);
    }

    async setSelectedPackage(folderName: string, packageName: string | undefined, isMultiProject: boolean) {
        await this._update('selectedPackage', packageName, folderName, isMultiProject);
    }

    /**
     * The currently selected build target ID
     */
    getSelectedBuildTargetId(folderName: string, isMultiProject: boolean): string | null {
        return this._get<string>('selectedBuildTarget', folderName, isMultiProject) || null;
    }

    async setSelectedBuildTarget(folderName: string, target: CargoTarget | null, isMultiProject: boolean) {
        const targetId = target ? target.id : null;
        await this._update('selectedBuildTarget', targetId, folderName, isMultiProject);
    }

    /**
     * The currently selected run target ID
     */
    getSelectedRunTargetId(folderName: string, isMultiProject: boolean): string | null {
        return this._get<string>('selectedRunTarget', folderName, isMultiProject) || null;
    }

    async setSelectedRunTarget(folderName: string, target: CargoTarget | null, isMultiProject: boolean) {
        const targetId = target ? target.id : null;
        await this._update('selectedRunTarget', targetId, folderName, isMultiProject);
    }

    /**
     * The currently selected benchmark target ID
     */
    getSelectedBenchmarkTargetId(folderName: string, isMultiProject: boolean): string | null {
        return this._get<string>('selectedBenchmarkTarget', folderName, isMultiProject) || null;
    }

    async setSelectedBenchmarkTarget(folderName: string, target: CargoTarget | null, isMultiProject: boolean) {
        const targetId = target ? target.id : null;
        await this._update('selectedBenchmarkTarget', targetId, folderName, isMultiProject);
    }

    /**
     * The currently selected platform target
     */
    getSelectedPlatformTarget(folderName: string, isMultiProject: boolean): string | null {
        return this._get<string>('selectedPlatformTarget', folderName, isMultiProject) || null;
    }

    async setSelectedPlatformTarget(folderName: string, targetName: string | null, isMultiProject: boolean) {
        await this._update('selectedPlatformTarget', targetName, folderName, isMultiProject);
    }

    /**
     * The currently selected features as an array of feature names
     */
    getSelectedFeatures(folderName: string, isMultiProject: boolean): string[] {
        return this._get<string[]>('selectedFeatures', folderName, isMultiProject) || [];
    }

    async setSelectedFeatures(folderName: string, features: string[], isMultiProject: boolean) {
        await this._update('selectedFeatures', features, folderName, isMultiProject);
    }

    /**
     * The currently selected profile name
     */
    getSelectedProfile(folderName: string, isMultiProject: boolean): string | null {
        return this._get<string>('selectedProfile', folderName, isMultiProject) || null;
    }

    async setSelectedProfile(folderName: string, profileName: string | null, isMultiProject: boolean) {
        await this._update('selectedProfile', profileName, folderName, isMultiProject);
    }

    // Project Outline View State - Filter and grouping settings

    /**
     * Whether to group targets by workspace member
     */
    getGroupByWorkspaceMember(folderName: string, isMultiProject: boolean): boolean {
        return this._get<boolean>('groupByWorkspaceMember', folderName, isMultiProject) ?? true;
    }

    async setGroupByWorkspaceMember(folderName: string, groupBy: boolean, isMultiProject: boolean) {
        await this._update('groupByWorkspaceMember', groupBy, folderName, isMultiProject);
    }

    /**
     * The workspace member filter string
     */
    getWorkspaceMemberFilter(folderName: string, isMultiProject: boolean): string {
        return this._get<string>('workspaceMemberFilter', folderName, isMultiProject) || '';
    }

    async setWorkspaceMemberFilter(folderName: string, filter: string, isMultiProject: boolean) {
        await this._update('workspaceMemberFilter', filter, folderName, isMultiProject);
    }

    /**
     * The target type filter as an array of target types
     */
    getTargetTypeFilter(folderName: string, isMultiProject: boolean): string[] {
        return this._get<string[]>('targetTypeFilter', folderName, isMultiProject) || ['bin', 'lib', 'example', 'bench'];
    }

    async setTargetTypeFilter(folderName: string, targetTypes: string[], isMultiProject: boolean) {
        await this._update('targetTypeFilter', targetTypes, folderName, isMultiProject);
    }

    /**
     * Whether the target type filter is active
     */
    getIsTargetTypeFilterActive(folderName: string, isMultiProject: boolean): boolean {
        return this._get<boolean>('isTargetTypeFilterActive', folderName, isMultiProject) ?? false;
    }

    async setIsTargetTypeFilterActive(folderName: string, isActive: boolean, isMultiProject: boolean) {
        await this._update('isTargetTypeFilterActive', isActive, folderName, isMultiProject);
    }

    /**
     * Whether to show features in the project outline
     */
    getShowFeatures(folderName: string, isMultiProject: boolean): boolean {
        return this._get<boolean>('showFeatures', folderName, isMultiProject) ?? true;
    }

    async setShowFeatures(folderName: string, showFeatures: boolean, isMultiProject: boolean) {
        await this._update('showFeatures', showFeatures, folderName, isMultiProject);
    }

    // Makefile View State - Filter settings for the Makefile view

    /**
     * The current task filter string
     */
    getMakefileTaskFilter(folderName: string, isMultiProject: boolean): string {
        return this._get<string>('makefileTaskFilter', folderName, isMultiProject) || '';
    }

    async setMakefileTaskFilter(folderName: string, filter: string, isMultiProject: boolean) {
        await this._update('makefileTaskFilter', filter, folderName, isMultiProject);
    }

    /**
     * The current category filter set
     */
    getMakefileCategoryFilter(folderName: string, isMultiProject: boolean): string[] {
        return this._get<string[]>('makefileCategoryFilter', folderName, isMultiProject) || [];
    }

    async setMakefileCategoryFilter(folderName: string, categories: string[], isMultiProject: boolean) {
        await this._update('makefileCategoryFilter', categories, folderName, isMultiProject);
    }

    /**
     * Whether the category filter is active (not showing all categories)
     */
    getIsMakefileCategoryFilterActive(folderName: string, isMultiProject: boolean): boolean {
        return this._get<boolean>('isMakefileCategoryFilterActive', folderName, isMultiProject) ?? false;
    }

    async setIsMakefileCategoryFilterActive(folderName: string, isActive: boolean, isMultiProject: boolean) {
        await this._update('isMakefileCategoryFilterActive', isActive, folderName, isMultiProject);
    }

    // Pinned Makefile Tasks State - List of pinned makefile tasks

    /**
     * The list of pinned makefile tasks
     */
    getPinnedMakefileTasks(folderName: string, isMultiProject: boolean): string[] {
        return this._get<string[]>('pinnedMakefileTasks', folderName, isMultiProject) || [];
    }

    async setPinnedMakefileTasks(folderName: string, tasks: string[], isMultiProject: boolean) {
        await this._update('pinnedMakefileTasks', tasks, folderName, isMultiProject);
    }

    /**
     * Reset all current workspace state. Mostly for troubleshooting
     */
    async reset(folderName: string, isMultiProject: boolean) {
        // Project Status View state
        await this.setSelectedPackage(folderName, undefined, isMultiProject);
        await this.setSelectedBuildTarget(folderName, null, isMultiProject);
        await this.setSelectedRunTarget(folderName, null, isMultiProject);
        await this.setSelectedBenchmarkTarget(folderName, null, isMultiProject);
        await this.setSelectedPlatformTarget(folderName, null, isMultiProject);
        await this.setSelectedFeatures(folderName, [], isMultiProject);
        await this.setSelectedProfile(folderName, null, isMultiProject);

        // Project Outline View state
        await this.setGroupByWorkspaceMember(folderName, true, isMultiProject);
        await this.setWorkspaceMemberFilter(folderName, '', isMultiProject);
        await this.setTargetTypeFilter(folderName, ['bin', 'lib', 'example', 'bench'], isMultiProject);
        await this.setIsTargetTypeFilterActive(folderName, false, isMultiProject);
        await this.setShowFeatures(folderName, true, isMultiProject);

        // Makefile View state
        await this.setMakefileTaskFilter(folderName, '', isMultiProject);
        await this.setMakefileCategoryFilter(folderName, [], isMultiProject);
        await this.setIsMakefileCategoryFilterActive(folderName, false, isMultiProject);

        // Pinned Makefile Tasks state
        await this.setPinnedMakefileTasks(folderName, [], isMultiProject);
    }
}
