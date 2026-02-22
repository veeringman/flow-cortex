#!/usr/bin/env python3
"""
Generate the upgraded FlowCortex Explorer UI
"""

import os

EXPLORER_HTML = '''<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
    <title>FlowCortex Explorer - Professional Blockchain Explorer</title>
    <meta name="description" content="FlowCortex Blockchain Explorer - View blocks, transactions, capsules, and network statistics">
    
    <!-- Tailwind CSS -->
    <script src="https://cdn.tailwindcss.com"></script>
    
    <!-- Chart.js for data visualization -->
    <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.min.js"></script>
    
    <!-- Crypto libraries -->
    <script src="https://cdn.jsdelivr.net/npm/tweetnacl@1.0.3/nacl.min.js"></script>
    
    <!-- Font Awesome -->
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.0/css/all.min.css">
    
    <style>
        :root {
            --primary: #3b82f6;
            --primary-dark: #1e40af;
            --primary-light: #60a5fa;
            --secondary: #10b981;
            --danger: #ef4444;
            --warning: #f59e0b;
            --purple: #8b5cf6;
            --pink: #ec4899;
        }
        
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
        }

        /* Sidebar Styles */
        .sidebar {
            transition: transform 0.3s ease-in-out;
            height: 100vh;
            position: fixed;
            left: 0;
            top: 0;
            z-index: 40;
            overflow-y: auto;
            scrollbar-width: thin;
        }

        .sidebar::-webkit-scrollbar {
            width: 6px;
        }

        .sidebar::-webkit-scrollbar-thumb {
            background: rgba(156, 163, 175, 0.5);
            border-radius: 3px;
        }

        @media (max-width: 1024px) {
            .sidebar {
                transform: translateX(-100%);
            }

            .sidebar.mobile-open {
                transform: translateX(0);
                box-shadow: 4px 0 12px rgba(0, 0, 0, 0.1);
            }

            .main-content {
                margin-left: 0 !important;
            }
        }

        /* Tab Navigation */
        .tab-button {
            position: relative;
            transition: all 0.2s ease;
        }

        .tab-button::before {
            content: '';
            position: absolute;
            left: 0;
            top: 50%;
            transform: translateY(-50%);
            width: 0;
            height: 0;
            background: var(--primary);
            border-radius: 0 4px 4px 0;
            transition: all 0.3s ease;
        }

        .tab-button.active::before {
            width: 4px;
            height: 80%;
        }

        .tab-button.active {
            background: linear-gradient(90deg, rgba(59, 130, 246, 0.1) 0%, transparent 100%);
            color: var(--primary);
        }

        .tab-button:hover:not(.active) {
            background-color: rgba(156, 163, 175, 0.1);
        }

        /* Card Styles */
        .card {
            border-radius: 12px;
            background: white;
            border: 1px solid #e5e7eb;
            transition: all 0.3s ease;
        }

        .dark .card {
            background: #1f2937;
            border-color: #374151;
        }

        .card:hover {
            box-shadow: 0 8px 24px rgba(0, 0, 0, 0.08);
            transform: translateY(-2px);
        }

        /* Stat Card */
        .stat-card {
            position: relative;
            overflow: hidden;
        }

        .stat-card::before {
            content: '';
            position: absolute;
            top: 0;
            right: 0;
            width: 100px;
            height: 100px;
            background: radial-gradient(circle, currentColor 0%, transparent 70%);
            opacity: 0.05;
            pointer-events: none;
        }

        /* Button Styles */
        .btn {
            padding: 0.625rem 1.25rem;
            border-radius: 8px;
            font-weight: 500;
            transition: all 0.2s ease;
            cursor: pointer;
            border: none;
            display: inline-flex;
            align-items: center;
            justify-content: center;
            gap: 0.5rem;
        }

        .btn-primary {
            background: linear-gradient(135deg, var(--primary) 0%, var(--primary-dark) 100%);
            color: white;
        }

        .btn-primary:hover:not(:disabled) {
            transform: translateY(-1px);
            box-shadow: 0 6px 20px rgba(59, 130, 246, 0.4);
        }

        .btn-secondary {
            background: var(--secondary);
            color: white;
        }

        .btn-secondary:hover:not(:disabled) {
            background: #059669;
        }

        .btn:disabled {
            opacity: 0.5;
            cursor: not-allowed;
        }

        .btn-sm {
            padding: 0.375rem 0.75rem;
            font-size: 0.875rem;
        }

        .btn-icon {
            width: 36px;
            height: 36px;
            padding: 0;
            border-radius: 8px;
        }

        /* Input Styles */
        .input-field {
            width: 100%;
            padding: 0.625rem 0.875rem;
            border: 1px solid #d1d5db;
            border-radius: 8px;
            transition: all 0.2s ease;
            background: white;
        }

        .dark .input-field {
            background: #1f2937;
            border-color: #374151;
            color: white;
        }

        .input-field:focus {
            outline: none;
            border-color: var(--primary);
            box-shadow: 0 0 0 4px rgba(59, 130, 246, 0.1);
        }

        /* Output/Code Box */
        .output-box {
            max-height: 500px;
            overflow-y: auto;
            background: #f9fafb;
            border: 1px solid #e5e7eb;
            border-radius: 8px;
            padding: 1rem;
            font-family: 'Monaco', 'Courier New', monospace;
            font-size: 0.8125rem;
            line-height: 1.6;
        }

        .dark .output-box {
            background: #111827;
            border-color: #374151;
        }

        .output-box pre {
            margin: 0;
            white-space: pre-wrap;
            word-break: break-all;
        }

        /* Badge Styles */
        .badge {
            display: inline-flex;
            align-items: center;
            gap: 0.25rem;
            padding: 0.25rem 0.625rem;
            border-radius: 12px;
            font-size: 0.75rem;
            font-weight: 600;
        }

        .badge-success {
            background: #d1fae5;
            color: #065f46;
        }

        .badge-error {
            background: #fee2e2;
            color: #991b1b;
        }

        .badge-warning {
            background: #fef3c7;
            color: #92400e;
        }

        .badge-info {
            background: #dbeafe;
            color: #1e40af;
        }

        .badge-purple {
            background: #ede9fe;
            color: #5b21b6;
        }

        /* Modal */
       .modal {
            display: none;
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.6);
            z-index: 1000;
            align-items: center;
            justify-content: center;
            backdrop-filter: blur(4px);
            animation: fadeIn 0.2s ease;
        }

        .modal.active {
            display: flex;
        }

        .modal-content {
            background: white;
            border-radius: 16px;
            max-width: 90vw;
            max-height: 90vh;
            overflow-y: auto;
            box-shadow: 0 25px 50px rgba(0, 0, 0, 0.3);
            animation: slideUp 0.3s ease;
        }

        .dark .modal-content {
            background: #1f2937;
        }

        @keyframes fadeIn {
            from { opacity: 0; }
            to { opacity: 1; }
        }

        @keyframes slideUp {
            from { transform: translateY(20px); opacity: 0; }
            to { transform: translateY(0); opacity: 1; }
        }

        /* Table */
        .data-table {
            width: 100%;
            border-collapse: separate;
            border-spacing: 0;
        }

        .data-table th {
            background: #f9fafb;
            padding: 0.875rem 1rem;
            text-align: left;
            font-weight: 600;
            font-size: 0.75rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: #6b7280;
            border-bottom: 2px solid #e5e7eb;
        }

        .dark .data-table th {
            background: #111827;
            color: #9ca3af;
            border-color: #374151;
        }

        .data-table td {
            padding: 1rem;
            border-bottom: 1px solid #f3f4f6;
        }

        .dark .data-table td {
            border-color: #374151;
        }

        .data-table tbody tr {
            transition: background 0.15s ease;
            cursor: pointer;
        }

        .data-table tbody tr:hover {
            background: #f9fafb;
        }

        .dark .data-table tbody tr:hover {
            background: #111827;
        }

        /* Spinner */
        .spinner {
            border: 3px solid rgba(59, 130, 246, 0.1);
            border-radius: 50%;
            border-top-color: var(--primary);
            width: 24px;
            height: 24px;
            animation: spin 0.8s linear infinite;
        }

        @keyframes spin {
            to { transform: rotate(360deg); }
        }

        /* Tooltip */
        .tooltip {
            position: relative;
        }

        .tooltip::after {
            content: attr(data-tooltip);
            position: absolute;
            bottom: 100%;
            left: 50%;
            transform: translateX(-50%) translateY(-8px);
            background: #111827;
            color: white;
            padding: 0.5rem 0.75rem;
            border-radius: 6px;
            font-size: 0.75rem;
            white-space: nowrap;
            opacity: 0;
            pointer-events: none;
            transition: opacity 0.2s ease;
            z-index: 100;
        }

        .tooltip:hover::after {
            opacity: 1;
        }

        /* Search Bar */
        .search-bar {
            position: relative;
        }

        .search-bar input {
            padding-left: 2.5rem;
        }

        .search-bar i {
            position: absolute;
            left: 0.875rem;
            top: 50%;
            transform: translateY(-50%);
            color: #9ca3af;
        }

        /* Mobile Bottom Nav */
        .mobile-nav {
            display: none;
            position: fixed;
            bottom: 0;
            left: 0;
            right: 0;
            background: white;
            border-top: 1px solid #e5e7eb;
            padding: 0.5rem;
            z-index: 50;
            box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.1);
        }

        .dark .mobile-nav {
            background: #1f2937;
            border-color: #374151;
        }

        @media (max-width: 768px) {
            .mobile-nav {
                display: flex;
            }
            
            .main-content {
                padding-bottom: 80px;
            }
        }

        .mobile-nav-btn {
            flex: 1;
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 0.25rem;
            padding: 0.5rem;
            border-radius: 8px;
            font-size: 0.625rem;
            transition: all 0.2s ease;
        }

        .mobile-nav-btn.active {
            background: rgba(59, 130, 246, 0.1);
            color: var(--primary);
        }

        /* Scrollbar */
        .custom-scrollbar {
            scrollbar-width: thin;
            scrollbar-color: #cbd5e1 transparent;
        }

        .custom-scrollbar::-webkit-scrollbar {
            width: 8px;
            height: 8px;
        }

        .custom-scrollbar::-webkit-scrollbar-track {
            background: transparent;
        }

        .custom-scrollbar::-webkit-scrollbar-thumb {
            background: #cbd5e1;
            border-radius: 4px;
        }

        .custom-scrollbar::-webkit-scrollbar-thumb:hover {
            background: #94a3b8;
        }

        /* Overlay */
        .overlay {
            display: none;
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.5);
            z-index: 30;
        }

        .overlay.active {
            display: block;
        }

        /* Empty State */
        .empty-state {
            text-align: center;
            padding: 3rem 1rem;
            color: #9ca3af;
        }

        .empty-state i {
            font-size: 3rem;
            margin-bottom: 1rem;
            opacity: 0.3;
        }

        @media (max-width: 640px) {
            .card {
                border-radius: 8px;
            }
            
            .btn {
                padding: 0.5rem 1rem;
                font-size: 0.875rem;
            }
        }
    </style>
</head>
<body class="bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100">
    <!-- Overlay -->
    <div class="overlay" id="overlay" onclick="closeSidebar()"></div>

    <!-- Sidebar -->
    <aside class="sidebar w-64 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700">
        <div class="px-6 py-5 border-b border-gray-200 dark:border-gray-700">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-xl font-bold bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent">
                        <i class="fas fa-cube mr-2 text-blue-600"></i>FlowCortex
                    </h1>
                    <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Blockchain Explorer</p>
                </div>
                <button onclick="closeSidebar()" class="lg:hidden p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg">
                    <i class="fas fa-times"></i>
                </button>
            </div>
        </div>

        <nav class="px-3 py-4 custom-scrollbar" style="max-height: calc(100vh - 180px); overflow-y: auto;">
            <div class="space-y-1">
                <button onclick="switchTab('dashboard')" class="tab-button active w-full text-left px-4 py-2.5 rounded-lg transition flex items-center gap-3">
                    <i class="fas fa-chart-line w-5"></i>
                    <span>Dashboard</span>
                </button>
                <button onclick="switchTab('blocks')" class="tab-button w-full text-left px-4 py-2.5 rounded-lg transition flex items-center gap-3">
                    <i class="fas fa-cube w-5"></i>
                    <span>Blocks</span>
                </button>
                <button onclick="switchTab('transactions')" class="tab-button w-full text-left px-4 py-2.5 rounded-lg transition flex items-center gap-3">
                    <i class="fas fa-exchange-alt w-5"></i>
                    <span>Transactions</span>
                </button>
                <button onclick="switchTab('balance')" class="tab-button w-full text-left px-4 py-2.5 rounded-lg transition flex items-center gap-3">
                    <i class="fas fa-wallet w-5"></i>
                    <span>Accounts</span>
                </button>
                <button onclick="switchTab('capsules')" class="tab-button w-full text-left px-4 py-2.5 rounded-lg transition flex items-center gap-3">
                    <i class="fas fa-box w-5"></i>
                    <span>Capsules</span>
                </button>
                <button onclick="switchTab('anchors')" class="tab-button w-full text-left px-4 py-2.5 rounded-lg transition flex items-center gap-3">
                    <i class="fas fa-anchor w-5"></i>
                    <span>Anchors</span>
                </button>
                <button onclick="switchTab('wallet')" class="tab-button w-full text-left px-4 py-2.5 rounded-lg transition flex items-center gap-3">
                    <i class="fas fa-key w-5"></i>
                    <span>Wallet</span>
                </button>
            </div>
        </nav>

        <div class="border-t border-gray-200 dark:border-gray-700 px-3 py-3">
            <button onclick="toggleTheme()" class="w-full px-3 py-2 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition text-sm flex items-center justify-center gap-2">
                <i class="fas fa-moon" id="theme-icon"></i>
                <span id="theme-text">Dark Mode</span>
            </button>
            <div class="mt-2 text-xs text-center">
                <div id="network-status" class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-gray-100 dark:bg-gray-700">
                    <span class="spinner w-3 h-3"></span>
                    <span>Connecting...</span>
                </div>
            </div>
        </div>
    </aside>

    <!-- Main Content -->
    <main class="main-content lg:ml-64 min-h-screen">
        <header class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 sticky top-0 z-20">
            <div class="px-4 lg:px-6 py-4">
                <div class="flex items-center justify-between gap-4">
                    <div class="flex items-center gap-3">
                        <button onclick="openSidebar()" class="lg:hidden btn-icon bg-gray-100 dark:bg-gray-700 hover:bg-gray-200">
                            <i class="fas fa-bars"></i>
                        </button>
                        <div>
                            <h2 id="pageTitle" class="text-xl lg:text-2xl font-bold">Dashboard</h2>
                            <p id="pageSubtitle" class="text-xs text-gray-500 dark:text-gray-400 hidden lg:block">Network Overview & Statistics</p>
                        </div>
                    </div>
                    <div class="flex items-center gap-2">
                        <div class="search-bar hidden md:block">
                            <input type="text" id="globalSearch" placeholder="Search blocks, txs..." class="input-field w-64" onkeypress="handleGlobalSearch(event)">
                            <i class="fas fa-search"></i>
                        </div>
                        <button onclick="refreshAll()" class="btn-icon bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 tooltip" data-tooltip="Refresh">
                            <i class="fas fa-sync-alt" id="refresh-icon"></i>
                        </button>
                    </div>
                </div>
            </div>
        </header>

        <div class="px-4 lg:px-6 py-6 custom-scrollbar">
'''

def generate_dashboard_tab():
    return '''            <!-- Dashboard Tab -->
            <div id="dashboard-content" class="tab-content space-y-6">
                <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
                    <div class="card stat-card p-4 lg:p-6" style="color: var(--primary);">
                        <div class="flex items-start justify-between">
                            <div class="flex-1">
                                <p class="text-xs lg:text-sm text-gray-500 dark:text-gray-400 mb-1">Total Blocks</p>
                                <p class="text-2xl lg:text-3xl font-bold" id="stat-blocks">0</p>
                                <p class="text-xs text-green-600 dark:text-green-400 mt-2">
                                    <i class="fas fa-arrow-up"></i>
                                    <span id="blocks-growth">+0</span>
                                </p>
                            </div>
                            <div class="w-10 h-10 lg:w-12 lg:h-12 rounded-full bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center">
                                <i class="fas fa-cube text-lg lg:text-xl"></i>
                            </div>
                        </div>
                    </div>

                    <div class="card stat-card p-4 lg:p-6" style="color: var(--secondary);">
                        <div class="flex items-start justify-between">
                            <div class="flex-1">
                                <p class="text-xs lg:text-sm text-gray-500 dark:text-gray-400 mb-1">Transactions</p>
                                <p class="text-2xl lg:text-3xl font-bold" id="stat-txs">0</p>
                                <p class="text-xs text-green-600 dark:text-green-400 mt-2">
                                    <i class="fas fa-arrow-up"></i>
                                    <span id="txs-growth">+0</span>
                                </p>
                            </div>
                            <div class="w-10 h-10 lg:w-12 lg:h-12 rounded-full bg-green-100 dark:bg-green-900/30 flex items-center justify-center">
                                <i class="fas fa-exchange-alt text-lg lg:text-xl"></i>
                            </div>
                        </div>
                    </div>

                    <div class="card stat-card p-4 lg:p-6" style="color: var(--warning);">
                        <div class="flex items-start justify-between">
                            <div class="flex-1">
                                <p class="text-xs lg:text-sm text-gray-500 dark:text-gray-400 mb-1">Pending</p>
                                <p class="text-2xl lg:text-3xl font-bold" id="stat-pending">0</p>
                                <p class="text-xs text-gray-500 dark:text-gray-400 mt-2">In mempool</p>
                            </div>
                            <div class="w-10 h-10 lg:w-12 lg:h-12 rounded-full bg-orange-100 dark:bg-orange-900/30 flex items-center justify-center">
                                <i class="fas fa-hourglass-half text-lg lg:text-xl"></i>
                            </div>
                        </div>
                    </div>

                    <div class="card stat-card p-4 lg:p-6" style="color: var(--purple);">
                        <div class="flex items-start justify-between">
                            <div class="flex-1">
                                <p class="text-xs lg:text-sm text-gray-500 dark:text-gray-400 mb-1">Capsules</p>
                                <p class="text-2xl lg:text-3xl font-bold" id="stat-capsules">0</p>
                                <p class="text-xs text-gray-500 dark:text-gray-400 mt-2">Smart contracts</p>
                            </div>
                            <div class="w-10 h-10 lg:w-12 lg:h-12 rounded-full bg-purple-100 dark:bg-purple-900/30 flex items-center justify-center">
                                <i class="fas fa-box text-lg lg:text-xl"></i>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    <div class="card p-4 lg:p-6">
                        <h3 class="text-lg font-semibold mb-4 flex items-center gap-2">
                            <i class="fas fa-chart-bar text-blue-600"></i>
                            Block Production
                        </h3>
                        <div style="position: relative; height: 250px;">
                            <canvas id="blockChart"></canvas>
                        </div>
                    </div>

                    <div class="card p-4 lg:p-6">
                        <h3 class="text-lg font-semibold mb-4 flex items-center gap-2">
                            <i class="fas fa-chart-pie text-purple-600"></i>
                            Transaction Types
                        </h3>
                        <div style="position: relative; height: 250px;">
                            <canvas id="txTypeChart"></canvas>
                        </div>
                    </div>
                </div>

                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    <div class="card p-4 lg:p-6">
                        <div class="flex items-center justify-between mb-4">
                            <h3 class="text-lg font-semibold">Recent Blocks</h3>
                            <button onclick="switchTab('blocks')" class="text-sm text-blue-600 hover:text-blue-700">
                                View all <i class="fas fa-arrow-right ml-1"></i>
                            </button>
                        </div>
                        <div id="recent-blocks" class="space-y-3">
                            <div class="empty-state">
                                <i class="fas fa-cube"></i>
                                <p>Loading...</p>
                            </div>
                        </div>
                    </div>

                    <div class="card p-4 lg:p-6">
                        <div class="flex items-center justify-between mb-4">
                            <h3 class="text-lg font-semibold">Network Info</h3>
                        </div>
                        <div class="space-y-3 text-sm">
                            <div class="flex justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
                                <span class="text-gray-600 dark:text-gray-400">State Root</span>
                                <code id="stat-root" class="text-xs">...</code>
                            </div>
                            <div class="flex justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
                                <span class="text-gray-600 dark:text-gray-400">API Endpoint</span>
                                <code class="text-xs">:3000</code>
                            </div>
                            <div class="flex justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
                                <span class="text-gray-600 dark:text-gray-400">Protocol</span>
                                <span class="badge badge-info">FlowCortex L1</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
'''

# Due to file size limits, I'll need to continue in a follow-up command
# Save the header and beginning of the file
output_path = "/workspaces/flow-cortex/explorer/templates/index-upgraded.html"
with open(output_path, 'w') as f:
    f.write(EXPLORER_HTML)
    f.write(generate_dashboard_tab())

print(f"✅ Explorer UI upgrade Part 1 generated: {output_path}")
print("⏳ Continue with Part 2 for complete tabs...")
