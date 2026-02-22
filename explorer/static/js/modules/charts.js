/**
 * Charts Module - Handles data visualization with Chart.js
 */

let blockChart = null;
let txTypeChart = null;

/**
 * Get theme-aware colors
 */
function getChartColors() {
    const isDark = document.documentElement.classList.contains('dark');
    return {
        text: isDark ? '#e5e7eb' : '#374151',
        textSecondary: isDark ? '#9ca3af' : '#6b7280',
        grid: isDark ? '#374151' : '#f3f4f6',
        primary: '#3b82f6',
        secondary: '#10b981',
        purple: '#8b5cf6',
        gray: '#6b7280'
    };
}

/**
 * Initialize block production chart
 */
export function initBlockChart(canvasId) {
    const ctx = document.getElementById(canvasId);
    if (!ctx) return null;
    
    const colors = getChartColors();
    
    blockChart = new Chart(ctx, {
        type: 'bar',
        data: {
            labels: [],
            datasets: [{
                label: 'Transactions per Block',
                data: [],
                backgroundColor: 'rgba(59, 130, 246, 0.5)',
                borderColor: colors.primary,
                borderWidth: 2,
                borderRadius: 6
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    display: false
                },
                tooltip: {
                    backgroundColor: 'rgba(17, 24, 39, 0.95)',
                    padding: 12,
                    titleColor: '#fff',
                    bodyColor: '#fff',
                    borderColor: colors.primary,
                    borderWidth: 1
                }
            },
            scales: {
                y: {
                    beginAtZero: true,
                    ticks: {
                        color: colors.textSecondary,
                        precision: 0
                    },
                    grid: {
                        color: colors.grid
                    }
                },
                x: {
                    ticks: {
                        color: colors.textSecondary
                    },
                    grid: {
                        display: false
                    }
                }
            }
        }
    });
    
    return blockChart;
}

/**
 * Update block chart with new data
 */
export function updateBlockChart(blocks) {
    if (!blockChart) return;
    
    const last10 = blocks.slice(-10);
    const labels = last10.map(b => `#${b.height}`);
    const data = last10.map(b => {
        try {
            return JSON.parse(b.txs_json || '[]').length;
        } catch {
            return 0;
        }
    });
    
    blockChart.data.labels = labels;
    blockChart.data.datasets[0].data = data;
    blockChart.update('none'); // No animation for updates
}

/**
 * Initialize transaction type pie chart
 */
export function initTxTypeChart(canvasId) {
    const ctx = document.getElementById(canvasId);
    if (!ctx) return null;
    
    const colors = getChartColors();
    
    txTypeChart = new Chart(ctx, {
        type: 'doughnut',
        data: {
            labels: ['Transfer', 'Mint', 'AnchorProof', 'Other'],
            datasets: [{
                data: [0, 0, 0, 0],
                backgroundColor: [
                    'rgba(59, 130, 246, 0.8)',
                    'rgba(16, 185, 129, 0.8)',
                    'rgba(139, 92, 246, 0.8)',
                    'rgba(107, 114, 128, 0.8)'
                ],
                borderWidth: 0
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    position: 'bottom',
                    labels: {
                        color: colors.text,
                        padding: 15,
                        usePointStyle: true,
                        pointStyle: 'circle'
                    }
                },
                tooltip: {
                    backgroundColor: 'rgba(17, 24, 39, 0.95)',
                    padding: 12,
                    titleColor: '#fff',
                    bodyColor: '#fff'
                }
            }
        }
    });
    
    return txTypeChart;
}

/**
 * Update transaction type chart
 */
export function updateTxTypeChart(blocks) {
    if (!txTypeChart) return;
    
    const typeCounts = {
        Transfer: 0,
        Mint: 0,
        AnchorProof: 0,
        Other: 0
    };
    
    blocks.forEach(block => {
        try {
            const txs = JSON.parse(block.txs_json || '[]');
            txs.forEach(tx => {
                const kind = tx.kind ? Object.keys(tx.kind)[0] : 'Other';
                if (typeCounts[kind] !== undefined) {
                    typeCounts[kind]++;
                } else {
                    typeCounts.Other++;
                }
            });
        } catch (error) {
            console.error('Error parsing transactions:', error);
        }
    });
    
    txTypeChart.data.datasets[0].data = Object.values(typeCounts);
    txTypeChart.update('none');
}

/**
 * Update chart theme when dark mode toggles
 */
export function updateChartTheme() {
    const colors = getChartColors();
    
    if (blockChart) {
        blockChart.options.scales.x.ticks.color = colors.textSecondary;
        blockChart.options.scales.y.ticks.color = colors.textSecondary;
        blockChart.options.scales.x.grid.color = colors.grid;
        blockChart.options.scales.y.grid.color = colors.grid;
        blockChart.update('none');
    }
    
    if (txTypeChart) {
        txTypeChart.options.plugins.legend.labels.color = colors.text;
        txTypeChart.update('none');
    }
}

/**
 * Destroy charts on cleanup
 */
export function destroyCharts() {
    if (blockChart) {
        blockChart.destroy();
        blockChart = null;
    }
    if (txTypeChart) {
        txTypeChart.destroy();
        txTypeChart = null;
    }
}
