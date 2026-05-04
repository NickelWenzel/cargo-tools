import * as vscode from 'vscode';
import { run, ExitToken } from './wasm/cargo_tools_vscode';

export let extension_context: vscode.ExtensionContext | undefined;
export let log = vscode.window.createOutputChannel("cargo-tools", { log: true });

let exit: ExitToken | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<any> {
	// This is important because context is used as a global variable in the typescript code
	extension_context = context;
	context.subscriptions.push(log);
	try {
		log.info('Cargo Tools extension activation started...');

		const workspaceFolder = vscode.workspace.workspaceFolders?.[0].uri.fsPath;
		if (!workspaceFolder) {
			throw new Error('No workspace folder found');
		}

		exit = run(workspaceFolder);

		log.info('Cargo Tools extension fully initialized!');
		return {};
	} catch (error) {
		await vscode.commands.executeCommand('setContext', 'cargoTools:workspaceHasCargo', false);
		await vscode.commands.executeCommand('setContext', 'cargoTools:workspaceHasMakefile', false);
		log.error('Failed to activate Cargo Tools extension:', error);
		vscode.window.showErrorMessage(`Cargo Tools extension failed to activate: ${error}`);
		throw error;
	}
}

export async function deactivate(): Promise<void> {
	log.info('Deactivating Cargo Tools extension...');

	try {
		exit?.exit();

		log.info('Cargo Tools extension deactivated successfully');
	} catch (error) {
		log.error('Error during Cargo Tools extension deactivation:', error);
	}
}
