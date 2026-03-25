"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const store_1 = require("./storage/store");
const api_1 = require("./github/api");
const searchPanel_1 = require("./webview/searchPanel");
const save_1 = require("./commands/save");
const upgrade_1 = require("./upgrade");
let store;
let github;
let upgradeManager;
function activate(context) {
    store = new store_1.ComponentStore(context.globalState);
    github = new api_1.GitHubImporter();
    upgradeManager = new upgrade_1.UpgradeManager(store);
    // Search command - opens the search panel
    const searchCmd = vscode.commands.registerCommand('component-crate.search', () => {
        searchPanel_1.SearchPanel.createOrShow(context.extensionUri, store);
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
            const choice = await vscode.window.showWarningMessage(limitCheck.message ?? 'Upgrade required to save more components.', 'Upgrade Now', 'Cancel');
            if (choice === 'Upgrade Now') {
                vscode.commands.executeCommand('component-crate.upgrade');
            }
            return;
        }
        await save_1.SaveQuickPick.run(selectedText, store, context);
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
        vscode.env.openExternal(vscode.Uri.parse('https://componentcrate.com/pricing'));
    });
    // Open dashboard
    const dashboardCmd = vscode.commands.registerCommand('component-crate.openDashboard', () => {
        searchPanel_1.SearchPanel.createOrShow(context.extensionUri, store);
    });
    // Update status bar with component count
    updateStatusBar(store);
    context.subscriptions.push(searchCmd, saveCmd, insertCmd, importCmd, upgradeCmd, dashboardCmd);
}
function updateStatusBar(store) {
    const count = store.count();
    const barItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
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
function deactivate() { }
