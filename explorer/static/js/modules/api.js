/**
 * API Module - Handles all HTTP communication with FlowCortex L1 node
 * 
 * Configuration:
 * - Uses same-origin /api proxy by default (avoids cross-origin cert issues)
 * - Set window.API_BASE to override
 * - Or set via data-api-base attribute on <body>
 */

// Determine API base URL (configurable)
// Default: same-origin /api proxy served by Explorer
let API_BASE = window.API_BASE || 
               document.body.getAttribute('data-api-base') || 
               '/api';

const apiBaseEl = document.getElementById('api-base-url');
if (apiBaseEl) {
    apiBaseEl.textContent = API_BASE;
}

// Allow dynamic configuration
window.setApiBase = (url) => {
    API_BASE = url;
    const el = document.getElementById('api-base-url');
    if (el) {
        el.textContent = API_BASE;
    }
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
 * Token API
 */
export const TokenAPI = {
    async createToken(payload) {
        return apiCall('/token/create', {
            method: 'POST',
            body: JSON.stringify(payload)
        });
    },

    async listTokens() {
        return apiCall('/tokens');
    },

    async getToken(symbol) {
        return apiCall(`/token/${encodeURIComponent(symbol)}`);
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
