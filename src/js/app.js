/**
 * Application shell — tab navigation, toasts, module bootstrap.
 */
import { initProcessesPanel } from './modules/processes.js';
import { initQuickFixesPanel } from './modules/quick-fixes.js';
import { initScannerPanel } from './modules/scanner.js';

const toastContainer = document.getElementById('toast-container');

export function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}

export function showToast(message, type = 'info', duration = 3500) {
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

function initTabNavigation() {
    const navButtons = document.querySelectorAll('.nav-btn[data-panel]');
    const panels = document.querySelectorAll('.panel');

    navButtons.forEach((btn) => {
        btn.addEventListener('click', () => {
            const targetPanel = btn.dataset.panel;

            navButtons.forEach((b) => b.classList.remove('active'));
            panels.forEach((p) => p.classList.remove('active'));

            btn.classList.add('active');
            const panel = document.getElementById(`panel-${targetPanel}`);
            if (panel) panel.classList.add('active');
        });
    });
}

function boot() {
    initTabNavigation();
    initProcessesPanel({ showToast, escapeHtml });
    initQuickFixesPanel({ showToast, escapeHtml });
    initScannerPanel({ showToast, escapeHtml });
}

boot();
