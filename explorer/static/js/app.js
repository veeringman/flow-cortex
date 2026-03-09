/**
 * FlowCortex Explorer - Main Application
 * Modular architecture with ES6 modules
 */

import { BlockAPI, TransactionAPI, BalanceAPI, CapsuleAPI, AnchorAPI, NetworkAPI, TokenAPI } from './modules/api.js';
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
    
    // Load saved API base from localStorage — migrate stale external URLs to /api proxy
    let savedApiBase = localStorage.getItem('apiBase');
    // Clear stale external URLs that pointed to L1 directly (causes cert issues)
    if (savedApiBase && /^https?:\/\//.test(savedApiBase)) {
        localStorage.removeItem('apiBase');
        savedApiBase = null;
    }
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

    await refreshTokenOptions();
    
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
    window.createToken = createToken;
    window.listTokens = listTokens;
    window.getToken = getToken;
    window.uploadCapsule = uploadCapsule;
    window.listCapsules = listCapsules;
    window.invokeCapsule = invokeCapsule;
    window.submitAnchor = submitAnchor;
    window.listAnchors = listAnchors;
    window.getAnchor = getAnchor;
    window.getAnchorByHash = getAnchorByHash;
    window.copyHash = copyHash;
    window.showBlockDetail = showBlockDetail;
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
    
    // Listen for tab changes to auto-load data
    window.addEventListener('tabChanged', (e) => {
        if (e.detail.tab === 'anchors') listAnchors();
    });
    
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
    
    // Enforce HTTPS
    const safeUrl = newUrl.replace(/^http:\/\//, 'https://');
    
    // Update the API base
    window.setApiBase(safeUrl);
    
    // Update the display
    const endpoint = document.getElementById('api-endpoint');
    if (endpoint) {
        endpoint.textContent = safeUrl;
    }
    
    // Save to localStorage
    localStorage.setItem('apiBase', safeUrl);
    
    // Close modal and show success
    UI.closeModal('apiConfigModal');
    UI.showToast('success', `API configured: ${safeUrl}`);
    
    console.log(`✅ API Base updated to: ${safeUrl}`);
    
    // Reload dashboard data with new API endpoint
    await loadDashboard();

    await refreshTokenOptions();
    
    // Test connection
    updateNetworkStatus();
}

/**
 * Load dashboard data
 */
async function loadDashboard() {
    try {
        const [blocks, pool, capsules, snapshot, events] = await Promise.all([
            BlockAPI.listBlocks(),
            TransactionAPI.getPool(),
            CapsuleAPI.list(),
            TransactionAPI.getSnapshot(),
            AnchorAPI.list()
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
                rootEl.innerHTML = hashSpan(snapshot.root, { color: 'gray', len: 8, endLen: 8 });
            }
        }

        // Update anchors & proofs on dashboard
        if (!events.error && events.events) {
            const evts = events.events;
            const commitments = evts.filter(e => e.event_type === 'commitment.anchored');
            const proofs = evts.filter(e => e.event_type === 'proof.verified');
            const statAnchors = document.getElementById('stat-anchors');
            if (statAnchors) statAnchors.textContent = `${commitments.length} / ${proofs.length}`;
            updateRecentAnchors(evts);
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
        const bHash = block.block_hash || '';
        const shortBHash = bHash ? bHash.slice(0, 12) + '...' + bHash.slice(-8) : '';
        return `
            <div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition cursor-pointer" 
                 onclick='showModal("Block #${block.height}", ${JSON.stringify(block).replace(/'/g, '&#39;')})'>
                <div class="flex items-center gap-3">
                    <div class="w-10 h-10 rounded-lg bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center">
                        <i class="fas fa-cube text-blue-600"></i>
                    </div>
                    <div>
                        <p class="font-semibold">Block #${block.height}</p>
                        <p class="text-xs text-gray-500 dark:text-gray-400">${txs.length} tx${txs.length !== 1 ? 's' : ''}${shortBHash ? ' · <span class="font-mono">' + shortBHash + '</span>' : ''}</p>
                    </div>
                </div>
                ${bHash ? '<button onclick="event.stopPropagation(); copyHash(\'' + bHash + '\')" class="text-gray-400 hover:text-blue-500 mr-2" title="Copy block hash"><i class="fas fa-copy"></i></button>' : ''}
                <i class="fas fa-chevron-right text-gray-400"></i>
            </div>
        `;
    }).join('');
}

/**
 * Helper: render a copyable hash/address span
 */
function hashSpan(hash, opts = {}) {
    const { color = 'blue', len = 10, endLen = 6, icon = '', mono = true, clickFn = '' } = opts;
    if (!hash) return '';
    const short = hash.length > len + endLen + 3 ? hash.slice(0, len) + '...' + hash.slice(-endLen) : hash;
    const monoClass = mono ? 'font-mono' : '';
    const clickAttr = clickFn ? `onclick="${clickFn}"` : '';
    const cursorClass = clickFn ? 'cursor-pointer hover:underline' : '';
    return `<span class="inline-flex items-center gap-1">${icon ? `<i class="${icon} mr-0.5"></i>` : ''}<span class="${monoClass} text-xs text-${color}-600 dark:text-${color}-400 ${cursorClass}" title="${hash}" ${clickAttr}>${short}</span><button onclick="event.stopPropagation(); copyHash('${hash}')" class="text-gray-400 hover:text-${color}-500" title="Copy"><i class="fas fa-copy text-xs"></i></button></span>`;
}

/**
 * Update recent anchors & proofs on dashboard
 */
function updateRecentAnchors(events) {
    const container = document.getElementById('recent-anchors');
    if (!container) return;

    if (!events || events.length === 0) {
        container.innerHTML = `<div class="text-gray-500 text-center py-4"><i class="fas fa-inbox opacity-50"></i><br/><span class="text-xs">No commitments or proofs yet</span></div>`;
        return;
    }

    // Show latest events first (reverse chronological)
    const sorted = [...events].reverse().slice(0, 8);

    container.innerHTML = sorted.map(evt => {
        if (evt.event_type === 'commitment.anchored') {
            const ts = new Date(evt.timestamp * 1000).toLocaleString();
            return `
                <div class="flex items-center justify-between p-3 bg-indigo-50 dark:bg-indigo-900/20 rounded-lg">
                    <div class="flex items-center gap-3">
                        <div class="w-8 h-8 rounded-lg bg-indigo-100 dark:bg-indigo-900/40 flex items-center justify-center">
                            <i class="fas fa-anchor text-indigo-600 text-sm"></i>
                        </div>
                        <div>
                            <p class="text-xs font-semibold">Commitment Anchored</p>
                            <div class="mt-0.5">${hashSpan(evt.commitment_hash, { color: 'indigo', len: 14, endLen: 6, clickFn: `switchTab('anchors')` })}</div>
                        </div>
                    </div>
                    <div class="text-right text-xs text-gray-500">
                        <div>Block ${evt.block_height}</div>
                        <div>${ts}</div>
                    </div>
                </div>`;
        } else {
            const ts = new Date(evt.timestamp * 1000).toLocaleString();
            return `
                <div class="flex items-center justify-between p-3 bg-green-50 dark:bg-green-900/20 rounded-lg">
                    <div class="flex items-center gap-3">
                        <div class="w-8 h-8 rounded-lg bg-green-100 dark:bg-green-900/40 flex items-center justify-center">
                            <i class="fas fa-shield-alt text-green-600 text-sm"></i>
                        </div>
                        <div>
                            <p class="text-xs font-semibold">${evt.verified ? '\u2713 Proof Verified' : '\u2717 Proof Failed'}</p>
                            <div class="mt-0.5">${hashSpan(evt.proof_hash, { color: 'green', len: 14, endLen: 6 })}</div>
                        </div>
                    </div>
                    <div class="text-right text-xs text-gray-500">
                        <div>Block ${evt.block_height}</div>
                        <div>${ts}</div>
                    </div>
                </div>`;
        }
    }).join('');
}

/**
 * Refresh all data
 */
async function refreshAll() {
    const icon = document.getElementById('refresh-icon');
    if (icon) icon.classList.add('fa-spin');
    
    await loadDashboard();
    await refreshTokenOptions();
    
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

async function refreshTokenOptions() {
    const select = document.getElementById('balance-token');
    if (!select) return;

    const data = await TokenAPI.listTokens();
    if (!data || data.error) return;

    const tokens = Array.isArray(data) ? data : (data.tokens || []);
    const current = select.value;
    select.innerHTML = '';

    if (!tokens.length) {
        const option = document.createElement('option');
        option.value = '';
        option.textContent = 'No tokens available';
        select.appendChild(option);
        return;
    }

    const placeholder = document.createElement('option');
    placeholder.value = '';
    placeholder.textContent = '-- Select Token --';
    select.appendChild(placeholder);

    tokens.forEach((token) => {
        const symbol = (token.symbol || '').toString();
        if (!symbol) return;
        const option = document.createElement('option');
        option.value = symbol;
        option.textContent = token.name ? `${symbol.toUpperCase()} - ${token.name}` : symbol.toUpperCase();
        select.appendChild(option);
    });

    if (current && Array.from(select.options).some(opt => opt.value === current)) {
        select.value = current;
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
                            <th>Block Hash</th>
                            <th>Transactions</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${data.reverse().map(block => {
                            const txs = Array.isArray(block.transactions) ? block.transactions : [];
                            const bHash = block.block_hash || '';
                            return `
                                <tr onclick='showBlockDetail(${JSON.stringify(block).replace(/'/g, '&#39;')})'>
                                    <td class="font-bold text-blue-600">#${block.height}</td>
                                    <td>${bHash ? hashSpan(bHash, { color: 'blue', len: 12, endLen: 8 }) : '-'}</td>
                                    <td>${txs.length} tx${txs.length !== 1 ? 's' : ''}</td>
                                    <td>
                                        <button class="btn btn-sm btn-primary" onclick="event.stopPropagation(); showBlockDetail(${JSON.stringify(block).replace(/'/g, '&#39;')})">
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

function showBlockDetail(block) {
    const detailEl = document.getElementById('blockDetails');
    const placeholderEl = document.getElementById('blockDetailsPlaceholder');
    if (!detailEl) return;
    if (placeholderEl) placeholderEl.classList.add('hidden');
    detailEl.classList.remove('hidden');

    const txs = Array.isArray(block.transactions) ? block.transactions : [];
    const bHash = block.block_hash || '';

    let html = `<div class="mb-4">`;
    html += `<h4 class="text-lg font-bold mb-2">Block #${block.height}</h4>`;
    if (bHash) {
        html += `<div class="flex items-center gap-2 mb-2"><span class="text-xs text-gray-500">Block Hash:</span>${hashSpan(bHash, { color: 'blue', len: 32, endLen: 12 })}</div>`;
    }
    html += `<span class="text-sm text-gray-500">${txs.length} transaction${txs.length !== 1 ? 's' : ''}</span>`;
    html += `</div>`;

    if (txs.length > 0) {
        html += `<div class="space-y-2">`;
        for (let i = 0; i < txs.length; i++) {
            const tx = txs[i];
            const txHash = tx.tx_hash || '';
            const kind = tx.kind || {};
            const txType = Object.keys(kind)[0] || 'Unknown';
            const detail = kind[txType] || {};

            html += `<div class="p-3 rounded-lg bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700">`;
            html += `<div class="flex items-center justify-between mb-1">`;
            html += `<span class="text-sm font-semibold">${txType}</span>`;
            html += `<span class="badge badge-info text-xs">TX #${i}</span>`;
            html += `</div>`;
            if (txHash) {
                html += `<div class="flex items-center gap-1 mb-2"><span class="text-xs text-gray-400">TX Hash:</span>${hashSpan(txHash, { color: 'purple', len: 20, endLen: 10 })}</div>`;
            }
            if (detail.from) {
                html += `<div class="flex items-center gap-1 mb-0.5"><span class="text-xs text-gray-500">From:</span>${hashSpan(detail.from, { color: 'blue', len: 16, endLen: 8 })}</div>`;
            }
            if (detail.to) {
                html += `<div class="flex items-center gap-1 mb-0.5"><span class="text-xs text-gray-500">To:</span>${hashSpan(detail.to, { color: 'blue', len: 16, endLen: 8 })}</div>`;
            }
            if (detail.token) {
                html += `<div class="text-xs text-gray-500 mb-0.5">Token: <span class="font-semibold">${detail.token}</span></div>`;
            }
            if (detail.amount !== undefined) {
                html += `<div class="text-xs text-gray-500 mb-0.5">Amount: <span class="font-semibold">${detail.amount.toLocaleString()}</span></div>`;
            }
            if (detail.reference) {
                html += `<div class="flex items-center gap-1 mb-0.5"><span class="text-xs text-gray-500">Ref:</span>${hashSpan(detail.reference, { color: 'gray', len: 20, endLen: 10 })}</div>`;
            }
            html += `</div>`;
        }
        html += `</div>`;
    }

    detailEl.innerHTML = html;
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
 * Tokens Tab
 */
async function createToken() {
    const symbol = document.getElementById('token-symbol')?.value.trim();
    const name = document.getElementById('token-name')?.value.trim();
    const decimals = parseInt(document.getElementById('token-decimals')?.value, 10);
    const initialSupply = parseInt(document.getElementById('token-supply')?.value, 10);
    const tokenType = document.getElementById('token-type')?.value || 'stablecoin';
    const metadataJson = document.getElementById('token-metadata')?.value.trim();

    if (!symbol || !name || Number.isNaN(decimals) || Number.isNaN(initialSupply)) {
        UI.displayOutput('token-create-output', { error: 'Symbol, name, decimals, and initial supply are required' });
        return;
    }

    const payload = {
        symbol,
        name,
        decimals,
        initial_supply: initialSupply,
        token_type: tokenType,
        metadata_json: metadataJson || ''
    };

    const data = await TokenAPI.createToken(payload);
    UI.displayOutput('token-create-output', data);
    if (!data.error && data.success) {
        UI.showToast(`Token created: ${data.symbol || symbol}`);
        await listTokens();
        await refreshTokenOptions();
    }
}

async function listTokens() {
    const list = document.getElementById('tokens-list');
    const data = await TokenAPI.listTokens();

    if (!data.error) {
        const tokens = Array.isArray(data) ? data : (data.tokens || []);
        if (list) {
            if (!tokens.length) {
                list.innerHTML = '<p class="text-gray-500 dark:text-gray-400 text-center py-4">No tokens found</p>';
            } else {
                const tokenLogo = (symbol) => {
                    const key = (symbol || '').toString().trim().toUpperCase();
                    if (key === 'FLOWER' || key === 'FLOWER'.toUpperCase()) return '/static/img/flower_logo.png';
                    if (key === 'PROOF') return '/static/img/proof_logo.png';
                    return '';
                };
                list.innerHTML = `
                    <div class="grid grid-cols-1 gap-3">
                        ${tokens.map(token => `
                            <div class="p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
                                <div class="flex justify-between items-center">
                                    <div class="flex items-center gap-2">
                                        ${tokenLogo(token.symbol) ? `<img src="${tokenLogo(token.symbol)}" alt="${token.symbol || 'token'}" class="token-logo" />` : ''}
                                        <div>
                                        <p class="font-semibold">${token.symbol || token.name || 'Token'}</p>
                                        <p class="text-xs text-gray-500 dark:text-gray-400">${token.name || ''}</p>
                                        </div>
                                    </div>
                                    <div class="text-right text-xs text-gray-500 dark:text-gray-400">
                                        <div>Supply: ${token.total_supply ?? '-'}</div>
                                        <div>Status: ${token.status || 'active'}</div>
                                    </div>
                                </div>
                            </div>
                        `).join('')}
                    </div>
                `;
            }
        }
        document.getElementById('tokens-output')?.classList.add('hidden');
    } else {
        UI.displayOutput('tokens-output', data);
    }
}

async function getToken() {
    const symbol = document.getElementById('token-lookup')?.value.trim();
    if (!symbol) {
        UI.displayOutput('token-detail-output', { error: 'Token symbol is required' });
        return;
    }
    const data = await TokenAPI.getToken(symbol);
    UI.displayOutput('token-detail-output', data);
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
    const listEl = document.getElementById('anchors-list');
    if (!listEl) return;

    if (data.error || !data.events || data.events.length === 0) {
        listEl.innerHTML = '<div class="text-gray-500 text-center py-4"><i class="fas fa-inbox text-lg opacity-50"></i><br/><span class="text-xs">No anchors or proofs found</span></div>';
        return;
    }

    const commitments = data.events.filter(e => e.event_type === 'commitment.anchored');
    const proofs = data.events.filter(e => e.event_type === 'proof.verified');

    let html = '';

    // Stats summary
    html += `<div class="mb-3 p-3 rounded-lg bg-blue-50 dark:bg-blue-900/30 border border-blue-200 dark:border-blue-700">`;
    html += `<div class="flex items-center gap-4 text-sm">`;
    html += `<span class="font-semibold text-blue-700 dark:text-blue-300"><i class="fas fa-anchor mr-1"></i>${commitments.length} Commitments</span>`;
    html += `<span class="font-semibold text-green-700 dark:text-green-300"><i class="fas fa-shield-alt mr-1"></i>${proofs.length} Proofs Verified</span>`;
    html += `</div></div>`;

    // Commitments
    if (commitments.length > 0) {
        html += `<div class="mb-2 text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Commitment Anchors</div>`;
        for (const c of commitments) {
            const hash = c.commitment_hash;
            const ts = new Date(c.timestamp * 1000).toLocaleString();
            html += `<div class="p-3 mb-2 rounded-lg bg-indigo-50 dark:bg-indigo-900/20 border border-indigo-200 dark:border-indigo-700 cursor-pointer hover:shadow" onclick="getAnchorByHash('${hash}')">`;
            html += `<div class="flex items-center justify-between">`;
            html += `<span class="inline-flex items-center gap-1">${hashSpan(hash, { color: 'indigo', len: 12, endLen: 8, icon: 'fas fa-anchor', clickFn: "getAnchorByHash('" + hash + "')" })}</span>`;
            html += `<span class="text-xs text-gray-500">Block ${c.block_height}</span>`;
            html += `</div>`;
            html += `<div class="text-xs text-gray-500 mt-1">${ts}</div>`;
            html += `</div>`;
        }
    }

    // Proofs
    if (proofs.length > 0) {
        html += `<div class="mt-3 mb-2 text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Verified Proofs</div>`;
        for (const p of proofs) {
            const proofHash = p.proof_hash;
            const commitHash = p.commitment_hash;
            const ts = new Date(p.timestamp * 1000).toLocaleString();
            html += `<div class="p-3 mb-2 rounded-lg bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-700">`;
            html += `<div class="flex items-center justify-between">`;
            html += `<span class="inline-flex items-center gap-1">${hashSpan(proofHash, { color: 'green', len: 12, endLen: 8, icon: 'fas fa-shield-alt' })}</span>`;
            html += `<span class="text-xs ${p.verified ? 'text-green-600' : 'text-red-600'}">${p.verified ? '✓ Verified' : '✗ Failed'}</span>`;
            html += `</div>`;
            html += `<div class="flex items-center gap-1 text-xs text-gray-500 mt-1">Commitment: ${hashSpan(commitHash, { color: 'indigo', len: 12, endLen: 8, clickFn: "getAnchorByHash('" + commitHash + "')" })} · Block ${p.block_height} · ${ts}</div>`;
            html += `</div>`;
        }
    }

    listEl.innerHTML = html;
    // Also show raw data in the output panel
    UI.displayOutput('anchor-query-output', data);
}

async function getAnchorByHash(hash) {
    const data = await AnchorAPI.get(hash);
    // Show formatted detail in anchor-query-output
    const outEl = document.getElementById('anchor-query-output');
    if (outEl && !data.error) {
        let html = `<div class="p-4 space-y-2">`;
        html += `<h4 class="font-bold text-sm mb-3"><i class="fas fa-anchor mr-1"></i> Commitment Detail</h4>`;
        html += `<div class="flex items-center gap-2"><span class="text-xs text-gray-500 w-28">Commitment:</span>${hashSpan(data.commitment_hash, { color: 'indigo', len: 24, endLen: 12 })}</div>`;
        if (data.policy_id) html += `<div class="flex items-center gap-2"><span class="text-xs text-gray-500 w-28">Policy:</span><span class="text-xs font-semibold">${data.policy_id}</span></div>`;
        if (data.txn_ref) html += `<div class="flex items-center gap-2"><span class="text-xs text-gray-500 w-28">TX Ref:</span>${hashSpan(data.txn_ref, { color: 'purple', len: 20, endLen: 10 })}</div>`;
        if (data.context_ref) html += `<div class="flex items-center gap-2"><span class="text-xs text-gray-500 w-28">Context:</span><span class="text-xs font-semibold">${data.context_ref}</span></div>`;
        html += `<div class="flex items-center gap-2"><span class="text-xs text-gray-500 w-28">Block Height:</span><span class="text-xs font-bold">${data.block_height}</span></div>`;
        html += `<div class="flex items-center gap-2"><span class="text-xs text-gray-500 w-28">Verified:</span><span class="text-xs ${data.verified ? 'text-green-600 font-bold' : 'text-red-600'}">${data.verified ? '\u2713 Yes' : '\u2717 No'}</span></div>`;
        if (data.timestamp) html += `<div class="flex items-center gap-2"><span class="text-xs text-gray-500 w-28">Timestamp:</span><span class="text-xs">${new Date(data.timestamp * 1000).toLocaleString()}</span></div>`;
        html += `</div>`;
        outEl.innerHTML = html;
        outEl.classList.remove('hidden');
    } else {
        UI.displayOutput('anchor-query-output', data);
    }
}

function copyHash(hash) {
    navigator.clipboard.writeText(hash).then(() => {
        UI.showToast('Hash copied!');
    }).catch(() => {
        // fallback
        const ta = document.createElement('textarea');
        ta.value = hash; document.body.appendChild(ta); ta.select();
        document.execCommand('copy'); document.body.removeChild(ta);
        UI.showToast('Hash copied!');
    });
}

async function getAnchor() {
    const id = document.getElementById('get-anchor-id').value;
    if (!id) {
        UI.displayOutput('anchor-query-output', { error: 'Commitment hash is required' });
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
