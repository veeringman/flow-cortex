/**
 * UI Module - Handles UI interactions, theme, and utilities
 */

/**
 * Current active tab
 */
let currentTab = 'dashboard';

/**
 * Switch between tabs/pages
 */
export function switchTab(tabName) {
    currentTab = tabName;
    
    // Hide all content
    document.querySelectorAll('.tab-content').forEach(el => {
        el.classList.add('hidden');
    });
    
    // Show selected content
    const contentEl = document.getElementById(`${tabName}-content`);
    if (contentEl) {
        contentEl.classList.remove('hidden');
    }
    
    // Update sidebar active state
    document.querySelectorAll('.tab-button').forEach(btn => {
        btn.classList.remove('active');
    });
    
    const activeBtn = document.querySelector(`[data-tab="${tabName}"]`);
    if (activeBtn) {
        activeBtn.classList.add('active');
    }
    
    // Update mobile nav
    document.querySelectorAll('.mobile-nav-btn').forEach(btn => {
        btn.classList.remove('active');
        if (btn.dataset.tab === tabName) {
            btn.classList.add('active');
        }
    });
    
    // Update page title
    const titles = {
        dashboard: { title: 'Dashboard', subtitle: 'Network Overview & Statistics' },
        balance: { title: 'Account Balance', subtitle: 'Check account balances' },
        tokens: { title: 'Tokens', subtitle: 'Create and manage tokens' },
        blocks: { title: 'Blocks', subtitle: 'Explore blockchain blocks' },
        transactions: { title: 'Transactions', subtitle: 'View transaction pool' },
        capsules: { title: 'Smart Contracts', subtitle: 'Deploy and execute capsules' },
        anchors: { title: 'Anchors & Proofs', subtitle: 'Anchor proof data' },
        wallet: { title: 'Wallet', subtitle: 'Sign and submit transactions' }
    };
    
    const titleInfo = titles[tabName] || { title: 'FlowCortex', subtitle: '' };
    const titleEl = document.getElementById('pageTitle');
    const subtitleEl = document.getElementById('pageSubtitle');
    
    if (titleEl) titleEl.textContent = titleInfo.title;
    if (subtitleEl) subtitleEl.textContent = titleInfo.subtitle;
    
    // Close sidebar on mobile
    closeSidebar();
    
    // Dispatch custom event for tab changes
    window.dispatchEvent(new CustomEvent('tabChanged', { detail: { tab: tabName } }));
}

/**
 * Get current tab
 */
export function getCurrentTab() {
    return currentTab;
}

/**
 * Sidebar management
 */
export function openSidebar() {
    document.querySelector('.sidebar')?.classList.add('mobile-open');
    document.getElementById('overlay')?.classList.add('active');
}

export function closeSidebar() {
    document.querySelector('.sidebar')?.classList.remove('mobile-open');
    document.getElementById('overlay')?.classList.remove('active');
}

/**
 * Theme management
 */
export function toggleTheme() {
    const html = document.documentElement;
    const isDark = html.classList.toggle('dark');
    localStorage.setItem('theme', isDark ? 'dark' : 'light');
    
    const icon = document.getElementById('theme-icon');
    const text = document.getElementById('theme-text');
    
    if (icon && text) {
        if (isDark) {
            icon.className = 'fas fa-sun';
            text.textContent = 'Light Mode';
        } else {
            icon.className = 'fas fa-moon';
            text.textContent = 'Dark Mode';
        }
    }
    
    // Dispatch theme change event
    window.dispatchEvent(new CustomEvent('themeChanged', { detail: { dark: isDark } }));
}

export function initTheme() {
    const savedTheme = localStorage.getItem('theme');
    if (savedTheme === 'dark') {
        document.documentElement.classList.add('dark');
        const icon = document.getElementById('theme-icon');
        const text = document.getElementById('theme-text');
        if (icon) icon.className = 'fas fa-sun';
        if (text) text.textContent = 'Light Mode';
    }
}

/**
 * Modal management
 */
export function showModal(modalIdOrTitle, content) {
    // Handle both old API (showModal(title, content)) and new API (showModal(modalId))
    
    // If content is provided, use old behavior
    if (content !== undefined) {
        const modal = document.getElementById('detailModal');
        const titleEl = document.getElementById('modal-title');
        const contentEl = document.getElementById('modal-content');
        
        if (!modal || !titleEl || !contentEl) return;
        
        titleEl.textContent = modalIdOrTitle;
        
        if (typeof content === 'object') {
            contentEl.innerHTML = `<pre>${JSON.stringify(content, null, 2)}</pre>`;
        } else {
            contentEl.textContent = content;
        }
        
        modal.classList.add('active');
        return;
    }
    
    // New behavior: show modal by ID
    const modal = document.getElementById(modalIdOrTitle);
    if (modal) {
        modal.classList.remove('hidden');
    }
}

export function closeModal(modalId) {
    // If no modalId provided, use old behavior
    if (!modalId) {
        document.getElementById('detailModal')?.classList.remove('active');
        return;
    }
    
    // New behavior: close modal by ID
    const modal = document.getElementById(modalId);
    if (modal) {
        modal.classList.add('hidden');
    }
}

/**
 * Display output in code box
 */
export function displayOutput(elementId, data, show = true) {
    const el = document.getElementById(elementId);
    if (!el) return;
    
    if (typeof data === 'object') {
        el.innerHTML = `<pre>${JSON.stringify(data, null, 2)}</pre>`;
    } else {
        el.textContent = data;
    }
    
    if (show) {
        el.classList.remove('hidden');
    }
}

/**
 * Utility functions
 */
export function truncateHash(hash, start = 10, end = 8) {
    if (!hash || hash.length <= start + end) return hash;
    return `${hash.slice(0, start)}...${hash.slice(-end)}`;
}

export function formatTimestamp(timestamp) {
    return new Date(timestamp).toLocaleString();
}

export function copyToClipboard(text) {
    navigator.clipboard.writeText(text).then(() => {
        showToast('Copied to clipboard!');
    }).catch(err => {
        console.error('Copy failed:', err);
        showToast('Copy failed', 'error');
    });
}

export function showToast(message, type = 'success') {
    // Simple toast implementation
    // TODO: Replace with a better toast library or custom component
    const toast = document.createElement('div');
    toast.className = `fixed bottom-4 right-4 px-6 py-3 rounded-lg shadow-lg z-50 ${
        type === 'error' ? 'bg-red-500' : 'bg-green-500'
    } text-white`;
    toast.textContent = message;
    document.body.appendChild(toast);
    
    setTimeout(() => {
        toast.style.opacity = '0';
        toast.style.transition = 'opacity 0.3s';
        setTimeout(() => toast.remove(), 300);
    }, 3000);
}

/**
 * Loading indicator
 */
export function showLoading(elementId) {
    const el = document.getElementById(elementId);
    if (el) {
        el.innerHTML = '<div class="flex justify-center py-8"><div class="spinner"></div></div>';
    }
}

export function hideLoading(elementId) {
    const el = document.getElementById(elementId);
    if (el) {
        el.innerHTML = '';
    }
}

/**
 * Initialize UI event listeners
 */
export function initUI() {
    // Close modal on Escape
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
            closeModal();
            closeSidebar();
        }
    });
    
    // Close sidebar when clicking outside on mobile
    document.addEventListener('click', (e) => {
        const sidebar = document.querySelector('.sidebar');
        const toggleBtn = document.querySelector('[onclick="openSidebar()"]');
        
        if (window.innerWidth <= 768 && 
            sidebar && 
            !sidebar.contains(e.target) && 
            toggleBtn && 
            !toggleBtn.contains(e.target)) {
            closeSidebar();
        }
    });
    
    console.log('✅ UI Module initialized');
}
