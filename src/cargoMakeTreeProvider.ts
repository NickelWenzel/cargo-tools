import * as vscode from 'vscode';
import { CargoMakeNodeHandler, CargoMakeTreeProviderHandler } from './wasm/cargo_tools_vscode';

export class CargoMakeNode extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly description: string,
        public readonly tooltip: string,
        public readonly handler: CargoMakeNodeHandler,
    ) {
        super(label, collapsibleState);
        this.description = description;
        this.tooltip = tooltip;
        this.handler = handler;
    }
}

export class CargoMakeTreeProvider implements vscode.TreeDataProvider<CargoMakeNode> {
    private _onDidChangeTreeData: vscode.EventEmitter<CargoMakeNode | undefined | null | void> = new vscode.EventEmitter<CargoMakeNode | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<CargoMakeNode | undefined | null | void> = this._onDidChangeTreeData.event;

    private handler: CargoMakeTreeProviderHandler;

    constructor(handler: CargoMakeTreeProviderHandler) {
        this.handler = handler;
        this.update();
    }

    update(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: CargoMakeNode): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: CargoMakeNode): Promise<CargoMakeNode[]> {
        if (!element) {
            // Root level - show categories
            return this.handler.categories();
        }

        return element.handler.tasks();
    }
}
