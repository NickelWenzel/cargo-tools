import * as vscode from 'vscode';
import { CargoMakeNodeHandler, Icon } from '../../../../../../vscode_extension/src/wasm/cargo_tools_vscode';
import { CargoMakePinnedNode, PinnedAliasNode } from '../pinned/tree_provider';

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

export function try_get_task_label(value: any[]): string | undefined {
    if (value[0] instanceof CargoMakeNode
        || value[0] instanceof CargoMakePinnedNode
        || value[0] instanceof PinnedAliasNode) {
        return value[0].label;
    }

    return undefined;
}
