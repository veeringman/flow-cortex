"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.FlowCortexL1Client = void 0;
const axios_1 = __importDefault(require("axios"));
/**
 * FlowCortex L1 Client for REST API
 * Provides methods to interact with the FlowCortex L1 node
 */
class FlowCortexL1Client {
    constructor(nodeUrl = 'http://127.0.0.1:3000') {
        this.baseURL = nodeUrl;
        this.client = axios_1.default.create({
            baseURL: this.baseURL,
            timeout: 10000,
        });
    }
    /**
     * Create a new account
     */
    async createAccount(account) {
        const response = await this.client.post('/account', { account });
        return { success: response.status === 201 };
    }
    /**
     * Get account balance
     */
    async getBalance(account, token) {
        const response = await this.client.get(`/balance/${account}/${String(token).toLowerCase()}`);
        return response.data;
    }
    /**
     * Mint tokens to an account
     */
    async mint(params) {
        await this.client.post('/mint', {
            ...params,
            token: String(params.token).toLowerCase(),
        });
    }
    /**
     * Transfer tokens between accounts
     */
    async transfer(params) {
        await this.client.post('/transfer', {
            ...params,
            token: String(params.token).toLowerCase(),
        });
    }
    /**
     * Submit a signed transaction
     */
    async submitTx(params) {
        // Convert to base64 if needed
        const payload = {
            ...params,
            pubkey: typeof params.pubkey === 'string' ? params.pubkey : Buffer.from(params.pubkey).toString('base64'),
            signature: typeof params.signature === 'string' ? params.signature : Buffer.from(params.signature).toString('base64'),
        };
        await this.client.post('/tx', payload);
    }
    /**
     * Get pending transaction pool
     */
    async getPool() {
        const response = await this.client.get('/pool');
        return response.data;
    }
    /**
     * Create a new block
     */
    async createBlock() {
        const response = await this.client.post('/block');
        return response.data;
    }
    /**
     * List all blocks
     */
    async listBlocks() {
        const response = await this.client.get('/blocks');
        return response.data;
    }
    /**
     * Get current state snapshot
     */
    async getSnapshot() {
        const response = await this.client.get('/snapshot');
        return response.data;
    }
    /**
     * List all anchors
     */
    async listAnchors() {
        const response = await this.client.get('/anchors');
        return response.data;
    }
    /**
     * Get a specific anchor by ID
     */
    async getAnchor(id) {
        const response = await this.client.get(`/anchor/${id}`);
        return response.data;
    }
    /**
     * Upload a capsule (wasm/bytecode)
     */
    async uploadCapsule(id, codeBase64) {
        const response = await this.client.post('/capsule', {
            id,
            code: codeBase64,
        });
        return response.data;
    }
    /**
     * List all capsules
     */
    async listCapsules() {
        const response = await this.client.get('/capsule');
        return response.data;
    }
    /**
     * Invoke a capsule with input data
     */
    async invokeCapsule(id, inputBase64) {
        const response = await this.client.post(`/capsule/${id}/invoke`, {
            input: inputBase64,
        });
        return response.data;
    }
}
exports.FlowCortexL1Client = FlowCortexL1Client;
