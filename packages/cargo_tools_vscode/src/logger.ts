import { log } from '../../../vscode_extension/src/extension';

export function log_debug(msg: string) {
    try { return log.debug(msg); } catch (error) { }
}

export function log_info(msg: string) {
    try { return log.info(msg); } catch (error) { }
}

export function log_warn(msg: string) {
    try { return log.warn(msg); } catch (error) { }
}

export function log_error(msg: string) {
    try { return log.error(msg); } catch (error) { }
}

export function log_trace(msg: string) {
    try { return log.trace(msg); } catch (error) { }
}
