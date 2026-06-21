// ============================================================================
// CLN — Main Application Script
// Handles: tab navigation, Tauri IPC, process monitoring, toast notifications
// ============================================================================

// ---------------------------------------------------------------------------
// IPC Layer — wraps Tauri's invoke() for type-safe calls to Rust backend
// ---------------------------------------------------------------------------
const { invoke } = window.__TAURI__.core;

const ipc = {
    /**
     * Fetch top N processes sorted by the given criteria.
     * @param {number} count - Number of processes to return (default: 5)
     * @param {string} sortBy - "memory" or "cpu"
     * @returns {Promise<Array<{pid: number, name: string, memory_mb: number, cpu_percent: number}>>}
     */
    getTopProcesses(count = 5, sortBy = 'memory') {
        return invoke('get_top_processes', { count, sortBy });
    },

    /**
     * Kill a process by its PID.
     * @param {number} pid
     * @returns {Promise<string>} Success message
     */
    killProcess(pid) {
        return invoke('kill_process', { pid });
    },
};

// ---------------------------------------------------------------------------
// Toast Notification System
// ---------------------------------------------------------------------------
const toastContainer = document.getElementById('toast-container');

function showToast(message, type = 'info', duration = 3500) {
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;

    const icons = { success: '✓', error: '✗', info: 'ℹ', warning: '⚠' };
    toast.innerHTML = `<span>${icons[type] || 'ℹ'}</span><span>${escapeHtml(message)}</span>`;

    toastContainer.appendChild(toast);

    setTimeout(() => {
        toast.classList.add('toast-exit');
        setTimeout(() => toast.remove(), 300);
    }, duration);
}

function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}

// ---------------------------------------------------------------------------
// Tab Navigation (Sidebar Router)
// ---------------------------------------------------------------------------
const navButtons = document.querySelectorAll('.nav-btn[data-panel]');
const panels = document.querySelectorAll('.panel');

navButtons.forEach((btn) => {
    btn.addEventListener('click', () => {
        const targetPanel = btn.dataset.panel;

        // Deactivate all
        navButtons.forEach((b) => b.classList.remove('active'));
        panels.forEach((p) => p.classList.remove('active'));

        // Activate selected
        btn.classList.add('active');
        const panel = document.getElementById(`panel-${targetPanel}`);
        if (panel) panel.classList.add('active');
    });
});

// ---------------------------------------------------------------------------
// Process Module — Table rendering, sorting, kill functionality
// ---------------------------------------------------------------------------
const processTableBody = document.getElementById('process-table-body');
const refreshBtn = document.getElementById('btn-refresh-processes');
const statusDot = document.getElementById('process-status-dot');
const statusText = document.getElementById('process-status-text');
const sortToggle = document.getElementById('sort-toggle');

let currentSortBy = 'memory';
let autoRefreshInterval = null;
let isLoadingProcesses = false;

// Sort toggle handler
sortToggle.addEventListener('click', (e) => {
    const option = e.target.closest('.sort-option');
    if (!option) return;

    currentSortBy = option.dataset.sort;

    // Update active state
    sortToggle.querySelectorAll('.sort-option').forEach((o) => o.classList.remove('active'));
    option.classList.add('active');

    // Reload with new sort
    loadProcesses();
});

/**
 * Returns a CSS class for memory usage level thresholding.
 */
function getMemoryLevel(mb) {
    if (mb >= 1000) return 'high';
    if (mb >= 300) return 'medium';
    return 'normal';
}

/**
 * Returns a CSS class for CPU usage level thresholding.
 */
function getCpuLevel(percent) {
    if (percent >= 50) return 'high';
    if (percent >= 15) return 'medium';
    return 'normal';
}

/**
 * Returns the rank badge CSS class.
 */
function getRankClass(rank) {
    if (rank === 1) return 'rank-1';
    if (rank === 2) return 'rank-2';
    if (rank === 3) return 'rank-3';
    return 'rank-default';
}

/**
 * Calculate the memory bar percentage (0–100) relative to the top process.
 */
function memoryBarPercent(mb, maxMb) {
    if (maxMb === 0) return 0;
    return Math.min(100, Math.round((mb / maxMb) * 100));
}

/**
 * Renders the process table rows.
 * @param {Array} processes
 */
function renderProcesses(processes) {
    if (processes.length === 0) {
        processTableBody.innerHTML = `
            <tr>
                <td colspan="6">
                    <div class="empty-state">
                        <div class="empty-icon">⚙</div>
                        <div class="empty-text">No processes found</div>
                        <div class="empty-hint">This is unexpected. Try running as Administrator.</div>
                    </div>
                </td>
            </tr>
        `;
        return;
    }

    const maxMb = processes[0]?.memory_mb || 1;

    processTableBody.innerHTML = processes
        .map((p, i) => {
            const rank = i + 1;
            const memLevel = getMemoryLevel(p.memory_mb);
            const cpuLevel = getCpuLevel(p.cpu_percent);
            const barPercent = memoryBarPercent(p.memory_mb, maxMb);
            const rankClass = getRankClass(rank);

            return `
                <tr>
                    <td class="cell-rank">
                        <span class="rank-badge ${rankClass}">${rank}</span>
                    </td>
                    <td class="cell-name" title="${escapeHtml(p.name)}">${escapeHtml(p.name)}</td>
                    <td class="cell-pid">${p.pid}</td>
                    <td class="cell-memory">
                        <div class="memory-bar-wrapper">
                            <span class="memory-${memLevel}">${p.memory_mb.toFixed(1)} MB</span>
                            <div class="memory-bar">
                                <div class="memory-bar-fill level-${memLevel}" style="width: ${barPercent}%"></div>
                            </div>
                        </div>
                    </td>
                    <td class="cell-cpu cpu-${cpuLevel}">${p.cpu_percent.toFixed(1)}%</td>
                    <td class="text-center">
                        <button
                            class="btn btn-danger btn-sm btn-kill"
                            data-pid="${p.pid}"
                            data-name="${escapeHtml(p.name)}"
                            title="Terminate ${escapeHtml(p.name)}"
                        >
                            Kill
                        </button>
                    </td>
                </tr>
            `;
        })
        .join('');
}

/**
 * Loads and displays the top processes from the Rust backend.
 */
async function loadProcesses() {
    if (isLoadingProcesses) return;
    isLoadingProcesses = true;

    try {
        statusDot.className = 'status-dot loading';
        statusText.textContent = 'Refreshing…';

        const processes = await ipc.getTopProcesses(5, currentSortBy);
        renderProcesses(processes);

        statusDot.className = 'status-dot online';
        statusText.textContent = `Updated at ${new Date().toLocaleTimeString()} · Sorted by ${currentSortBy === 'cpu' ? 'CPU' : 'RAM'} · ${processes.length} processes`;
    } catch (err) {
        statusDot.className = 'status-dot error';
        statusText.textContent = `Error: ${err}`;
        showToast(`Failed to load processes: ${err}`, 'error');
        console.error('Failed to load processes:', err);
    } finally {
        isLoadingProcesses = false;
    }
}

/**
 * Handles kill button clicks via event delegation on the table body.
 */
processTableBody.addEventListener('click', async (e) => {
    const btn = e.target.closest('.btn-kill');
    if (!btn) return;

    const pid = Number(btn.dataset.pid);
    const name = btn.dataset.name;

    // Confirmation
    const confirmed = confirm(`Terminate "${name}" (PID ${pid})?`);
    if (!confirmed) return;

    try {
        btn.disabled = true;
        btn.textContent = '…';

        const msg = await ipc.killProcess(pid);
        showToast(msg, 'success');

        // Refresh the list after killing
        await loadProcesses();
    } catch (err) {
        showToast(`Failed to kill ${name}: ${err}`, 'error');
        btn.disabled = false;
        btn.textContent = 'Kill';
    }
});

// Refresh button
refreshBtn.addEventListener('click', () => loadProcesses());

// ---------------------------------------------------------------------------
// Auto-refresh: update process list every 3 seconds
// ---------------------------------------------------------------------------
function startAutoRefresh() {
    stopAutoRefresh();
    autoRefreshInterval = setInterval(loadProcesses, 3000);
}

function stopAutoRefresh() {
    if (autoRefreshInterval) {
        clearInterval(autoRefreshInterval);
        autoRefreshInterval = null;
    }
}

// Pause auto-refresh when the tab is not visible (performance optimization)
document.addEventListener('visibilitychange', () => {
    if (document.hidden) {
        stopAutoRefresh();
    } else {
        loadProcesses();
        startAutoRefresh();
    }
});

// ---------------------------------------------------------------------------
// Boot
// ---------------------------------------------------------------------------
(async function init() {
    // Initial load
    await loadProcesses();
    startAutoRefresh();
})();
