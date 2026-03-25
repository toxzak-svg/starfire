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
exports.GitHubImporter = void 0;
const vscode = __importStar(require("vscode"));
class GitHubImporter {
    constructor() {
        this.token = vscode.workspace.getConfiguration('componentCrate').get('githubToken');
    }
    authHeaders() {
        const headers = {
            Accept: 'application/vnd.github.v3+json',
        };
        if (this.token) {
            headers['Authorization'] = `Bearer ${this.token}`;
        }
        return headers;
    }
    async importFromRepo(store, _context) {
        // Step 1: pick a repository
        const repo = await this.pickRepository();
        if (!repo)
            return;
        const [owner, repoName] = repo.full_name.split('/');
        // Step 2: browse and pick a file
        const selectedFile = await this.browseAndPick(owner, repoName, '');
        if (!selectedFile)
            return;
        // Step 3: download and import
        await this.importFile(selectedFile, owner, repoName, store);
    }
    async pickRepository() {
        const response = await fetch('https://api.github.com/user/repos?per_page=100&sort=updated', { headers: this.authHeaders() });
        if (!response.ok) {
            const err = await response.text();
            vscode.window.showErrorMessage(`GitHub API error: ${response.status} — ${err}`);
            return undefined;
        }
        const repos = (await response.json());
        const items = repos.map(r => ({
            label: r.name,
            description: r.full_name,
        }));
        // Allow entering any public repo
        items.push({
            label: '$(globe) Enter repository URL or name',
            description: 'e.g. facebook/react or https://github.com/facebook/react',
        });
        const picked = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select a repository (or choose "Enter URL" for any public repo)',
            ignoreFocusOut: true,
        });
        if (!picked)
            return undefined;
        // User picked "enter URL" — ask them
        if (picked.label.startsWith('$(globe)')) {
            const input = await vscode.window.showInputBox({
                prompt: 'Enter GitHub repository (owner/repo)',
                validateInput: v => (v.includes('/') ? undefined : 'Use format: owner/repo'),
            });
            if (!input)
                return undefined;
            const [o, n] = input.replace('https://github.com/', '').split('/');
            return { name: n || input, full_name: `${o}/${n || input}`, default_branch: 'main' };
        }
        const fullName = picked.description || picked.label;
        return { name: picked.label, full_name: fullName, default_branch: 'main' };
    }
    async browseFiles(owner, repo, path) {
        const url = path
            ? `https://api.github.com/repos/${owner}/${repo}/contents/${path}`
            : `https://api.github.com/repos/${owner}/${repo}/contents`;
        const response = await fetch(url, { headers: this.authHeaders() });
        if (!response.ok)
            return undefined;
        const files = (await response.json());
        return files.sort((a, b) => {
            if (a.type !== b.type)
                return a.type === 'dir' ? -1 : 1;
            return a.name.localeCompare(b.name);
        });
    }
    async browseAndPick(owner, repo, path) {
        const files = await this.browseFiles(owner, repo, path);
        if (!files) {
            vscode.window.showWarningMessage('Could not browse repository. Check your internet connection or token.');
            return undefined;
        }
        // Build quick pick items
        const items = [];
        if (path) {
            items.push({
                label: '$(arrow-left) ..',
                description: 'Go up one level',
                file: { name: '..', path: path.split('/').slice(0, -1).join('/'), type: 'dir' },
            });
        }
        for (const f of files) {
            items.push({
                label: f.type === 'dir' ? `$(file-directory) ${f.name}` : `$(file-code) ${f.name}`,
                description: f.path,
                file: f,
            });
        }
        const picked = await vscode.window.showQuickPick(items, {
            placeHolder: `${owner}/${repo}${path ? '/' + path : ''} — select a file or folder`,
            ignoreFocusOut: true,
        });
        if (!picked)
            return undefined;
        if (picked.file.name === '..') {
            return this.browseAndPick(owner, repo, picked.file.path);
        }
        if (picked.file.type === 'dir') {
            return this.browseAndPick(owner, repo, picked.file.path);
        }
        return picked.file;
    }
    async importFile(file, owner, repo, store) {
        if (!file.download_url) {
            vscode.window.showWarningMessage('Cannot download this file type');
            return;
        }
        const response = await fetch(file.download_url);
        if (!response.ok) {
            vscode.window.showErrorMessage('Failed to download file');
            return;
        }
        const code = await response.text();
        const name = await vscode.window.showInputBox({
            prompt: 'Component name',
            value: file.name.replace(/\.[^.]+$/, ''),
            validateInput: v => (v.trim() ? undefined : 'Name is required'),
        });
        if (!name)
            return;
        const tagsRaw = await vscode.window.showInputBox({
            prompt: 'Tags (comma-separated, optional)',
        });
        const ext = file.name.split('.').pop() || '';
        const langMap = {
            ts: 'typescript', tsx: 'typescript', js: 'javascript',
            jsx: 'javascript', py: 'python', go: 'go', rs: 'rust',
            svelte: 'svelte', vue: 'vue', css: 'css', scss: 'scss',
        };
        store.add({
            name: name.trim(),
            code,
            language: langMap[ext] || ext,
            tags: tagsRaw ? tagsRaw.split(',').map(t => t.trim()).filter(Boolean) : [],
            source: 'github',
            githubPath: file.path,
            synced: false,
        });
        vscode.window.showInformationMessage(`Imported "${name}" from ${owner}/${repo}! (${store.count()}/10)`);
    }
}
exports.GitHubImporter = GitHubImporter;
