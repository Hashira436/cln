/**
 * Quick Fixes panel — temp cleanup, disk health, event log viewer.
 */
import { ipc } from '../ipc.js';

/**
 * @param {{ showToast: Function, escapeHtml: Function }} deps
 */
export function initQuickFixesPanel({ showToast, escapeHtml }) {
    const tempSizeEl = document.getElementById('temp-size-label');
    const tempPathsEl = document.getElementById('temp-paths-label');
    const cleanBtn = document.getElementById('btn-clean-temp');
    const refreshBtn = document.getElementById('btn-refresh-fixes');
    const diskCardsEl = document.getElementById('disk-health-cards');
    const smartListEl = document.getElementById('smart-status-list');
    const eventsBodyEl = document.getElementById('event-log-body');
    const fixesStatusDot = document.getElementById('fixes-status-dot');
    const fixesStatusText = document.getElementById('fixes-status-text');

    let isLoading = false;

    function setStatus(mode, text) {
        fixesStatusDot.className = `status-dot ${mode}`;
        fixesStatusText.textContent = text;
    }

    function formatBytes(bytes) {
        if (bytes >= 1_073_741_824) {
            return `${(bytes / 1_073_741_824).toFixed(2)} GB`;
        }
        if (bytes >= 1_048_576) {
            return `${(bytes / 1_048_576).toFixed(1)} MB`;
        }
        if (bytes >= 1024) {
            return `${(bytes / 1024).toFixed(1)} KB`;
        }
        return `${bytes} B`;
    }

    function formatTimestamp(value) {
        const parsed = Date.parse(value);
        if (Number.isNaN(parsed)) {
            return value;
        }
        return new Date(parsed).toLocaleString();
    }

    function smartBadgeClass(status) {
        const normalized = status.toLowerCase();
        if (normalized.includes('ok')) return 'badge-ok';
        if (normalized.includes('pred') || normalized.includes('fail')) return 'badge-danger';
        return 'badge-muted';
    }

    function renderDiskHealth(report) {
        if (!report.volumes.length) {
            diskCardsEl.innerHTML = `
                <div class="empty-state compact">
                    <div class="empty-text">No mounted volumes detected</div>
                </div>
            `;
        } else {
            diskCardsEl.innerHTML = report.volumes
                .map((volume) => {
                    const usedPercent = Math.min(100, Math.max(0, volume.used_percent));
                    const level =
                        usedPercent >= 90 ? 'high' : usedPercent >= 75 ? 'medium' : 'normal';

                    return `
                        <article class="metric-card">
                            <div class="metric-card-header">
                                <span class="metric-title">${escapeHtml(volume.mount_point)}</span>
                                <span class="metric-subtitle">${escapeHtml(volume.name)}</span>
                            </div>
                            <div class="metric-value">${formatBytes(volume.free_bytes)} free</div>
                            <div class="metric-meta">
                                ${formatBytes(volume.total_bytes)} total · ${usedPercent.toFixed(1)}% used
                            </div>
                            <div class="memory-bar metric-bar">
                                <div class="memory-bar-fill level-${level}" style="width: ${usedPercent}%"></div>
                            </div>
                        </article>
                    `;
                })
                .join('');
        }

        if (!report.physical_disks.length) {
            smartListEl.innerHTML = `
                <li class="smart-item">
                    <span class="badge badge-muted">Unknown</span>
                    <span>SMART data unavailable (WMI query returned no disks)</span>
                </li>
            `;
            return;
        }

        smartListEl.innerHTML = report.physical_disks
            .map(
                (disk) => `
                <li class="smart-item">
                    <span class="badge ${smartBadgeClass(disk.smart_status)}">${escapeHtml(disk.smart_status)}</span>
                    <span>
                        <strong>${escapeHtml(disk.model)}</strong>
                        · ${formatBytes(disk.size_bytes)}
                    </span>
                </li>
            `
            )
            .join('');
    }

    function renderEvents(events) {
        if (!events.length) {
            eventsBodyEl.innerHTML = `
                <tr>
                    <td colspan="5">
                        <div class="empty-state compact">
                            <div class="empty-text">No critical or error events in the last 24 hours</div>
                        </div>
                    </td>
                </tr>
            `;
            return;
        }

        eventsBodyEl.innerHTML = events
            .map(
                (event) => `
                <tr>
                    <td class="cell-time">${escapeHtml(formatTimestamp(event.timestamp))}</td>
                    <td><span class="badge ${event.level === 'Critical' ? 'badge-danger' : 'badge-warning'}">${escapeHtml(event.level)}</span></td>
                    <td class="cell-name">${escapeHtml(event.channel)}</td>
                    <td class="cell-name">${escapeHtml(event.provider)}</td>
                    <td class="cell-message" title="${escapeHtml(event.message)}">${escapeHtml(event.message)}</td>
                </tr>
            `
            )
            .join('');
    }

    async function loadTempStats() {
        const stats = await ipc.getTempFolderStats();
        tempSizeEl.textContent = `${formatBytes(stats.total_bytes)} in ${stats.file_count} files`;
        tempPathsEl.textContent = stats.paths_scanned.join(' · ');
        return stats;
    }

    async function refreshPanel() {
        if (isLoading) return;
        isLoading = true;
        setStatus('loading', 'Refreshing quick fixes…');

        try {
            const [_, diskHealth, events] = await Promise.all([
                loadTempStats(),
                ipc.getDiskHealth(),
                ipc.getCriticalEvents(),
            ]);

            renderDiskHealth(diskHealth);
            renderEvents(events);
            setStatus('online', `Updated at ${new Date().toLocaleTimeString()}`);
        } catch (err) {
            setStatus('error', `Error: ${err}`);
            showToast(`Quick fixes refresh failed: ${err}`, 'error');
            console.error(err);
        } finally {
            isLoading = false;
        }
    }

    cleanBtn.addEventListener('click', async () => {
        if (!confirm('Delete temp files from user TEMP, Windows\\Temp, and SoftwareDistribution\\Download?')) {
            return;
        }

        try {
            cleanBtn.disabled = true;
            cleanBtn.textContent = 'Cleaning…';

            const result = await ipc.cleanTempFiles();
            await loadTempStats();

            showToast(
                `Freed ${formatBytes(result.bytes_freed)} (${result.files_deleted} deleted, ${result.files_skipped} skipped)`,
                'success',
                5000
            );
        } catch (err) {
            showToast(`Temp cleanup failed: ${err}`, 'error');
        } finally {
            cleanBtn.disabled = false;
            cleanBtn.textContent = 'Clean Temp Files';
        }
    });

    refreshBtn.addEventListener('click', () => refreshPanel());

    refreshPanel();
}
