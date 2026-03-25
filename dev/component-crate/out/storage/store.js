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
exports.ComponentStore = void 0;
const vscode = __importStar(require("vscode"));
class ComponentStore {
    constructor(globalState) {
        this.globalState = globalState;
        this._onDidChange = new vscode.EventEmitter();
        this.data = globalState.get('component-crate-data', {
            components: [],
            version: 1,
        });
    }
    get onDidChange() {
        return this._onDidChange.event;
    }
    save() {
        this.globalState.update('component-crate-data', this.data);
        this._onDidChange.fire();
    }
    add(component) {
        const saved = {
            ...component,
            id: this.generateId(),
            createdAt: new Date().toISOString(),
        };
        this.data.components.push(saved);
        this.save();
        return saved;
    }
    getAll() {
        return [...this.data.components];
    }
    getById(id) {
        return this.data.components.find(c => c.id === id);
    }
    search(query) {
        const q = query.toLowerCase();
        return this.data.components.filter(c => c.name.toLowerCase().includes(q) ||
            c.code.toLowerCase().includes(q) ||
            c.tags.some(t => t.toLowerCase().includes(q)));
    }
    delete(id) {
        const idx = this.data.components.findIndex(c => c.id === id);
        if (idx === -1)
            return false;
        this.data.components.splice(idx, 1);
        this.save();
        return true;
    }
    update(id, patch) {
        const comp = this.getById(id);
        if (!comp)
            return false;
        Object.assign(comp, patch);
        this.save();
        return true;
    }
    count() {
        return this.data.components.filter(c => c.source !== 'github').length;
    }
    generateId() {
        return `comp_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
    }
}
exports.ComponentStore = ComponentStore;
