import * as vscode from 'vscode';
import { ComponentStore } from './storage/store';

const FREE_LIMIT = 10;

export interface LimitResult {
	allowed: boolean;
	message?: string;
	upgradeUrl?: string;
}

export class UpgradeManager {
	constructor(private store: ComponentStore) {}

	checkLimit(): LimitResult {
		const count = this.store.count();
		if (count < FREE_LIMIT) {
			return { allowed: true };
		}
		return {
			allowed: false,
			message: `Free limit reached (${count}/${FREE_LIMIT} components). Upgrade to store unlimited components.`,
			upgradeUrl: 'https://componentcrate.com/pricing',
		};
	}

	getRemainingSlots(): number {
		return Math.max(0, FREE_LIMIT - this.store.count());
	}
}
