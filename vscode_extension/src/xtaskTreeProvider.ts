import * as vscode from 'vscode';
import { Icon } from './wasm/cargo_tools_vscode';

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

export function try_get_xtask_label(value: any[]): string | undefined {
    if (value[0] instanceof XtaskNode) {
        return value[0].label;
    }
    return undefined;
}
