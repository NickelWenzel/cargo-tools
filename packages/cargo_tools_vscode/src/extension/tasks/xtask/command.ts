import { XtaskNode } from './tree_provider';

export function try_get_xtask_label(value: any[]): string | undefined {
    if (value[0] instanceof XtaskNode) {
        return value[0].label;
    }
    return undefined;
}
