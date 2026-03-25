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
exports.SaveQuickPick = void 0;
const vscode = __importStar(require("vscode"));
class SaveQuickPick {
    static async run(selectedText, store, context) {
        // Detect language from VS Code
        const langId = vscode.window.activeTextEditor?.document.languageId || 'text';
        const name = await vscode.window.showInputBox({
            prompt: 'Component name',
            validateInput: v => (v.trim() ? undefined : 'Name is required'),
            value: this.suggestName(selectedText),
        });
        if (!name)
            return;
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
        const action = await vscode.window.showInformationMessage(`Saved "${comp.name}" (${store.count()}/10 components)`, 'View All', 'Insert Now');
        if (action === 'View All') {
            vscode.commands.executeCommand('component-crate.search');
        }
        else if (action === 'Insert Now') {
            const editor = vscode.window.activeTextEditor;
            if (editor) {
                editor.edit(editBuilder => {
                    editBuilder.insert(editor.selection.active, comp.code);
                });
            }
        }
    }
    static suggestName(code) {
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
                    if (g)
                        return g;
                }
            }
        }
        return '';
    }
}
exports.SaveQuickPick = SaveQuickPick;
