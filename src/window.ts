import * as vscode from 'vscode';
import { QuickPickItem } from './wasm/cargo_tools_vscode';
import { log } from 'console';

function to_items(items: QuickPickItem[]): vscode.QuickPickItem[] {
    return items.map(item => ({
        label: item.label,
        description: item.description,
        detail: item.detail,
        picked: item.picked ?? false,
    }));
}

export async function show_quick_pick(items: QuickPickItem[]): Promise<number | null> {
    log(`Show quick pick for ${JSON.stringify(items)}`);
    const vsCodeItems = to_items(items);

    const selected = await vscode.window.showQuickPick(vsCodeItems, {
        placeHolder: 'Select an option'
    });

    if (!selected) {
        return null;
    }

    return vsCodeItems.indexOf(selected);
}

export async function show_quick_pick_multiple(items: QuickPickItem[], on_pick: (item: string[]) => any): Promise<number[] | null> {
    const vsCodeItems = to_items(items);

    let initial_selected = vsCodeItems.filter(item => item.picked);

    // Create QuickPick for real-time filtering with preview
    const quickPick = vscode.window.createQuickPick();
    quickPick.placeholder = 'Select options (multi-select enabled)';
    quickPick.items = vsCodeItems;
    quickPick.selectedItems = initial_selected;
    quickPick.canSelectMany = true;

    let wasAccepted = false;
    let selected = null;

    const onChangeSelection = quickPick.onDidChangeSelection((items) => {
        let current = items.map(item => item.label.slice("$(package) ".length));
        on_pick(current);
        selected = items.map(item => vsCodeItems.indexOf(item));
    });

    quickPick.onDidAccept(() => {
        wasAccepted = true;
        quickPick.hide();
    });

    quickPick.show();

    // await hiding the quick pick
    await new Promise<void>(resolve => {
        const onHide = quickPick.onDidHide(() => {
            // If user canceled (didn't accept), restore original filter
            if (!wasAccepted) {
                selected = null;
            }

            // onChangeValue.dispose();
            onChangeSelection.dispose();
            quickPick.dispose();
            onHide.dispose();
            resolve();
        });
    });

    return selected;
}

export async function show_quick_pick_type(current: string, items: QuickPickItem[], on_type: (filter: string) => void): Promise<string | null> {
    const vsCodeItems = to_items(items);

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
            quickPick.items = vsCodeItems;
        } else {
            // Filter and show matching members
            const matchingItems = vsCodeItems.filter(item =>
                item.label.toLowerCase().includes(filter)
            );

            quickPick.items = matchingItems;
        }
    };

    // Initial population
    updateItems(quickPick.value);

    const onChange = quickPick.onDidChangeValue((value) => {
        on_type(value);
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
