import * as vscode from 'vscode';

export interface SavedComponent {
	id: string;
	name: string;
	code: string;
	language: string;
	tags: string[];
	createdAt: string;
	source?: 'local' | 'github';
	githubPath?: string;
	synced: boolean;
}

export interface StoreData {
	components: SavedComponent[];
	version: number;
}

export class ComponentStore {
	private data: StoreData;
	private _onDidChange = new vscode.EventEmitter<void>();

	constructor(private globalState: vscode.Memento) {
		this.data = globalState.get<StoreData>('component-crate-data', {
			components: [],
			version: 1,
		});
	}

	get onDidChange(): vscode.Event<void> {
		return this._onDidChange.event;
	}

	private save() {
		this.globalState.update('component-crate-data', this.data);
		this._onDidChange.fire();
	}

	add(component: Omit<SavedComponent, 'id' | 'createdAt'>): SavedComponent {
		const saved: SavedComponent = {
			...component,
			id: this.generateId(),
			createdAt: new Date().toISOString(),
		};
		this.data.components.push(saved);
		this.save();
		return saved;
	}

	getAll(): SavedComponent[] {
		return [...this.data.components];
	}

	getById(id: string): SavedComponent | undefined {
		return this.data.components.find(c => c.id === id);
	}

	search(query: string): SavedComponent[] {
		const q = query.toLowerCase();
		return this.data.components.filter(
			c =>
				c.name.toLowerCase().includes(q) ||
				c.code.toLowerCase().includes(q) ||
				c.tags.some(t => t.toLowerCase().includes(q))
		);
	}

	delete(id: string): boolean {
		const idx = this.data.components.findIndex(c => c.id === id);
		if (idx === -1) return false;
		this.data.components.splice(idx, 1);
		this.save();
		return true;
	}

	update(id: string, patch: Partial<SavedComponent>): boolean {
		const comp = this.getById(id);
		if (!comp) return false;
		Object.assign(comp, patch);
		this.save();
		return true;
	}

	count(): number {
		return this.data.components.filter(c => c.source !== 'github').length;
	}

	private generateId(): string {
		return `comp_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
	}
}
