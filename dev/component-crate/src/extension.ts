import * as vscode from 'vscode';
import { ComponentStore } from './storage/store';
import { GitHubImporter } from './github/api';
import { SearchPanel } from './webview/searchPanel';
import { SaveQuickPick } from './commands/save';
import { UpgradeManager } from './upgrade';

let store: ComponentStore;
let github: GitHubImporter;
let upgradeManager: UpgradeManager;

export function activate(context: vscode.ExtensionContext) {
	store = new ComponentStore(context.globalState);
	github = new GitHubImporter();
	upgradeManager = new UpgradeManager(store);

	// Search command - opens the search panel
	const searchCmd = vscode.commands.registerCommand('component-crate.search', () => {
		SearchPanel.createOrShow(context.extensionUri, store);
	});

	// Save command - saves selected text as a component
	const saveCmd = vscode.commands.registerCommand('component-crate.save', async () => {
		const editor = vscode.window.activeTextEditor;
		if (!editor) {
			vscode.window.showWarningMessage('No active editor');
			return;
		}

		const selection = editor.selection;
		const selectedText = editor.document.getText(selection);

		if (!selectedText.trim()) {
			vscode.window.showWarningMessage('No text selected');
			return;
		}

		// Check freemium limit
		const limitCheck = upgradeManager.checkLimit();
		if (!limitCheck.allowed) {
			const choice = await vscode.window.showWarningMessage(
				limitCheck.message ?? 'Upgrade required to save more components.',
				'Upgrade Now',
				'Cancel'
			);
			if (choice === 'Upgrade Now') {
				vscode.commands.executeCommand('component-crate.upgrade');
			}
			return;
		}

		await SaveQuickPick.run(selectedText, store, context);
	});

	// Insert command - insert a saved component
	const insertCmd = vscode.commands.registerCommand('component-crate.insert', async () => {
		const components = store.getAll();
		if (components.length === 0) {
			vscode.window.showInformationMessage('No saved components. Use "Save to Crate" first.');
			return;
		}

		const items = components.map(c => ({
			label: c.name,
			description: c.tags.join(', ') || c.language,
			component: c,
		}));

		const picked = await vscode.window.showQuickPick(items, {
			placeHolder: 'Select a component to insert',
		});

		if (picked) {
			const editor = vscode.window.activeTextEditor;
			if (editor) {
				editor.edit(editBuilder => {
					editBuilder.insert(editor.selection.active, picked.component.code);
				});
				vscode.window.showInformationMessage(`Inserted: ${picked.label}`);
			}
		}
	});

	// Import from GitHub command
	const importCmd = vscode.commands.registerCommand('component-crate.importFromGithub', async () => {
		const limitCheck = upgradeManager.checkLimit();
		if (!limitCheck.allowed) {
			vscode.window.showWarningMessage(limitCheck.message ?? 'Upgrade required.', 'Upgrade Now').then(choice => {
				if (choice === 'Upgrade Now') {
					vscode.commands.executeCommand('component-crate.upgrade');
				}
			});
			return;
		}

		await github.importFromRepo(store, context);
	});

	// Upgrade command
	const upgradeCmd = vscode.commands.registerCommand('component-crate.upgrade', () => {
		vscode.env.openExternal(
			vscode.Uri.parse('https://componentcrate.com/pricing')
		);
	});

	// Open dashboard
	const dashboardCmd = vscode.commands.registerCommand('component-crate.openDashboard', () => {
		SearchPanel.createOrShow(context.extensionUri, store);
	});

	// Update status bar with component count
	updateStatusBar(store);

	context.subscriptions.push(
		searchCmd, saveCmd, insertCmd, importCmd, upgradeCmd, dashboardCmd
	);
}

function updateStatusBar(store: ComponentStore) {
	const count = store.count();
	const barItem = vscode.window.createStatusBarItem(
		vscode.StatusBarAlignment.Right,
		100
	);
	barItem.text = `$(box) Crate: ${count}/10`;
	barItem.command = 'component-crate.search';
	barItem.tooltip = `${count} components saved. Click to search.`;
	barItem.show();

	// Refresh when store changes
	store.onDidChange(() => {
		const newCount = store.count();
		barItem.text = `$(box) Crate: ${newCount}/10`;
	});
}

export function deactivate() {}
