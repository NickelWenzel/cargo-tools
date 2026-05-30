import * as vscode from 'vscode';
import { CargoMakeTreeProviderHandler, XtaskTreeProviderHandler } from './wasm/cargo_tools_vscode';
import { CargoMakeNode } from './cargoMakeTreeProvider';
import { XtaskNode } from './xtaskTreeProvider';

class SectionNode extends vscode.TreeItem {
    constructor(
        public readonly key: 'makefile' | 'alias',
        label: string,
        icon: string,
    ) {
        super(label, vscode.TreeItemCollapsibleState.Expanded);
        this.iconPath = new vscode.ThemeIcon(icon);
        this.contextValue = `${key}Section`;
    }
}

type TaskNode = SectionNode | CargoMakeNode | XtaskNode;

export class TasksTreeProvider implements vscode.TreeDataProvider<TaskNode> {
    private _onDidChangeTreeData: vscode.EventEmitter<TaskNode | undefined | null | void> =
        new vscode.EventEmitter<TaskNode | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<TaskNode | undefined | null | void> =
        this._onDidChangeTreeData.event;

    private cmHandler: CargoMakeTreeProviderHandler;
    private xtHandler: XtaskTreeProviderHandler;

    private readonly makefileSection = new SectionNode('makefile', 'Makefile Tasks', 'list-tree');
    private readonly aliasSection = new SectionNode('alias', 'Alias Tasks', 'terminal');

    constructor(cmHandler: CargoMakeTreeProviderHandler, xtHandler: XtaskTreeProviderHandler) {
        this.cmHandler = cmHandler;
        this.xtHandler = xtHandler;

        vscode.window.createTreeView('cargoToolsMakefile', {
            treeDataProvider: this,
            showCollapseAll: true,
            canSelectMany: false,
        });
    }

    update_cargo_make(handler: CargoMakeTreeProviderHandler): void {
        this.cmHandler = handler;
        this._onDidChangeTreeData.fire();
    }

    update_xtask(handler: XtaskTreeProviderHandler): void {
        this.xtHandler = handler;
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: TaskNode): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: TaskNode): Promise<TaskNode[]> {
        if (!element) {
            return [this.makefileSection, this.aliasSection];
        }
        if (element instanceof SectionNode) {
            if (element.key === 'makefile') {
                return this.cmHandler.categories() as unknown as CargoMakeNode[];
            }
            return await this.xtHandler.aliases() as unknown as XtaskNode[];
        }
        if (element instanceof CargoMakeNode) {
            return element.handler.tasks() as unknown as CargoMakeNode[];
        }
        return [];
    }
}
