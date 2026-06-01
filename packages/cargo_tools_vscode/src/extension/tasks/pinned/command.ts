import { PinnedAliasNode } from './tree_provider';

export function try_get_pinned_alias_key(value: any[]): string | undefined {
    if (value[0] instanceof PinnedAliasNode) {
        return `${value[0].label}|${value[0].description}`;
    }
    return undefined;
}
