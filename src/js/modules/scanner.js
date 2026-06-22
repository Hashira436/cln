const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

class TreeNode {
    constructor(name, fullPath) {
        this.name = name;
        this.fullPath = fullPath;
        this.children = {};
        this.resultData = null; // ScanResult if this matched
        this.isExpanded = true;
    }
}

let unlistenScanResult = null;
let currentSearchActive = false;
let rootNodes = {};
let nodeMap = new Map();
let renderTimeout = null;

export function initScannerPanel({ showToast, escapeHtml }) {
    const btnSearch = document.getElementById('btn-search-files');
    const inputPath = document.getElementById('scanner-path');
    const inputQuery = document.getElementById('scanner-query');
    const tableBody = document.getElementById('scanner-table-body');
    const statusDot = document.getElementById('scanner-status-dot');
    const statusText = document.getElementById('scanner-status-text');

    function insertPath(result) {
        const parts = result.path.split(/[\\/]/).filter(Boolean);
        if (parts.length === 0) return;

        let currentPath = parts[0];
        let currentNode = rootNodes[currentPath];
        if (!currentNode) {
            currentNode = new TreeNode(parts[0], currentPath);
            rootNodes[currentPath] = currentNode;
            nodeMap.set(currentPath, currentNode);
        }

        for (let i = 1; i < parts.length; i++) {
            const part = parts[i];
            currentPath += '\\' + part;
            if (!currentNode.children[part]) {
                const childNode = new TreeNode(part, currentPath);
                currentNode.children[part] = childNode;
                nodeMap.set(currentPath, childNode);
            }
            currentNode = currentNode.children[part];
        }
        currentNode.resultData = result;
    }

    function renderTreeNow() {
        const fragment = document.createDocumentFragment();

        function traverse(node, depth) {
            const tr = document.createElement('tr');
            tr.className = 'tree-row';

            const hasChildren = Object.keys(node.children).length > 0;
            const toggleClass = hasChildren ? (node.isExpanded ? 'tree-toggle expanded' : 'tree-toggle') : 'tree-toggle hidden';

            const isArtificial = !node.resultData;
            const labelClass = isArtificial ? 'tree-label artificial' : 'tree-label';

            const formatSize = (bytes) => {
                if (bytes === 0) return '-';
                const mb = bytes / (1024 * 1024);
                return mb < 1 ? '<1 MB' : `${mb.toFixed(1)} MB`;
            };

            const fileType = node.resultData ? node.resultData.file_type : 'Directory';
            const sizeStr = node.resultData ? formatSize(node.resultData.size_bytes) : '-';
            const reasonStr = node.resultData ? escapeHtml(node.resultData.associated_reason) : '(Parent directory)';

            const nukeBtnHtml = fileType === 'Directory' && node.resultData
                ? `<button class="btn btn-danger btn-nuke" data-path="${escapeHtml(node.fullPath)}" style="padding: 0.25rem 0.5rem; font-size: 0.8rem;">Nuke</button>`
                : '';

            tr.innerHTML = `
                <td class="col-path" style="padding-left: ${depth * 20 + 10}px">
                    <div class="tree-cell-path">
                        <span class="${toggleClass}" data-path="${escapeHtml(node.fullPath)}">▶</span>
                        <span class="${labelClass}" title="${escapeHtml(node.fullPath)}">${escapeHtml(node.name)}</span>
                    </div>
                </td>
                <td class="col-type">${escapeHtml(fileType)}</td>
                <td class="col-size">${sizeStr}</td>
                <td class="col-reason">${reasonStr}</td>
                <td class="col-action">${nukeBtnHtml}</td>
            `;
            fragment.appendChild(tr);

            if (node.isExpanded && hasChildren) {
                // Directories first, then alphabetical
                const childNodes = Object.values(node.children);
                childNodes.sort((a, b) => {
                    const aIsDir = a.resultData ? a.resultData.file_type === 'Directory' : true;
                    const bIsDir = b.resultData ? b.resultData.file_type === 'Directory' : true;
                    if (aIsDir && !bIsDir) return -1;
                    if (!aIsDir && bIsDir) return 1;
                    return a.name.localeCompare(b.name);
                });

                childNodes.forEach(child => traverse(child, depth + 1));
            }
        }

        const roots = Object.values(rootNodes);
        roots.sort((a, b) => a.name.localeCompare(b.name));
        roots.forEach(rootNode => traverse(rootNode, 0));

        // Use rAF to append without freezing the thread
        requestAnimationFrame(() => {
            tableBody.innerHTML = '';
            tableBody.appendChild(fragment);
        });
    }

    function debouncedRender() {
        if (renderTimeout) clearTimeout(renderTimeout);
        renderTimeout = setTimeout(renderTreeNow, 150);
    }

    async function handleNuke(path) {
        const confirmed = confirm(`Are you sure you want to nuke this directory?\n\n${path}\n\nThis bypasses the Recycle Bin and cannot be undone.`);
        if (!confirmed) return;

        try {
            statusDot.classList.add('loading');
            statusText.textContent = `Nuking ${path}...`;
            await invoke('nuke_directory', { path });
            showToast(`Successfully nuked ${path}`, 'success');
        } catch (error) {
            console.error('Nuke failed:', error);
            showToast(`Failed to nuke: ${error}`, 'error');
        } finally {
            statusDot.classList.remove('loading');
            statusText.textContent = 'Ready';
        }
    }

    async function startSearch() {
        if (currentSearchActive) {
            showToast('A search is already running.', 'warning');
            return;
        }

        const path = inputPath.value.trim();
        const query = inputQuery.value.trim();

        if (!path || !query) {
            showToast('Please enter both a path and a search query.', 'warning');
            return;
        }

        currentSearchActive = true;
        rootNodes = {};
        nodeMap.clear();
        tableBody.innerHTML = '<tr><td colspan="5" class="text-center" style="padding: 2rem;">Searching...</td></tr>';
        
        statusDot.classList.add('loading');
        statusText.textContent = `Scanning in ${path}...`;
        btnSearch.disabled = true;

        if (unlistenScanResult) {
            unlistenScanResult();
        }

        unlistenScanResult = await listen('scan-result', (event) => {
            const batch = event.payload; 
            if (!batch || !batch.length) return;

            batch.forEach(result => insertPath(result));
            debouncedRender();
        });

        try {
            await invoke('search_files', { path, query });
            statusText.textContent = 'Scan completed.';
        } catch (error) {
            console.error('Scan error:', error);
            statusText.textContent = 'Scan failed.';
            showToast(error, 'error');
        } finally {
            currentSearchActive = false;
            statusDot.classList.remove('loading');
            btnSearch.disabled = false;
            // One final render to ensure all nodes are shown
            debouncedRender();
        }
    }

    btnSearch.addEventListener('click', startSearch);
    inputQuery.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') startSearch();
    });

    tableBody.addEventListener('click', (e) => {
        if (e.target.classList.contains('tree-toggle')) {
            const path = e.target.dataset.path;
            const node = nodeMap.get(path);
            if (node) {
                node.isExpanded = !node.isExpanded;
                renderTreeNow(); // Immediate render for user interaction
            }
        } else if (e.target.classList.contains('btn-nuke')) {
            const path = e.target.dataset.path;
            handleNuke(path);
        }
    });
}
