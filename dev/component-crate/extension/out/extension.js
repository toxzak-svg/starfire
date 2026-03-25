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
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const vscode = __importStar(require("vscode"));
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const THEME = {
    dark: '#0d0d0d',
    panel: '#1a1a1a',
    border: '#333',
    accent: '#BFFF00',
    text: '#e0e0e0',
    textDim: '#888',
    success: '#00ff88',
};
function getCratePath() {
    const configDir = path.join(process.env.HOME || process.env.USERPROFILE || '', '.component-crate');
    if (!fs.existsSync(configDir)) {
        fs.mkdirSync(configDir, { recursive: true });
    }
    return path.join(configDir, 'crate.json');
}
function loadCrate() {
    const cratePath = getCratePath();
    if (fs.existsSync(cratePath)) {
        try {
            return JSON.parse(fs.readFileSync(cratePath, 'utf8'));
        }
        catch {
            return [];
        }
    }
    return [];
}
function saveCrate(components) {
    fs.writeFileSync(getCratePath(), JSON.stringify(components, null, 2));
}
function activate(context) {
    // Open Crate Command
    const openCommand = vscode.commands.registerCommand('component-crate.open', async () => {
        const components = loadCrate();
        // Create sleek webview
        const panel = vscode.window.createWebviewPanel('componentCrate', 'Component Crate', vscode.ViewColumn.One, {
            enableScripts: true,
        });
        panel.webview.html = getWebviewHtml(components);
        // Handle messages from webview
        panel.webview.onDidReceiveMessage(async (message) => {
            if (message.command === 'insert') {
                const editor = vscode.window.activeTextEditor;
                if (editor) {
                    editor.insertSnippet(new vscode.SnippetString(message.code));
                }
            }
            else if (message.command === 'delete') {
                const components = loadCrate().filter(c => c.name !== message.name);
                saveCrate(components);
                panel.webview.html = getWebviewHtml(components);
            }
        });
    });
    // Search Command
    const searchCommand = vscode.commands.registerCommand('component-crate.search', async () => {
        const components = loadCrate();
        if (components.length === 0) {
            vscode.window.showInformationMessage('Your crate is empty! Use "Save Selection" to add components.');
            return;
        }
        const items = components.map(c => ({
            label: c.name,
            detail: c.tags.join(', '),
            description: `${c.code.split('\n').length} lines`,
            component: c,
        }));
        const selected = await vscode.window.showQuickPick(items, {
            matchOnDetail: true,
            placeHolder: 'Search components...',
        });
        if (selected) {
            const editor = vscode.window.activeTextEditor;
            if (editor) {
                editor.insertSnippet(new vscode.SnippetString(selected.component.code));
                vscode.window.showInformationMessage(`Inserted ${selected.label}`);
            }
        }
    });
    // Save Selection Command
    const saveCommand = vscode.commands.registerCommand('component-crate.save', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage('No editor active');
            return;
        }
        const selection = editor.selection;
        const selectedCode = editor.document.getText(selection);
        if (!selectedCode || selectedCode.trim() === '') {
            vscode.window.showErrorMessage('No code selected');
            return;
        }
        const name = await vscode.window.showInputBox({
            prompt: 'Component name',
            placeHolder: 'e.g., ButtonPrimary',
            validateInput: (value) => {
                return value && value.length > 0 ? null : 'Name is required';
            }
        });
        if (!name)
            return;
        const tagInput = await vscode.window.showInputBox({
            prompt: 'Tags (comma separated)',
            placeHolder: 'e.g., button, primary, form',
        });
        const tags = tagInput ? tagInput.split(',').map(t => t.trim()).filter(t => t) : [];
        const components = loadCrate();
        components.push({
            name,
            code: selectedCode,
            tags,
            created: Date.now(),
        });
        saveCrate(components);
        vscode.window.showInformationMessage(`Saved "${name}" to crate!`);
    });
    context.subscriptions.push(openCommand, searchCommand, saveCommand);
}
exports.activate = activate;
function getWebviewHtml(components) {
    const componentCards = components.map(c => `
		<div class="component-card" data-name="${c.name}" data-code="${encodeURIComponent(c.code)}">
			<div class="card-header">
				<span class="name">${c.name}</span>
				<span class="lines">${c.code.split('\n').length} lines</span>
			</div>
			<div class="tags">
				${c.tags.map(t => `<span class="tag">${t}</span>`).join('')}
			</div>
			<div class="preview">${escapeHtml(c.code.split('\n').slice(0, 3).join('\n'))}...</div>
			<div class="actions">
				<button class="btn-insert" data-code="${encodeURIComponent(c.code)}">Insert</button>
				<button class="btn-delete" data-name="${c.name}">Delete</button>
			</div>
		</div>
	`).join('');
    return `
<!DOCTYPE html>
<html>
<head>
	<style>
		* {
			box-sizing: border-box;
			margin: 0;
			padding: 0;
		}
		
		body {
			font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
			background: ${THEME.dark};
			color: ${THEME.text};
			padding: 24px;
			min-height: 100vh;
		}
		
		.header {
			display: flex;
			justify-content: space-between;
			align-items: center;
			margin-bottom: 32px;
			padding-bottom: 16px;
			border-bottom: 1px solid ${THEME.border};
		}
		
		.logo {
			font-size: 24px;
			font-weight: bold;
			color: ${THEME.accent};
			letter-spacing: -1px;
		}
		
		.logo span {
			color: ${THEME.text};
		}
		
		.stats {
			color: ${THEME.textDim};
			font-size: 14px;
		}
		
		.search-bar {
			width: 100%;
			padding: 16px 20px;
			background: ${THEME.panel};
			border: 1px solid ${THEME.border};
			color: ${THEME.text};
			font-family: inherit;
			font-size: 16px;
			margin-bottom: 24px;
			outline: none;
			transition: border-color 0.2s;
		}
		
		.search-bar:focus {
			border-color: ${THEME.accent};
		}
		
		.search-bar::placeholder {
			color: ${THEME.textDim};
		}
		
		.grid {
			display: grid;
			grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
			gap: 20px;
		}
		
		.component-card {
			background: ${THEME.panel};
			border: 1px solid ${THEME.border};
			border-radius: 8px;
			padding: 16px;
			transition: transform 0.2s, border-color 0.2s;
		}
		
		.component-card:hover {
			transform: translateY(-2px);
			border-color: ${THEME.accent};
		}
		
		.card-header {
			display: flex;
			justify-content: space-between;
			align-items: center;
			margin-bottom: 12px;
		}
		
		.name {
			font-weight: bold;
			font-size: 16px;
			color: ${THEME.accent};
		}
		
		.lines {
			color: ${THEME.textDim};
			font-size: 12px;
		}
		
		.tags {
			display: flex;
			flex-wrap: wrap;
			gap: 8px;
			margin-bottom: 12px;
		}
		
		.tag {
			background: ${THEME.dark};
			color: ${THEME.textDim};
			padding: 4px 8px;
			border-radius: 4px;
			font-size: 12px;
		}
		
		.preview {
			background: ${THEME.dark};
			padding: 12px;
			border-radius: 4px;
			font-size: 12px;
			color: ${THEME.textDim};
			white-space: pre-wrap;
			overflow: hidden;
			max-height: 60px;
			margin-bottom: 12px;
		}
		
		.actions {
			display: flex;
			gap: 8px;
		}
		
		button {
			flex: 1;
			padding: 10px 16px;
			border: none;
			border-radius: 6px;
			font-family: inherit;
			font-size: 14px;
			font-weight: bold;
			cursor: pointer;
			transition: all 0.2s;
		}
		
		.btn-insert {
			background: ${THEME.accent};
			color: ${THEME.dark};
		}
		
		.btn-insert:hover {
			background: #d4ff4d;
		}
		
		.btn-delete {
			background: transparent;
			color: ${THEME.textDim};
			border: 1px solid ${THEME.border};
		}
		
		.btn-delete:hover {
			color: #ff6b6b;
			border-color: #ff6b6b;
		}
		
		.empty {
			text-align: center;
			padding: 60px 20px;
			color: ${THEME.textDim};
		}
		
		.empty-icon {
			font-size: 48px;
			margin-bottom: 16px;
		}
		
		.shortcut {
			display: inline-block;
			background: ${THEME.panel};
			padding: 4px 8px;
			border-radius: 4px;
			font-size: 12px;
			color: ${THEME.accent};
			margin-left: 8px;
		}
	</style>
</head>
<body>
	<div class="header">
		<div class="logo">COMPONENT<span>CRATE</span></div>
		<div class="stats">${components.length} components <span class="shortcut">⌘⇧C to search</span></div>
	</div>
	
	<input type="text" class="search-bar" placeholder="Search components..." id="search">
	
	${components.length === 0 ? `
		<div class="empty">
			<div class="empty-icon">📦</div>
			<div>Your crate is empty</div>
			<div style="margin-top: 8px; font-size: 14px;">Select code → Cmd+Shift+P → "Component Crate: Save"</div>
		</div>
	` : `
		<div class="grid" id="grid">
			${componentCards}
		</div>
	`}
	
	<script>
		const vscode = acquireVsCodeApi();
		
		// Search functionality
		document.getElementById('search').addEventListener('input', (e) => {
			const query = e.target.value.toLowerCase();
			const cards = document.querySelectorAll('.component-card');
			
			cards.forEach(card => {
				const name = card.dataset.name.toLowerCase();
				const tags = card.querySelector('.tags').textContent.toLowerCase();
				
				if (name.includes(query) || tags.includes(query)) {
					card.style.display = 'block';
				} else {
					card.style.display = 'none';
				}
			});
		});
		
		// Insert button
		document.querySelectorAll('.btn-insert').forEach(btn => {
			btn.addEventListener('click', () => {
				const code = decodeURIComponent(btn.dataset.code);
				vscode.postMessage({ command: 'insert', code });
			});
		});
		
		// Delete button
		document.querySelectorAll('.btn-delete').forEach(btn => {
			btn.addEventListener('click', () => {
				if (confirm('Delete this component?')) {
					vscode.postMessage({ command: 'delete', name: btn.dataset.name });
				}
			});
		});
	</script>
</body>
</html>
	`;
}
function escapeHtml(text) {
    return text
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;');
}
function deactivate() { }
exports.deactivate = deactivate;
