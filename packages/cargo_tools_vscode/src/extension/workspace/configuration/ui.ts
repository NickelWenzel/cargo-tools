import * as vscode from 'vscode';
import { CargoConfigurationTreeProviderHandler } from '../../../../../../vscode_extension/src/wasm/cargo_tools_vscode';
import { CargoNode } from './treeprovider';

export class CargoConfigurationTreeProvider implements vscode.TreeDataProvider<CargoNode> {
    private _onDidChangeTreeData: vscode.EventEmitter<CargoNode | undefined | null | void> = new vscode.EventEmitter<CargoNode | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<CargoNode | undefined | null | void> = this._onDidChangeTreeData.event;

    private handler: CargoConfigurationTreeProviderHandler;

    constructor(handler: CargoConfigurationTreeProviderHandler) {
        this.handler = handler;

        // register on creation
        vscode.window.createTreeView('cargoToolsConfiguration', {
            treeDataProvider: this,
            showCollapseAll: false,
            canSelectMany: false
        });
    }

    update(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: CargoNode): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: CargoNode): Promise<CargoNode[]> {
        return this.handler.children(element ? element.node_type : undefined);
    }
}

export function get_rust_analyzer_check_targets(): string[] {
    let config = vscode.workspace.getConfiguration('rust-analyzer');
    return config.get('check.targets', []) || [];
}

export async function update_rust_analyzer_check_targets(targets: string[]) {
    let config = vscode.workspace.getConfiguration('rust-analyzer');

    if (targets.length === 0) {
        // Remove setting if no targets selected
        await config.update('check.targets', undefined, vscode.ConfigurationTarget.Workspace);
    } else {
        // Set the new targets
        await config.update('check.targets', targets, vscode.ConfigurationTarget.Workspace);
    }
}
