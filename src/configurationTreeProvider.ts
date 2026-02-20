import * as vscode from 'vscode';
import { CargoNodeHandler, CargoConfigurationTreeProviderHandler, Icon } from './wasm/cargo_tools_vscode';

export class CargoNode extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly icon: Icon,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly handler: CargoNodeHandler,
        public readonly contextValue?: string,
        public readonly description?: string,
        public readonly tooltip?: string,
        public readonly cmd?: string,
        public readonly cmd_arg?: string,
    ) {
        super(label, collapsibleState);
        this.iconPath = new vscode.ThemeIcon(icon.icon, new vscode.ThemeColor(icon.color));
        this.contextValue = contextValue;
        this.command = cmd ? {
            command: cmd,
            title: '',
            arguments: cmd_arg ? [cmd_arg] : undefined,
        } : undefined;
        this.description = description;
        this.tooltip = tooltip;
        this.handler = handler;
    }
}

export class CargoConfigurationTreeProvider implements vscode.TreeDataProvider<CargoNode> {
    private _onDidChangeTreeData: vscode.EventEmitter<CargoNode | undefined | null | void> = new vscode.EventEmitter<CargoNode | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<CargoNode | undefined | null | void> = this._onDidChangeTreeData.event;

    private handler: CargoConfigurationTreeProviderHandler;

    constructor(handler: CargoConfigurationTreeProviderHandler) {
        this.handler = handler;
        this.update(handler);

        // register on creation
        vscode.window.createTreeView('cargoToolsConfiguration', {
            treeDataProvider: this,
            showCollapseAll: false,
            canSelectMany: false
        });
    }

    update(handler: CargoConfigurationTreeProviderHandler): void {
        this.handler = handler;
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: CargoNode): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: CargoNode): Promise<CargoNode[]> {
        if (!element) {
            // Root level - show config categories
            return this.handler.children();
        }

        return element.handler.children(this.handler);
    }
}

export function try_get_cargo_node_handler(value: any): CargoNodeHandler | undefined {
    if (value instanceof CargoNode) {
        return value.handler;
    }

    return undefined;
}