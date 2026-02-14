import * as vscode from 'vscode';
import { CargoMakeNodeHandler, CargoMakeTreeProviderHandler, Icon } from './wasm/cargo_tools_vscode';

export class CargoMakeNode extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly icon: Icon,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly contextValue: string,
        public readonly description: string,
        public readonly tooltip: string,
        public readonly handler: CargoMakeNodeHandler,
    ) {
        super(label, collapsibleState);
        this.iconPath = new vscode.ThemeIcon(icon.icon, new vscode.ThemeColor(icon.color));
        this.contextValue = contextValue;
        this.description = description;
        this.tooltip = tooltip;
        this.handler = handler;
    }

    get_handler(): CargoMakeNodeHandler {
        return this.handler;
    }
}

export function try_as_node(value: any): CargoMakeNode | undefined {
    if (!(value instanceof CargoMakeNode)) {
        return undefined;
    }

    return value;
}

export class CargoMakeTreeProvider implements vscode.TreeDataProvider<CargoMakeNode> {
    private _onDidChangeTreeData: vscode.EventEmitter<CargoMakeNode | undefined | null | void> = new vscode.EventEmitter<CargoMakeNode | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<CargoMakeNode | undefined | null | void> = this._onDidChangeTreeData.event;

    private handler: CargoMakeTreeProviderHandler;

    constructor(handler: CargoMakeTreeProviderHandler) {
        this.handler = handler;
        this.update(handler);

        // register on creation
        vscode.window.createTreeView('cargoToolsMakefile', {
            treeDataProvider: this,
            showCollapseAll: true,
            canSelectMany: false
        });
    }

    update(handler: CargoMakeTreeProviderHandler): void {
        this.handler = handler;
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
