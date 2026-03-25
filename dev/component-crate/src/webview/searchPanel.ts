import * as vscode from 'vscode';
import * as path from 'path';
import { ComponentStore, SavedComponent } from '../storage/store';

export class SearchPanel {
	public static currentPanel: SearchPanel | undefined;
	public static readonly viewType = 'component-crate.search';

	private readonly panel: vscode.WebviewPanel;
	private readonly extensionUri: vscode.Uri;
	private store: ComponentStore;

	private constructor(panel: vscode.WebviewPanel, uri: vscode.Uri, store: ComponentStore) {
		this.panel = panel;
		this.extensionUri = uri;
		this.store = store;

		this.panel.webview.html = this.getHtml('Loading...');

		// Live search as user types
		this.panel.webview.onDidReceiveMessage(async (msg) => {
			switch (msg.type) {
				case 'search':
					this.handleSearch(msg.query);
					break;
				case 'insert':
					this.handleInsert(msg.id);
					break;
				case 'delete':
					this.handleDelete(msg.id);
					break;
				case 'preview':
					this.handlePreview(msg.code, msg.lang);
					break;
			}
		});

		this.panel.onDidDispose(() => {
			SearchPanel.currentPanel = undefined;
		});

		this.render();
	}

	static createOrShow(uri: vscode.Uri, store: ComponentStore) {
		if (SearchPanel.currentPanel) {
			SearchPanel.currentPanel.panel.reveal(vscode.ViewColumn.One, true);
			return;
		}

		const panel = vscode.window.createWebviewPanel(
			SearchPanel.viewType,
			'Component Crate',
			{ viewColumn: vscode.ViewColumn.One, preserveFocus: false },
			{ enableScripts: true, retainContextWhenHidden: true }
		);

		SearchPanel.currentPanel = new SearchPanel(panel, uri, store);
	}

	private handleSearch(query: string) {
		const components = query.trim()
			? this.store.search(query)
			: this.store.getAll();

		this.panel.webview.postMessage({
			type: 'results',
			components: components.slice(0, 20),
			count: components.length,
		});
	}

	private handleInsert(id: string) {
		const comp = this.store.getById(id);
		if (!comp) return;

		const editor = vscode.window.activeTextEditor;
		if (editor) {
			editor.edit(editBuilder => {
				editBuilder.insert(editor.selection.active, comp.code);
			});
		}
		this.panel.webview.postMessage({ type: 'inserted', id });
	}

	private handleDelete(id: string) {
		this.store.delete(id);
		this.panel.webview.postMessage({ type: 'deleted', id });
	}

	private handlePreview(code: string, lang: string) {
		// Show in a read-only editor tab
		const doc = vscode.workspace.openTextDocument({
			content: code,
			language: lang,
		}).then(d => vscode.window.showTextDocument(d, vscode.ViewColumn.Two));
	}

	private render() {
		this.panel.webview.html = this.getHtml('');
		// Load initial data
		setTimeout(() => this.handleSearch(''), 100);
	}

	private getHtml(_searchQuery: string): string {
		return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Component Crate</title>
<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  :root {
    --bg: #1e1e1e;
    --surface: #252526;
    --border: #3c3c3c;
    --text: #cccccc;
    --text-muted: #808080;
    --accent: #0e639c;
    --accent-hover: #1177bb;
    --danger: #f14c4c;
    --success: #89d185;
  }
  body { font-family: system-ui, sans-serif; background: var(--bg); color: var(--text); min-height: 100vh; }
  .header { padding: 16px; border-bottom: 1px solid var(--border); display: flex; align-items: center; gap: 12px; }
  .logo { font-size: 18px; font-weight: 700; color: #fff; }
  .logo span { color: var(--accent); }
  .search-box { flex: 1; }
  .search-box input {
    width: 100%; padding: 8px 12px; background: var(--surface);
    border: 1px solid var(--border); border-radius: 6px;
    color: var(--text); font-size: 14px; outline: none;
  }
  .search-box input:focus { border-color: var(--accent); }
  .meta { padding: 8px 16px; font-size: 12px; color: var(--text-muted); border-bottom: 1px solid var(--border); }
  .results { padding: 8px; }
  .card {
    padding: 12px; margin-bottom: 8px; background: var(--surface);
    border: 1px solid var(--border); border-radius: 6px;
    cursor: pointer; transition: border-color 0.15s;
  }
  .card:hover { border-color: var(--accent); }
  .card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px; }
  .card-name { font-weight: 600; color: #fff; font-size: 14px; }
  .card-lang { font-size: 11px; background: var(--border); padding: 2px 6px; border-radius: 4px; color: var(--text-muted); }
  .card-code {
    font-family: 'Fira Code', 'Cascadia Code', monospace;
    font-size: 11px; color: var(--text-muted); white-space: pre-wrap;
    max-height: 60px; overflow: hidden; position: relative;
  }
  .card-code::after { content: ''; position: absolute; bottom: 0; left: 0; right: 0; height: 20px; background: linear-gradient(transparent, var(--surface)); }
  .card-tags { margin-top: 6px; display: flex; gap: 4px; flex-wrap: wrap; }
  .tag { font-size: 11px; background: var(--accent); color: #fff; padding: 1px 6px; border-radius: 4px; }
  .card-actions { margin-top: 8px; display: flex; gap: 8px; }
  .btn {
    font-size: 12px; padding: 4px 10px; border-radius: 4px;
    border: none; cursor: pointer; transition: background 0.15s;
  }
  .btn-insert { background: var(--accent); color: #fff; }
  .btn-insert:hover { background: var(--accent-hover); }
  .btn-preview { background: var(--border); color: var(--text); }
  .btn-preview:hover { background: #4a4a4a; }
  .btn-delete { background: transparent; color: var(--danger); border: 1px solid var(--danger); }
  .btn-delete:hover { background: var(--danger); color: #fff; }
  .btn-github { background: transparent; color: var(--success); border: 1px solid var(--success); font-size: 11px; }
  .empty { text-align: center; padding: 40px; color: var(--text-muted); }
  .empty-icon { font-size: 40px; margin-bottom: 12px; }
  .count-badge { background: var(--accent); color: #fff; padding: 2px 8px; border-radius: 10px; font-size: 12px; }
  .toast { position: fixed; bottom: 16px; left: 50%; transform: translateX(-50%); background: var(--success); color: #000; padding: 8px 16px; border-radius: 6px; font-size: 13px; opacity: 0; transition: opacity 0.2s; pointer-events: none; }
  .toast.show { opacity: 1; }
  .free-banner { margin: 8px; padding: 8px 12px; background: #1a3a1a; border: 1px solid var(--success); border-radius: 6px; font-size: 12px; color: var(--success); display: none; }
  .free-banner.visible { display: block; }
</style>
</head>
<body>
  <div class="header">
    <div class="logo">Component<span>Crate</span></div>
    <div class="search-box">
      <input type="text" id="searchInput" placeholder="Search components..." autofocus>
    </div>
  </div>

  <div class="free-banner" id="freeBanner">
    You're using the free plan (10 components). <a href="#" onclick="openUpgrade()" style="color:inherit;text-decoration:underline;">Upgrade for unlimited →</a>
  </div>

  <div class="meta" id="meta">Loading...</div>
  <div class="results" id="results"></div>
  <div class="toast" id="toast"></div>

  <script>
    const vscode = acquireVsCodeApi();
    const searchInput = document.getElementById('searchInput');
    const resultsDiv = document.getElementById('results');
    const metaDiv = document.getElementById('meta');
    const toast = document.getElementById('toast');
    const freeBanner = document.getElementById('freeBanner');

    let components = [];

    searchInput.addEventListener('input', () => {
      vscode.postMessage({ type: 'search', query: searchInput.value });
    });

    vscode.postMessage({ type: 'search', query: '' });

    window.addEventListener('message', e => {
      const msg = e.data;
      if (msg.type === 'results') {
        components = msg.components;
        render(components, msg.count);
      }
      if (msg.type === 'inserted') {
        showToast('Component inserted!');
      }
      if (msg.type === 'deleted') {
        showToast('Component deleted');
        vscode.postMessage({ type: 'search', query: searchInput.value });
      }
    });

    function render(comps, total) {
      const count = comps.length;
      const totalAll = total;
      metaDiv.textContent = count === totalAll
        ? \`\${totalAll} component\${totalAll !== 1 ? 's' : ''} saved\`
        : \`\${count} of \${totalAll} results\`;

      freeBanner.className = totalAll >= 10 ? 'free-banner visible' : 'free-banner';

      if (!comps.length) {
        resultsDiv.innerHTML = \`<div class="empty">
          <div class="empty-icon">📦</div>
          <div>No components yet.</div>
          <div style="margin-top:8px;font-size:12px;">Select code in editor → right-click → Save to Crate</div>
        </div>\`;
        return;
      }

      resultsDiv.innerHTML = comps.map(c => \`
        <div class="card" id="card-\${c.id}">
          <div class="card-header">
            <span class="card-name">\${esc(c.name)}</span>
            <span class="card-lang">\${esc(c.language)}</span>
          </div>
          <div class="card-code">\${esc(c.code.slice(0, 200))}</div>
          \${c.tags.length ? \`<div class="card-tags">\${c.tags.map(t => \`<span class="tag">\${esc(t)}</span>\`).join('')}</div>\` : ''}
          <div class="card-actions">
            <button class="btn btn-insert" onclick="insert('\${c.id}')">Insert</button>
            <button class="btn btn-preview" onclick="preview('\${c.id}')">Preview</button>
            <button class="btn btn-delete" onclick="deleteComp('\${c.id}')">Delete</button>
            \${c.source === 'github' ? '<span class="btn btn-github">from GitHub</span>' : ''}
          </div>
        </div>
      \`).join('');
    }

    function insert(id) { vscode.postMessage({ type: 'insert', id }); }
    function preview(id) {
      const c = components.find(x => x.id === id);
      if (c) vscode.postMessage({ type: 'preview', code: c.code, lang: c.language });
    }
    function deleteComp(id) {
      if (confirm('Delete this component?')) vscode.postMessage({ type: 'delete', id });
    }
    function openUpgrade() { vscode.postMessage({ type: 'upgrade' }); }
    function esc(s) {
      return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
    }
    function showToast(msg) {
      toast.textContent = msg;
      toast.classList.add('show');
      setTimeout(() => toast.classList.remove('show'), 2000);
    }
  </script>
</body>
</html>`;
	}
}
