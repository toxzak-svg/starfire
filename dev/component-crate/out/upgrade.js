"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.UpgradeManager = void 0;
const FREE_LIMIT = 10;
class UpgradeManager {
    constructor(store) {
        this.store = store;
    }
    checkLimit() {
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
    getRemainingSlots() {
        return Math.max(0, FREE_LIMIT - this.store.count());
    }
}
exports.UpgradeManager = UpgradeManager;
