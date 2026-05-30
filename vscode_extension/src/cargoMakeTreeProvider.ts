import * as vscode from 'vscode';
import { CargoMakeNodeHandler, CargoMakePinnedTreeProviderHandler, Icon } from './wasm/cargo_tools_vscode';

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

export class PinnedAliasNode extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly icon: Icon,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly contextValue: string,
        public readonly description: string,
        public readonly tooltip: string,
    ) {
        super(label, collapsibleState);
        this.iconPath = new vscode.ThemeIcon(icon.icon, new vscode.ThemeColor(icon.color));
        this.contextValue = contextValue;
        this.description = description;
        this.tooltip = tooltip;
    }
}

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

export function try_get_task_label(value: any[]): string | undefined {
    if (value[0] instanceof CargoMakeNode
        || value[0] instanceof CargoMakePinnedNode
        || value[0] instanceof PinnedAliasNode) {
        return value[0].label;
    }

    return undefined;
}

export function try_get_pinned_alias_key(value: any[]): string | undefined {
    if (value[0] instanceof PinnedAliasNode) {
        return `${value[0].label}|${value[0].description}`;
    }
    return undefined;
}
