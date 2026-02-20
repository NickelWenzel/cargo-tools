import * as vscode from 'vscode';
import { CargoMakeNodeHandler, CargoMakeTreeProviderHandler, CargoMakePinnedTreeProviderHandler, Icon } from './wasm/cargo_tools_vscode';

export class CargoMakeNode extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly icon: Icon,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly contextValue: string,
        public readonly description: string,
        public readonly handler: CargoMakeNodeHandler,
        public readonly tooltip?: string,
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

export class CargoMakePinnedNode extends vscode.TreeItem {
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
}

export class CargoMakePinnedTreeProvider implements vscode.TreeDataProvider<CargoMakePinnedNode> {
    private _onDidChangeTreeData: vscode.EventEmitter<CargoMakePinnedNode | undefined | null | void> = new vscode.EventEmitter<CargoMakePinnedNode | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<CargoMakePinnedNode | undefined | null | void> = this._onDidChangeTreeData.event;

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

    getTreeItem(element: CargoMakePinnedNode): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: CargoMakePinnedNode): Promise<CargoMakePinnedNode[]> {
        // There's only one level to the tree
        return this.handler.pinned_tasks();
    }
}

export function try_get_cargo_make_node_handler(value: any): CargoMakeNodeHandler | undefined {
    if (value instanceof CargoMakeNode) {
        return value.handler;
    }
    if (value instanceof CargoMakePinnedNode) {
        return value.handler;
    }

    return undefined;
}
