import * as vscode from 'vscode';
import { ComponentStore } from '../storage/store';

export class SaveQuickPick {
	static async run(
		selectedText: string,
		store: ComponentStore,
		context: vscode.ExtensionContext
	): Promise<void> {
		// Detect language from VS Code
		const langId = vscode.window.activeTextEditor?.document.languageId || 'text';

		const name = await vscode.window.showInputBox({
			prompt: 'Component name',
			validateInput: v => (v.trim() ? undefined : 'Name is required'),
			value: this.suggestName(selectedText),
		});
		if (!name) return;

		const tagsRaw = await vscode.window.showInputBox({
			prompt: 'Tags (comma-separated, optional)',
		});

		const tags = tagsRaw
			? tagsRaw.split(',').map(t => t.trim()).filter(Boolean)
			: [];

		const comp = store.add({
			name: name.trim(),
			code: selectedText,
			language: langId,
			tags,
			source: 'local',
			synced: false,
		});

		// Show a notification with actions
		const action = await vscode.window.showInformationMessage(
			`Saved "${comp.name}" (${store.count()}/10 components)`,
			'View All',
			'Insert Now'
		);

		if (action === 'View All') {
			vscode.commands.executeCommand('component-crate.search');
		} else if (action === 'Insert Now') {
			const editor = vscode.window.activeTextEditor;
			if (editor) {
				editor.edit(editBuilder => {
					editBuilder.insert(editor.selection.active, comp.code);
				});
			}
		}
	}

	private static suggestName(code: string): string {
		// Try to extract a useful name from the code
		// Look for function/component/class declarations
		const patterns = [
			/(?:function|const|class|export)\s+(\w+)/,
			/['"](\w+Modal)['"]/,
			/<(?:(\w+)Box|(\w+)Card|(\w+)Modal)/,
		];

		for (const pattern of patterns) {
			const m = code.match(pattern);
			if (m) {
				for (const g of m.slice(1)) {
					if (g) return g;
				}
			}
		}

		return '';
	}
}
