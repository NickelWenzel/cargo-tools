import * as vscode from 'vscode';
import { OutlineNodeType, Icon } from '../../../../../../vscode_extension/src/wasm/cargo_tools_vscode';

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
