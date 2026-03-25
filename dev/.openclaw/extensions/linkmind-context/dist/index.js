const PLUGIN_ID = "linkmind-context";
const DEFAULT_API_URL = "https://api.linkmind.dev/v1";
const DEFAULT_COMPRESSION_THRESHOLD = 1000;
class LinkMindClient {
    config;
    constructor(config) {
        this.config = {
            apiUrl: config.apiUrl || DEFAULT_API_URL,
            apiKey: config.apiKey || "",
            compressionThreshold: config.compressionThreshold || DEFAULT_COMPRESSION_THRESHOLD,
            debug: config.debug || false,
        };
    }
    getConfig() {
        return this.config;
    }
    async compress(params) {
        if (this.config.debug) {
            console.log("[LinkMind] Calling compress API with", params.messages.length, "messages");
        }
        const response = await fetch(`${this.config.apiUrl}/openclaw/compress`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                ...(this.config.apiKey ? { Authorization: `Bearer ${this.config.apiKey}` } : {}),
            },
            body: JSON.stringify({
                sessionId: params.sessionId,
                messages: params.messages,
                tokenBudget: params.tokenBudget,
                currentTokenCount: params.currentTokenCount,
            }),
        });
        if (!response.ok) {
            throw new Error(`[LinkMind] compress API error: ${response.status} ${response.statusText}`);
        }
        const data = (await response.json());
        if (data.status !== "success") {
            throw new Error(`[LinkMind] compress API returned error: ${data.error ?? "unknown error"}`);
        }
        return {
            messages: data.messages,
            tokensAfter: data.tokensAfter ?? params.currentTokenCount ?? 0,
        };
    }
}
class LinkMindContextEngine {
    info = {
        id: PLUGIN_ID,
        name: "LinkMind Intelligent Context Compression Engine",
        version: "1.0.0",
        ownsCompaction: true,
    };
    client;
    sessionId;
    sessionFile;
    pendingMessages;
    constructor(config = {}) {
        this.client = new LinkMindClient(config);
    }
    async bootstrap(params) {
        this.sessionId = params.sessionId;
        this.sessionFile = params.sessionFile;
        if (this.client.getConfig().debug) {
            console.log(`[LinkMindPlugin] Initialization complete, session ID: ${params.sessionId}, session file: ${params.sessionFile}`);
        }
        return {
            bootstrapped: true,
            importedMessages: 0,
        };
    }
    async ingest(params) {
        if (params.isHeartbeat) {
            return { ingested: false };
        }
        if (this.client.getConfig().debug) {
            console.log(`[LinkMindPlugin] Message ingested (no storage), role: ${params.message.role}, chars: ${this.contentChars(params.message.content)}`);
        }
        return { ingested: true };
    }
    async afterTurn(params) {
        if (params.isHeartbeat) {
            return;
        }
        const config = this.client.getConfig();
        const totalChars = params.messages.reduce((sum, message) => sum + this.contentChars(message.content), 0);
        if (config.debug) {
            console.log(`[LinkMindPlugin] afterTurn: messages=${params.messages.length}, chars=${totalChars}, budget=${params.tokenBudget}`);
        }
        if (totalChars <= config.compressionThreshold) {
            return;
        }
        console.log(`[LinkMindPlugin] Threshold exceeded (chars=${totalChars} > threshold=${config.compressionThreshold}), triggering compact...`);
        this.pendingMessages = params.messages;
        try {
            await this.compact({
                sessionId: params.sessionId,
                sessionFile: params.sessionFile,
                currentTokenCount: Math.ceil(totalChars / 4),
                ...(params.tokenBudget !== undefined && { tokenBudget: params.tokenBudget }),
                ...(params.runtimeContext !== undefined && { runtimeContext: params.runtimeContext }),
            });
        }
        finally {
            this.pendingMessages = undefined;
        }
    }
    async assemble(params) {
        const estimatedTokens = params.messages.reduce((sum, message) => sum + Math.ceil(this.contentChars(message.content) / 4), 0);
        if (this.client.getConfig().debug) {
            console.log(`[LinkMindPlugin] Context assembly complete, total ${params.messages.length} messages, estimated tokens: ${estimatedTokens}`);
        }
        return {
            messages: params.messages,
            estimatedTokens,
        };
    }
    async compact(params) {
        const config = this.client.getConfig();
        const tokensBefore = params.currentTokenCount || 0;
        const messages = this.pendingMessages;
        if (config.debug) {
            console.log(`[LinkMindPlugin] Starting compression, current tokens: ${tokensBefore}, budget: ${params.tokenBudget}`);
        }
        if (!messages || messages.length === 0) {
            if (config.debug) {
                console.log("[LinkMindPlugin] No pending messages, skipping compression");
            }
            return { ok: true, compacted: false };
        }
        try {
            const result = await this.client.compress({
                sessionId: params.sessionId,
                messages,
                ...(params.tokenBudget !== undefined && { tokenBudget: params.tokenBudget }),
                currentTokenCount: tokensBefore,
            });
            if (config.debug) {
                console.log(`[LinkMindPlugin] Compression complete, tokens before: ${tokensBefore}, after: ${result.tokensAfter}`);
            }
            return {
                ok: true,
                compacted: true,
                result: {
                    tokensBefore,
                    tokensAfter: result.tokensAfter,
                    details: {
                        compressedMessages: result.messages.length,
                        compressionRatio: tokensBefore > 0 ? result.tokensAfter / tokensBefore : 1,
                    },
                },
            };
        }
        catch (error) {
            console.error("[LinkMindPlugin] Compression API call failed:", error);
            return {
                ok: false,
                compacted: false,
                reason: String(error),
            };
        }
    }
    async dispose() {
        if (this.client.getConfig().debug) {
            console.log("[LinkMindPlugin] Resources released");
        }
        this.sessionId = undefined;
        this.sessionFile = undefined;
        this.pendingMessages = undefined;
    }
    contentChars(content) {
        if (typeof content === "string") {
            return content.length;
        }
        if (Array.isArray(content)) {
            return content.reduce((sum, block) => {
                return sum + (typeof block.text === "string" ? block.text.length : 0);
            }, 0);
        }
        return 0;
    }
}
export function createPlugin(config = {}) {
    return new LinkMindContextEngine(config);
}
export default {
    id: PLUGIN_ID,
    name: "LinkMind Context Engine",
    description: "A context engine that compresses chat history using the LinkMind API.",
    register(api) {
        api.logger.info("[LinkMindPlugin] Plugin initialized, preparing Context Engine...");
        const config = api.pluginConfig ?? {};
        if (typeof api.registerContextEngine === "function") {
            api.registerContextEngine(PLUGIN_ID, () => new LinkMindContextEngine(config));
            api.logger.info("[LinkMindPlugin] Context Engine factory registered via registerContextEngine(id, factory)");
            return;
        }
        api.logger.error("[LinkMindPlugin] Error: registerContextEngine not found on API.");
        api.logger.error("[LinkMindPlugin] Available API keys:", Object.keys(api));
    },
};
//# sourceMappingURL=index.js.map