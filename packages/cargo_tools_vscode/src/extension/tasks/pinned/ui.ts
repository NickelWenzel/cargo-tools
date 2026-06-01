import * as vscode from 'vscode';
import { CargoMakePinnedTreeProviderHandler } from '../../../../../../vscode_extension/src/wasm/cargo_tools_vscode';
import { CargoMakePinnedNode, PinnedAliasNode } from './tree_provider';

export class CargoMakePinnedTreeProvider implements vscode.TreeDataProvider<CargoMakePinnedNode | PinnedAliasNode> {
    private _onDidChangeTreeData: vscode.EventEmitter<CargoMakePinnedNode | PinnedAliasNode | undefined | null | void> =
        new vscode.EventEmitter<CargoMakePinnedNode | PinnedAliasNode | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<CargoMakePinnedNode | PinnedAliasNode | undefined | null | void> =
        this._onDidChangeTreeData.event;

    private handler: CargoMakePinnedTreeProviderHandler;

    constructor(handler: CargoMakePinnedTreeProviderHandler) {
        this.handler = handler;
        this.update(handler);

        // register on creation
        vscode.window.createTreeView('cargoToolsPinnedMakefileTasks', {
            treeDataProvider: this,
            showCollapseAll: false,
            canSelectMany: false
        });
    }

    update(handler: CargoMakePinnedTreeProviderHandler): void {
        this.handler = handler;
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: CargoMakePinnedNode | PinnedAliasNode): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: CargoMakePinnedNode | PinnedAliasNode): Promise<(CargoMakePinnedNode | PinnedAliasNode)[]> {
        if (element) { return []; }
        return [
            ...this.handler.pinned_tasks() as unknown as CargoMakePinnedNode[],
            ...this.handler.pinned_aliases() as unknown as PinnedAliasNode[],
        ];
    }
}

export async function showInformationMessage(message: string, items: string[]): Promise<string | undefined> {
    return await vscode.window.showInformationMessage(message, ...items);
}
