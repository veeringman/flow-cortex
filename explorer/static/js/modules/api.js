/**
 * API Module - Handles all HTTP communication with FlowCortex L1 node
 * 
 * Configuration:
 * - Set window.API_BASE to override default
 * - Or set via data-api-base attribute on <body>
 * - Defaults to http://127.0.0.1:3000
 */

// Determine API base URL (configurable)
let API_BASE = window.API_BASE || 
               document.body.getAttribute('data-api-base') || 
               'http://127.0.0.1:3000';

// Allow dynamic configuration
window.setApiBase = (url) => {
    API_BASE = url;
    console.log(`📡 API Base URL changed to: ${API_BASE}`);
};

/**
 * Generic API call wrapper with error handling
 */
export async function apiCall(endpoint, options = {}) {
    try {
        // Remove trailing slash from API_BASE and ensure endpoint starts with /
        const base = API_BASE.replace(/\/+$/, '');
        const path = endpoint.startsWith('/') ? endpoint : `/${endpoint}`;
        const url = `${base}${path}`;
        
        const response = await fetch(url, {
            headers: { 'Content-Type': 'application/json' },
            ...options
        });
        return await response.json();
    } catch (error) {
        console.error(`API Error [${endpoint}]:`, error);
        return { error: error.message };
    }
}

/**
 * Balance API
 */
export const BalanceAPI = {
    async getBalance(account, token) {
        return apiCall(`/balance/${account}/${token}`);
    }
};

/**
 * Block API
 */
export const BlockAPI = {
    async listBlocks() {
        return apiCall('/blocks');
    },
    
    async createBlock() {
        return apiCall('/block', { method: 'POST' });
    }
};

/**
 * Transaction API
 */
export const TransactionAPI = {
    async getPool() {
        return apiCall('/pool');
    },
    
    async submitTransaction(signedTx) {
        return apiCall('/tx', {
            method: 'POST',
            body: JSON.stringify(signedTx)
        });
    },
    
    async getSnapshot() {
        return apiCall('/snapshot');
    }
};

/**
 * Capsule API
 */
export const CapsuleAPI = {
    async list() {
        return apiCall('/capsule');
    },
    
    async upload(id, code) {
        return apiCall('/capsule', {
            method: 'POST',
            body: JSON.stringify({ id, code })
        });
    },
    
    async invoke(id, input) {
        return apiCall(`/capsule/${encodeURIComponent(id)}/invoke`, {
            method: 'POST',
            body: JSON.stringify({ input })
        });
    }
};

/**
 * Anchor API
 */
export const AnchorAPI = {
    async list() {
        return apiCall('/anchors');
    },
    
    async get(id) {
        return apiCall(`/anchor/${encodeURIComponent(id)}`);
    }
};

/**
 * Network Status
 */
export const NetworkAPI = {
    async checkStatus() {
        try {
            await fetch(`${API_BASE}/blocks`);
            return { online: true };
        } catch {
            return { online: false };
        }
    }
};
