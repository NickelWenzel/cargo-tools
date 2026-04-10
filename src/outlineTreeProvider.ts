import * as vscode from 'vscode';
import { OutlineNodeType, CargoOutlineTreeProviderHandler, Icon } from './wasm/cargo_tools_vscode';

export class CargoOutlineNode extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly icon: Icon,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly node_type: OutlineNodeType,
        public readonly contextValue?: string,
        public readonly description?: string,
        public readonly tooltip?: string,
        public readonly cmd?: string,
        public readonly cmd_arg?: string,
        public readonly target_package?: string,
        public readonly target?: string,
    ) {
        super(label, collapsibleState);
        this.iconPath = new vscode.ThemeIcon(icon.icon, new vscode.ThemeColor(icon.color));
        this.contextValue = contextValue;
        this.command = cmd ? {
            command: cmd,
            title: '',
            arguments: cmd_arg ? [vscode.Uri.file(cmd_arg)] : undefined,
        } : undefined;
        this.description = description;
        this.tooltip = tooltip;
        this.node_type = node_type;
        this.target_package = target_package;
        this.target = target;
    }

    static feature(
        label: string,
        icon: Icon,
        collapsibleState: vscode.TreeItemCollapsibleState,
        node_type: OutlineNodeType,
        cmd: string,
        cmd_args: string[],
    ): CargoOutlineNode {
        let node = new CargoOutlineNode(label,
            icon,
            collapsibleState,
            node_type);

        node.command = {
            command: cmd,
            title: 'Toggle feature',
            arguments: cmd_args,
        };

        return node;
    }
}

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
        return this.handler.children(element ? element.node_type : undefined);
    }
}

export function try_get_package(value: any[]): string | undefined {
    return value[0] instanceof CargoOutlineNode ? value[0].target_package : undefined;
}

export function try_get_target(value: any[]): string | undefined {
    return value[0] instanceof CargoOutlineNode ? value[0].target : undefined;
}