import * as vscode from 'vscode';
import { NodeType, Icon } from '../../../../../../vscode_extension/src/wasm/cargo_tools_vscode';

export class CargoNode extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly icon: Icon,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly node_type: NodeType,
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
        this.node_type = node_type;
    }
}
