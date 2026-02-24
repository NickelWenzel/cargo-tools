import * as vscode from 'vscode';
import { CargoOutlineNodeHandler, CargoOutlineTreeProviderHandler, Icon } from './wasm/cargo_tools_vscode';

export class CargoOutlineNode extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly icon: Icon,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly handler: CargoOutlineNodeHandler,
        public readonly contextValue?: string,
        public readonly description?: string,
        public readonly tooltip?: string,
        public readonly cmd?: string,
        public readonly cmd_args?: string[],
    ) {
        super(label, collapsibleState);
        this.iconPath = new vscode.ThemeIcon(icon.icon, new vscode.ThemeColor(icon.color));
        this.contextValue = contextValue;
        this.command = cmd ? {
            command: cmd,
            title: '',
            arguments: cmd_args ? cmd_args : undefined,
        } : undefined;
        this.description = description;
        this.tooltip = tooltip;
        this.handler = handler;
    }
}

export class CargoOutlineTreeProvider implements vscode.TreeDataProvider<CargoOutlineNode> {
    private _onDidChangeTreeData: vscode.EventEmitter<CargoOutlineNode | undefined | null | void> = new vscode.EventEmitter<CargoOutlineNode | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<CargoOutlineNode | undefined | null | void> = this._onDidChangeTreeData.event;

    private handler: CargoOutlineTreeProviderHandler;

    constructor(handler: CargoOutlineTreeProviderHandler) {
        this.handler = handler;
        this.update(handler);

        // register on creation
        vscode.window.createTreeView('cargoToolsProjectOutline', {
            treeDataProvider: this,
            showCollapseAll: true,
            canSelectMany: false
        });
    }

    update(handler: CargoOutlineTreeProviderHandler): void {
        this.handler = handler;
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: CargoOutlineNode): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: CargoOutlineNode): Promise<CargoOutlineNode[]> {
        if (!element) {
            // Root level - show config categories
            return this.handler.children();
        }

        return element.handler.children(this.handler);
    }
}

export function try_get_cargo_node_handler(value: any): CargoOutlineNodeHandler | undefined {
    if (value instanceof CargoOutlineNode) {
        return value.handler;
    }

    return undefined;
}