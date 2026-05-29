import * as vscode from 'vscode';
import { XtaskTreeProviderHandler, Icon } from './wasm/cargo_tools_vscode';

export class XtaskNode extends vscode.TreeItem {
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

export class XtaskTreeProvider implements vscode.TreeDataProvider<XtaskNode> {
    private _onDidChangeTreeData: vscode.EventEmitter<XtaskNode | undefined | null | void> =
        new vscode.EventEmitter<XtaskNode | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<XtaskNode | undefined | null | void> =
        this._onDidChangeTreeData.event;

    private handler: XtaskTreeProviderHandler;

    constructor(handler: XtaskTreeProviderHandler) {
        this.handler = handler;

        vscode.window.createTreeView('cargoToolsXtaskAliases', {
            treeDataProvider: this,
            showCollapseAll: false,
            canSelectMany: false,
        });
    }

    update(handler: XtaskTreeProviderHandler): void {
        this.handler = handler;
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: XtaskNode): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: XtaskNode): Promise<XtaskNode[]> {
        if (element) { return []; }
        return await this.handler.aliases() as unknown as XtaskNode[];
    }
}

export function try_get_xtask_label(value: any[]): string | undefined {
    if (value[0] instanceof XtaskNode) {
        return value[0].label;
    }
    return undefined;
}
