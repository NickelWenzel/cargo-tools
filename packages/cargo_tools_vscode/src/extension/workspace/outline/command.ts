import { CargoOutlineNode } from './treeprovider';

export function try_get_node_type(value: any[]): any | undefined {
    return value[0] instanceof CargoOutlineNode ? value[0].node_type.cloned() : undefined;
}
