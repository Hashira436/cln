/**
 * Processes panel — polls backend, renders table, handles Kill via delegation.
 */
import { ipc } from '../ipc.js';

const POLL_INTERVAL_MS = 3000;
const TOP_COUNT = 5;

/**
 * @param {{ showToast: (msg: string, type?: string) => void, escapeHtml: (s: string) => string }} deps
 */
export function initProcessesPanel({ showToast, escapeHtml }) {
    const processTableBody = document.getElementById('process-table-body');
    const refreshBtn = document.getElementById('btn-refresh-processes');
    const statusDot = document.getElementById('process-status-dot');
    const statusText = document.getElementById('process-status-text');
    const sortToggle = document.getElementById('sort-toggle');

    let currentSortBy = 'memory';
    let autoRefreshInterval = null;
    let isLoadingProcesses = false;

    sortToggle.addEventListener('click', (e) => {
        const option = e.target.closest('.sort-option');
        if (!option) return;

        currentSortBy = option.dataset.sort;
        sortToggle.querySelectorAll('.sort-option').forEach((o) => o.classList.remove('active'));
        option.classList.add('active');
        loadProcesses();
    });

    function getMemoryLevel(mb) {
        if (mb >= 1000) return 'high';
        if (mb >= 300) return 'medium';
        return 'normal';
    }

    function getCpuLevel(percent) {
        if (percent >= 50) return 'high';
        if (percent >= 15) return 'medium';
        return 'normal';
    }

    function getRankClass(rank) {
        if (rank === 1) return 'rank-1';
        if (rank === 2) return 'rank-2';
        if (rank === 3) return 'rank-3';
        return 'rank-default';
    }

    function memoryBarPercent(mb, maxMb) {
        if (maxMb === 0) return 0;
        return Math.min(100, Math.round((mb / maxMb) * 100));
    }

    function renderProcesses(processes) {
        if (processes.length === 0) {
            processTableBody.innerHTML = `
                <tr>
                    <td colspan="6">
                        <div class="empty-state">
                            <div class="empty-icon">⚙</div>
                            <div class="empty-text">No processes found</div>
                            <div class="empty-hint">Try running as Administrator if the list stays empty.</div>
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
                const safeName = escapeHtml(p.name);

                return `
                    <tr>
                        <td class="cell-rank">
                            <span class="rank-badge ${rankClass}">${rank}</span>
                        </td>
                        <td class="cell-name" title="${safeName}">${safeName}</td>
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
                                data-name="${safeName}"
                                title="Terminate ${safeName}"
                            >
                                Kill
                            </button>
                        </td>
                    </tr>
                `;
            })
            .join('');
    }

    async function loadProcesses() {
        if (isLoadingProcesses) return;
        isLoadingProcesses = true;

        try {
            statusDot.className = 'status-dot loading';
            statusText.textContent = 'Refreshing…';

            const processes = await ipc.getTopProcesses(TOP_COUNT, currentSortBy);
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

    processTableBody.addEventListener('click', async (e) => {
        const btn = e.target.closest('.btn-kill');
        if (!btn) return;

        const pid = Number(btn.dataset.pid);
        const name = btn.dataset.name;

        if (!confirm(`Terminate "${name}" (PID ${pid})?`)) return;

        try {
            btn.disabled = true;
            btn.textContent = '…';

            const msg = await ipc.killProcess(pid);
            showToast(msg, 'success');
            await loadProcesses();
        } catch (err) {
            showToast(`Failed to kill ${name}: ${err}`, 'error');
            btn.disabled = false;
            btn.textContent = 'Kill';
        }
    });

    refreshBtn.addEventListener('click', () => loadProcesses());

    function startAutoRefresh() {
        stopAutoRefresh();
        autoRefreshInterval = setInterval(loadProcesses, POLL_INTERVAL_MS);
    }

    function stopAutoRefresh() {
        if (autoRefreshInterval) {
            clearInterval(autoRefreshInterval);
            autoRefreshInterval = null;
        }
    }

    document.addEventListener('visibilitychange', () => {
        if (document.hidden) {
            stopAutoRefresh();
        } else {
            loadProcesses();
            startAutoRefresh();
        }
    });

    loadProcesses();
    startAutoRefresh();
}
