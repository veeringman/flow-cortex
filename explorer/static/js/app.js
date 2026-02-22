/**
 * FlowCortex Explorer - Main Application
 * Modular architecture with ES6 modules
 */

import { BlockAPI, TransactionAPI, BalanceAPI, CapsuleAPI, AnchorAPI, NetworkAPI } from './modules/api.js';
import { connectWallet, disconnectWallet, signTransaction, isWalletConnected, createTransferTx, createAnchorTx } from './modules/wallet.js';
import { initBlockChart, initTxTypeChart, updateBlockChart, updateTxTypeChart, updateChartTheme } from './modules/charts.js';
import * as UI from './modules/ui.js';

/**
 * Application State
 */
const state = {
    stats: {
        blocks: 0,
        txs: 0,
        pending: 0,
        capsules: 0,
        prevBlocks: 0,
        prevTxs: 0
    },
    blocks: [],
    pool: [],
    capsules: [],
    anchors: []
};

/**
 * Initialize the application
 */
async function init() {
    console.log('🚀 Initializing FlowCortex Explorer...');
    
    // Load saved API base from localStorage
    const savedApiBase = localStorage.getItem('apiBase');
    if (savedApiBase) {
        window.setApiBase(savedApiBase);
        const endpoint = document.getElementById('api-endpoint');
        if (endpoint) {
            endpoint.textContent = savedApiBase;
        }
        const apiInput = document.getElementById('apiBaseInput');
        if (apiInput) {
            apiInput.value = savedApiBase;
        }
    }
    
    // Initialize UI
    UI.initTheme();
    UI.initUI();
    
    // Initialize charts
    initBlockChart('blockChart');
    initTxTypeChart('txTypeChart');
    
    // Setup global functions for HTML onclick handlers
    window.switchTab = UI.switchTab;
    window.openSidebar = UI.openSidebar;
    window.closeSidebar = UI.closeSidebar;
    window.toggleTheme = () => {
        UI.toggleTheme();
        updateChartTheme();
    };
    window.showModal = UI.showModal;
    window.closeModal = UI.closeModal;
    window.refreshAll = refreshAll;
    window.handleGlobalSearch = handleGlobalSearch;
    window.updateApiBase = updateApiBase;
    
    // Setup page-specific functions
    window.queryBalance = queryBalance;
    window.queryBlocks = queryBlocks;
    window.createBlock = createBlock;
    window.queryPool = queryPool;
    window.querySnapshot = querySnapshot;
    window.uploadCapsule = uploadCapsule;
    window.listCapsules = listCapsules;
    window.invokeCapsule = invokeCapsule;
    window.submitAnchor = submitAnchor;
    window.listAnchors = listAnchors;
    window.getAnchor = getAnchor;
    window.connectWallet = handleConnectWallet;
    window.signAndSubmit = signAndSubmit;
    window.beautifyTxJson = beautifyTxJson;
    window.loadExampleTx = loadExampleTx;
    window.copyToClipboard = UI.copyToClipboard;
    
    // Initial data load
    await loadDashboard();
    
    // Setup periodic updates
    setInterval(updateNetworkStatus, 5000);
    setInterval(loadDashboard, 15000);
    
    // Listen for theme changes
    window.addEventListener('themeChanged', () => {
        updateChartTheme();
    });
    
    console.log('✅ FlowCortex Explorer ready!');
}

/**
 * Update network status indicator
 */
async function updateNetworkStatus() {
    const status = await NetworkAPI.checkStatus();
    const statusEl = document.getElementById('network-status');
    
    if (statusEl) {
        if (status.online) {
            statusEl.innerHTML = `
                <div class="w-2 h-2 rounded-full bg-green-500 animate-pulse"></div>
                <span class="text-green-600 dark:text-green-400">Connected</span>
            `;
        } else {
            statusEl.innerHTML = `
                <div class="w-2 h-2 rounded-full bg-red-500"></div>
                <span class="text-red-600 dark:text-red-400">Offline</span>
            `;
        }
    }
}

/**
 * Update API Base URL
 */
async function updateApiBase() {
    const input = document.getElementById('apiBaseInput');
    const newUrl = input.value.trim();
    
    if (!newUrl) {
        UI.showToast('error', 'Please enter a valid API URL');
        return;
    }
    
    // Update the API base
    window.setApiBase(newUrl);
    
    // Update the display
    const endpoint = document.getElementById('api-endpoint');
    if (endpoint) {
        endpoint.textContent = newUrl;
    }
    
    // Save to localStorage
    localStorage.setItem('apiBase', newUrl);
    
    // Test connection
    updateNetworkStatus();
    
    // Close modal and show success
    UI.closeModal('apiConfigModal');
    UI.showToast('success', `API configured: ${newUrl}`);
    
    console.log(`✅ API Base updated to: ${newUrl}`);
}

/**
 * Load dashboard data
 */
async function loadDashboard() {
    try {
        const [blocks, pool, capsules, snapshot] = await Promise.all([
            BlockAPI.listBlocks(),
            TransactionAPI.getPool(),
            CapsuleAPI.list(),
            TransactionAPI.getSnapshot()
        ]);
        
        // Update blocks
        if (!blocks.error && Array.isArray(blocks)) {
            state.prevBlocks = state.stats.blocks;
            state.stats.blocks = blocks.length;
            state.blocks = blocks;
            
            // Calculate total transactions
            state.prevTxs = state.stats.txs;
            state.stats.txs = blocks.reduce((sum, b) => {
                try {
                    const txs = Array.isArray(b.transactions) ? b.transactions : JSON.parse(b.txs_json || '[]');
                    return sum + txs.length;
                } catch {
                    return sum;
                }
            }, 0);
            
            updateDashboardStats();
            updateRecentBlocks(blocks.slice(-5).reverse());
            updateBlockChart(blocks);
            updateTxTypeChart(blocks);
        }
        
        // Update pool
        if (!pool.error) {
            try {
                const pending = JSON.parse(pool.pending_json || '[]');
                state.stats.pending = pending.length;
                state.pool = pending;
                updateDashboardStats();
            } catch (error) {
                console.error('Error parsing pool:', error);
            }
        }
        
        // Update capsules
        if (!capsules.error && capsules.capsules) {
            state.stats.capsules = capsules.capsules.length;
            state.capsules = capsules.capsules;
            updateDashboardStats();
        }
        
        // Update snapshot
        if (!snapshot.error && snapshot.root) {
            const rootEl = document.getElementById('stat-root');
            if (rootEl) {
                rootEl.textContent = UI.truncateHash(snapshot.root, 8, 8);
            }
        }
        
    } catch (error) {
        console.error('Dashboard load error:', error);
    }
}

/**
 * Update dashboard statistics
 */
function updateDashboardStats() {
    const statBlocks = document.getElementById('stat-blocks');
    const statTxs = document.getElementById('stat-txs');
    const statPending = document.getElementById('stat-pending');
    const statCapsules = document.getElementById('stat-capsules');
    
    if (statBlocks) statBlocks.textContent = state.stats.blocks;
    if (statTxs) statTxs.textContent = state.stats.txs;
    if (statPending) statPending.textContent = state.stats.pending;
    if (statCapsules) statCapsules.textContent = state.stats.capsules;
    
    // Growth indicators (optional elements)
    const blockGrowth = state.stats.blocks - state.prevBlocks;
    const txGrowth = state.stats.txs - state.prevTxs;
    
    const blocksGrowthEl = document.getElementById('blocks-growth');
    const txsGrowthEl = document.getElementById('txs-growth');
    
    if (blocksGrowthEl) blocksGrowthEl.textContent = `+${blockGrowth}`;
    if (txsGrowthEl) txsGrowthEl.textContent = `+${txGrowth}`;
}

/**
 * Update recent blocks display
 */
function updateRecentBlocks(blocks) {
    const container = document.getElementById('recent-blocks');
    if (!container) return;
    
    if (!blocks || blocks.length === 0) {
        container.innerHTML = `
            <div class="empty-state py-8">
                <i class="fas fa-cube"></i>
                <p>No blocks yet</p>
            </div>
        `;
        return;
    }
    
    container.innerHTML = blocks.map(block => {
        const txs = Array.isArray(block.transactions) ? block.transactions : JSON.parse(block.txs_json || '[]');
        return `
            <div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition cursor-pointer" 
                 onclick='showModal("Block #${block.height}", ${JSON.stringify(block).replace(/'/g, '&#39;')})'>
                <div class="flex items-center gap-3">
                    <div class="w-10 h-10 rounded-lg bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center">
                        <i class="fas fa-cube text-blue-600"></i>
                    </div>
                    <div>
                        <p class="font-semibold">Block #${block.height}</p>
                        <p class="text-xs text-gray-500 dark:text-gray-400">${txs.length} tx${txs.length !== 1 ? 's' : ''}</p>
                    </div>
                </div>
                <i class="fas fa-chevron-right text-gray-400"></i>
            </div>
        `;
    }).join('');
}

/**
 * Refresh all data
 */
async function refreshAll() {
    const icon = document.getElementById('refresh-icon');
    if (icon) icon.classList.add('fa-spin');
    
    await loadDashboard();
    
    // Refresh current tab content
    const currentTab = UI.getCurrentTab();
    if (currentTab === 'blocks') await queryBlocks();
    else if (currentTab === 'transactions') await queryPool();
    
    if (icon) {
        setTimeout(() => icon.classList.remove('fa-spin'), 500);
    }
}

/**
 * Balance Tab
 */
async function queryBalance() {
    const account = document.getElementById('balance-account').value || 'admin';
    const token = document.getElementById('balance-token').value || 'proof';
    
    const data = await BalanceAPI.getBalance(account, token);
    
    if (!data.error) {
        const resultEl = document.getElementById('balance-result');
        if (resultEl) {
            resultEl.classList.remove('hidden');
            document.getElementById('balance-result-account').textContent = data.account;
            document.getElementById('balance-result-token').textContent = data.token;
            document.getElementById('balance-result-amount').textContent = data.balance;
            document.getElementById('balance-output').classList.add('hidden');
        }
    } else {
        document.getElementById('balance-result')?.classList.add('hidden');
        UI.displayOutput('balance-output', data);
    }
}

/**
 * Blocks Tab
 */
async function queryBlocks() {
    const container = document.getElementById('blocks-list');
    if (!container) return;
    
    UI.showLoading('blocks-list');
    const data = await BlockAPI.listBlocks();
    
    if (!data.error && Array.isArray(data)) {
        if (data.length === 0) {
            container.innerHTML = `
                <div class="empty-state">
                    <i class="fas fa-cube"></i>
                    <p>No blocks found</p>
                </div>
            `;
        } else {
            container.innerHTML = `
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>Height</th>
                            <th>Transactions</th>
                            <th class="hidden md:table-cell">Details</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${data.reverse().map(block => {
                            const txs = Array.isArray(block.transactions) ? block.transactions : JSON.parse(block.txs_json || '[]');
                            return `
                                <tr onclick='showModal("Block #${block.height}", ${JSON.stringify(block).replace(/'/g, '&#39;')})'>
                                    <td class="font-bold text-blue-600">#${block.height}</td>
                                    <td>${txs.length} tx${txs.length !== 1 ? 's' : ''}</td>
                                    <td class="hidden md:table-cell">
                                        <span class="badge badge-info">${txs.length} transactions</span>
                                    </td>
                                    <td>
                                        <button class="btn btn-sm btn-primary" onclick="event.stopPropagation(); showModal('Block #${block.height}', ${JSON.stringify(block).replace(/'/g, '&#39;')})">
                                            <i class="fas fa-eye"></i>
                                        </button>
                                    </td>
                                </tr>
                            `;
                        }).join('')}
                    </tbody>
                </table>
            `;
        }
    } else {
        container.innerHTML = `
            <div class="empty-state">
                <i class="fas fa-exclamation-circle"></i>
                <p class="text-red-600">Error: ${data.error || 'Failed to load blocks'}</p>
            </div>
        `;
    }
}

async function createBlock() {
    const data = await BlockAPI.createBlock();
    if (!data.error) {
        UI.showToast('Block created successfully!');
        await queryBlocks();
        await loadDashboard();
    } else {
        UI.showModal('Error Creating Block', data);
    }
}

/**
 * Transactions Tab
 */
async function queryPool() {
    const container = document.getElementById('pool-list');
    if (!container) return;
    
    UI.showLoading('pool-list');
    const data = await TransactionAPI.getPool();
    
    if (!data.error) {
        try {
            const pool = JSON.parse(data.pending_json || '[]');
            if (pool.length === 0) {
                container.innerHTML = `
                    <div class="empty-state">
                        <i class="fas fa-hourglass-half"></i>
                        <p>No pending transactions</p>
                    </div>
                `;
            } else {
                container.innerHTML = `
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th>#</th>
                                <th>Type</th>
                                <th class="hidden md:table-cell">Details</th>
                                <th>Actions</th>
                            </tr>
                        </thead>
                        <tbody>
                            ${pool.map((tx, idx) => {
                                const type = tx.kind ? Object.keys(tx.kind)[0] : 'Unknown';
                                return `
                                    <tr onclick='showModal("Transaction #${idx + 1}", ${JSON.stringify(tx).replace(/'/g, '&#39;')})'>
                                        <td class="font-bold">#${idx + 1}</td>
                                        <td><span class="badge badge-warning">${type}</span></td>
                                        <td class="hidden md:table-cell font-mono text-xs">${UI.truncateHash(JSON.stringify(tx), 30, 10)}</td>
                                        <td>
                                            <button class="btn btn-sm btn-primary" onclick="event.stopPropagation(); showModal('Transaction #${idx + 1}', ${JSON.stringify(tx).replace(/'/g, '&#39;')})">
                                                <i class="fas fa-eye"></i>
                                            </button>
                                        </td>
                                    </tr>
                                `;
                            }).join('')}
                        </tbody>
                    </table>
                `;
            }
        } catch (error) {
            container.innerHTML = `
                <div class="empty-state">
                    <i class="fas fa-exclamation-circle"></i>
                    <p class="text-red-600">Error parsing pool data</p>
                </div>
            `;
        }
    } else {
        container.innerHTML = `
            <div class="empty-state">
                <i class="fas fa-exclamation-circle"></i>
                <p class="text-red-600">Error: ${data.error}</p>
            </div>
        `;
    }
}

async function querySnapshot() {
    const data = await TransactionAPI.getSnapshot();
    UI.displayOutput('snapshot-output', data);
}

/**
 * Capsules Tab
 */
async function uploadCapsule() {
    const id = document.getElementById('capsule-id').value;
    const code = document.getElementById('capsule-code').value;
    
    if (!id || !code) {
        UI.displayOutput('capsule-upload-output', { error: 'ID and code are required' });
        return;
    }
    
    const data = await CapsuleAPI.upload(id, code);
    UI.displayOutput('capsule-upload-output', data);
    
    if (!data.error) {
        document.getElementById('capsule-id').value = '';
        document.getElementById('capsule-code').value = '';
        UI.showToast('Capsule uploaded successfully!');
    }
}

async function listCapsules() {
    const data = await CapsuleAPI.list();
    const output = document.getElementById('capsules-output');
    const list = document.getElementById('capsules-list');
    
    if (!data.error && data.capsules) {
        output?.classList.remove('hidden');
        if (data.capsules.length === 0) {
            if (list) list.innerHTML = '<p class="text-gray-500">No capsules deployed</p>';
        } else {
            if (list) {
                list.innerHTML = `
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                        ${data.capsules.map(id => `
                            <div class="p-3 bg-gray-50 dark:bg-gray-800 rounded-lg flex items-center justify-between">
                                <div class="flex items-center gap-2">
                                    <i class="fas fa-box text-purple-600"></i>
                                    <code class="text-sm">${id}</code>
                                </div>
                                <button onclick="copyToClipboard('${id}')" class="btn-icon btn-sm">
                                    <i class="fas fa-copy"></i>
                                </button>
                            </div>
                        `).join('')}
                    </div>
                `;
            }
        }
    } else {
        if (list) UI.displayOutput('capsules-list', data.error || 'Error loading capsules');
    }
}

async function invokeCapsule() {
    const id = document.getElementById('invoke-id').value;
    const input = document.getElementById('invoke-input').value;
    
    if (!id) {
        UI.displayOutput('capsule-invoke-output', { error: 'Capsule ID is required' });
        return;
    }
    
    const data = await CapsuleAPI.invoke(id, input || '');
    UI.displayOutput('capsule-invoke-output', data);
}

/**
 * Anchors Tab
 */
async function submitAnchor() {
    const id = document.getElementById('anchor-id').value;
    const proof = document.getElementById('anchor-proof').value;
    
    if (!id || !proof) {
        UI.displayOutput('anchor-submit-output', { error: 'ID and proof are required' });
        return;
    }
    
    try {
        const tx = createAnchorTx(id, proof);
        document.getElementById('tx-json').value = JSON.stringify(tx, null, 2);
        UI.displayOutput('anchor-submit-output', {
            status: 'Transaction prepared',
            message: 'Go to Wallet tab to sign and submit'
        });
        setTimeout(() => UI.switchTab('wallet'), 1500);
    } catch (error) {
        UI.displayOutput('anchor-submit-output', { error: 'Invalid base64 proof: ' + error.message });
    }
}

async function listAnchors() {
    const data = await AnchorAPI.list();
    UI.displayOutput('anchor-query-output', data);
}

async function getAnchor() {
    const id = document.getElementById('get-anchor-id').value;
    if (!id) {
        UI.displayOutput('anchor-query-output', { error: 'Anchor ID is required' });
        return;
    }
    
    const data = await AnchorAPI.get(id);
    UI.displayOutput('anchor-query-output', data);
}

/**
 * Wallet Tab
 */
function handleConnectWallet() {
    const pubkey = document.getElementById('wallet-pubkey').value.trim();
    const privkey = document.getElementById('wallet-privkey').value.trim();
    
    if (!pubkey || !privkey) {
        UI.showModal('Error', { error: 'Both keys are required' });
        return;
    }
    
    const result = connectWallet(pubkey, privkey);
    if (result.success) {
        document.getElementById('wallet-connected-status')?.classList.remove('hidden');
        UI.showToast('Wallet connected successfully!');
    } else {
        UI.showModal('Error', { error: result.error });
    }
}

async function signAndSubmit() {
    if (!isWalletConnected()) {
        UI.displayOutput('tx-submit-output', { error: 'Wallet not connected' });
        return;
    }
    
    const txJson = document.getElementById('tx-json').value.trim();
    if (!txJson) {
        UI.displayOutput('tx-submit-output', { error: 'Transaction JSON is required' });
        return;
    }
    
    try {
        const tx = JSON.parse(txJson);
        const signedTx = signTransaction(tx);
        const data = await TransactionAPI.submitTransaction(signedTx);
        
        UI.displayOutput('tx-submit-output', data);
        
        if (!data.error && data.success) {
            UI.showToast('Transaction submitted successfully!');
            setTimeout(() => {
                loadDashboard();
                if (UI.getCurrentTab() === 'transactions') queryPool();
            }, 1000);
        }
    } catch (error) {
        UI.displayOutput('tx-submit-output', { error: error.message });
    }
}

function beautifyTxJson() {
    const txJson = document.getElementById('tx-json').value.trim();
    if (txJson) {
        try {
            const formatted = JSON.stringify(JSON.parse(txJson), null, 2);
            document.getElementById('tx-json').value = formatted;
        } catch (e) {
            UI.showModal('Error', { error: 'Invalid JSON: ' + e.message });
        }
    }
}

function loadExampleTx() {
    const example = createTransferTx('admin', 'user1', 'Proof', 100);
    document.getElementById('tx-json').value = JSON.stringify(example, null, 2);
}

/**
 * Global search handler
 */
function handleGlobalSearch(event) {
    if (event.key === 'Enter') {
        const query = document.getElementById('globalSearch').value.trim();
        if (query) {
            if (!isNaN(query)) {
                UI.switchTab('blocks');
                queryBlocks();
            } else {
                UI.showModal('Search', { message: 'Advanced search coming soon!' });
            }
        }
    }
}

/**
 * Export for module usage
 */
export { init };
