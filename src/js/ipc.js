/**
 * IPC layer — thin wrapper around Tauri invoke for backend commands.
 */
const { invoke } = window.__TAURI__.core;

export const ipc = {
    getTopProcesses(count = 5, sortBy = 'memory') {
        return invoke('get_top_processes', { count, sort_by: sortBy });
    },

    killProcess(pid) {
        return invoke('kill_process', { pid });
    },

    getTempFolderStats() {
        return invoke('get_temp_folder_stats');
    },

    cleanTempFiles() {
        return invoke('clean_temp_files');
    },

    getDiskHealth() {
        return invoke('get_disk_health');
    },

    getCriticalEvents() {
        return invoke('get_critical_events');
    },

    /** Phase 3 scaffold — triggers streaming scan-result events. */
    searchFiles(query) {
        return invoke('search_files', { query });
    },
};
