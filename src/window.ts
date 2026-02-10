import * as vscode from 'vscode';
import { QuickPickItem } from './wasm/cargo_tools_vscode';
import { log } from 'console';

export async function show_quick_pick(items: QuickPickItem[]): Promise<number | null> {
    log(`Show quick pick for ${JSON.stringify(items)}`);
    const vsCodeItems: vscode.QuickPickItem[] = items.map(item => ({
        label: item.label,
        description: item.description,
        detail: item.detail,
        picked: item.picked ?? false,
    }));

    const selected = await vscode.window.showQuickPick(vsCodeItems, {
        placeHolder: 'Select an option'
    });

    if (!selected) {
        return null;
    }

    return vsCodeItems.indexOf(selected);
}

export async function show_quick_pick_multiple(items: QuickPickItem[]): Promise<number[] | null> {
    const vsCodeItems: vscode.QuickPickItem[] = items.map(item => ({
        label: item.label,
        description: item.description,
        detail: item.detail,
        picked: item.picked ?? false,
    }));

    const selected = await vscode.window.showQuickPick(vsCodeItems, {
        placeHolder: 'Select options (multi-select enabled)',
        canPickMany: true
    });

    if (!selected || selected.length === 0) {
        return null;
    }

    return selected.map(item => vsCodeItems.indexOf(item));
}

export async function show_quick_pick_type(current: string, items: QuickPickItem[], on_change_callback: (filter: string) => void): Promise<string | null> {
    const vsCodeItems: vscode.QuickPickItem[] = items.map(item => ({
        label: `$(package) ${item.label}`,
        description: item.description,
        detail: item.detail,
        picked: item.picked ?? false,
    }));

    // Create QuickPick for real-time filtering with preview
    const quickPick = vscode.window.createQuickPick();
    quickPick.placeholder = 'Type to filter, press Enter to apply';
    quickPick.value = current;
    quickPick.matchOnDescription = true;
    quickPick.matchOnDetail = true;
    quickPick.selectedItems = [];

    let filter = null;
    let wasAccepted = false;

    // Function to update QuickPick items based on current filter
    const updateItems = (filterValue: string) => {
        const filter = filterValue.toLowerCase().trim();

        if (!filter) {
            // Show all members when no filter
            quickPick.items = items;
        } else {
            // Filter and show matching members
            const matchingItems = items.filter(item =>
                item.label.toLowerCase().includes(filter)
            );

            quickPick.items = matchingItems;
        }
    };

    // Initial population
    updateItems(quickPick.value);

    const onChange = quickPick.onDidChangeValue((value) => {
        on_change_callback(value);
        updateItems(value);
        // Clear any selections after updating items
        quickPick.selectedItems = [];
    });

    quickPick.onDidAccept(() => {
        wasAccepted = true;
        // Always use the typed value as filter since items are unselectable
        filter = quickPick.value;
        quickPick.hide();
    });

    quickPick.show();

    // await hiding the quick pick
    await new Promise<void>(resolve => {
        const onHide = quickPick.onDidHide(() => {
            // If user canceled (didn't accept), restore original filter
            if (!wasAccepted) {
                filter = null;
            }

            onChange.dispose();
            quickPick.dispose();
            onHide.dispose();
            resolve();
        });
    });

    return filter;
}
