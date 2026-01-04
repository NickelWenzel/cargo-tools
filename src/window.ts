import * as vscode from 'vscode';
import { QuickPickItem } from './wasm/cargo_tools_vscode';

export async function show_quick_pick(items: QuickPickItem[]): Promise<number | null> {
    const vsCodeItems: vscode.QuickPickItem[] = items.map(item => ({
        label: item.label(),
        description: item.description(),
        detail: item.detail(),
    }));

    const selected = await vscode.window.showQuickPick(vsCodeItems, {
        placeHolder: 'Select an option'
    });

    if (!selected) {
        return null;
    }

    return vsCodeItems.indexOf(selected);
}
