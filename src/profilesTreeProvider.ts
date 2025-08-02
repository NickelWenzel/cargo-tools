import * as vscode from 'vscode';
import { CargoWorkspace } from './cargoWorkspace';
import { CargoProfile } from './cargoProfile';

export class ProfileTreeItem extends vscode.TreeItem {
    constructor(
        public readonly profile: CargoProfile,
        public readonly isActive: boolean
    ) {
        super(CargoProfile.getDisplayName(profile), vscode.TreeItemCollapsibleState.None);
        
        this.tooltip = CargoProfile.getDescription(profile);
        this.contextValue = 'cargoProfile';
        this.command = {
            command: 'cargo-tools.selectProfile',
            title: 'Select Profile',
            arguments: [profile]
        };

        if (isActive) {
            this.iconPath = new vscode.ThemeIcon('check');
            this.description = '(active)';
        }
    }
}

export class ProfilesTreeProvider implements vscode.TreeDataProvider<ProfileTreeItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<ProfileTreeItem | undefined | null | void>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    constructor(private workspace: CargoWorkspace) {
        workspace.onDidChangeProfile(() => {
            this._onDidChangeTreeData.fire();
        });
    }

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: ProfileTreeItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: ProfileTreeItem): Thenable<ProfileTreeItem[]> {
        if (!element) {
            // Root level - return all profiles
            const profiles = CargoProfile.getAllProfiles();
            const currentProfile = this.workspace.currentProfile;
            
            return Promise.resolve(
                profiles.map(profile => new ProfileTreeItem(profile, profile === currentProfile))
            );
        }
        
        return Promise.resolve([]);
    }
}
