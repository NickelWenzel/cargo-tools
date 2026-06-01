import * as vscode from 'vscode';
import { CargoOutlineTreeProviderHandler } from '../../../../../../vscode_extension/src/wasm/cargo_tools_vscode';
import { CargoOutlineNode } from './treeprovider';

export class CargoOutlineTreeProvider implements vscode.TreeDataProvider<CargoOutlineNode> {
    private _onDidChangeTreeData: vscode.EventEmitter<CargoOutlineNode | undefined | null | void> = new vscode.EventEmitter<CargoOutlineNode | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<CargoOutlineNode | undefined | null | void> = this._onDidChangeTreeData.event;

    private handler: CargoOutlineTreeProviderHandler;

    constructor(handler: CargoOutlineTreeProviderHandler) {
        this.handler = handler;

        // register on creation
        vscode.window.createTreeView('cargoToolsProjectOutline', {
            treeDataProvider: this,
            showCollapseAll: true,
            canSelectMany: false
        });
    }

    update(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: CargoOutlineNode): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: CargoOutlineNode): Promise<CargoOutlineNode[]> {
        return this.handler.children(element ? element.node_type.cloned() : undefined);
    }
}
