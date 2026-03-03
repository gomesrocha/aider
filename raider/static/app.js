document.addEventListener("DOMContentLoaded", () => {
    showView('dashboard');
});

function showView(viewId) {
    if (viewId === 'settings') loadSettings();
    document.querySelectorAll('.view').forEach(el => el.classList.add('hidden'));
    document.getElementById(viewId).classList.remove('hidden');
}

async function loadSettings() {
    try {
        const res = await fetch('/api/settings');
        if (res.ok) {
            const data = await res.json();
            if (data.openai_key) document.getElementById('openai_key').value = data.openai_key;
            if (data.anthropic_key) document.getElementById('anthropic_key').value = data.anthropic_key;
            if (data.gemini_key) document.getElementById('gemini_key').value = data.gemini_key;
        }
    } catch (e) {
        console.error("Failed to load settings", e);
    }
}

async function saveSettings() {
    const payload = {
        openai_key: document.getElementById('openai_key').value || null,
        anthropic_key: document.getElementById('anthropic_key').value || null,
        gemini_key: document.getElementById('gemini_key').value || null,
    };

    try {
        const res = await fetch('/api/settings', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });
        if (res.ok) {
            const msg = document.getElementById('settings-msg');
            msg.classList.remove('hidden');
            setTimeout(() => msg.classList.add('hidden'), 3000);
        } else {
            alert("Failed to save settings");
        }
    } catch (e) {
        console.error(e);
        alert("Network error while saving settings");
    }
}

let activeTaskId = null;
let pollInterval = null;

async function startAgent() {
    const targetDir = document.getElementById('target_dir').value;
    const filesInput = document.getElementById('target_files').value;
    const provider = document.getElementById('provider').value;
    const modelName = document.getElementById('model_name').value;
    const prompt = document.getElementById('prompt').value;

    if (!targetDir || !prompt) {
        alert("Please fill in Target Directory and Prompt");
        return;
    }

    const targetFiles = filesInput ? filesInput.split(',').map(s => s.trim()).filter(Boolean) : null;

    const payload = {
        target_dir: targetDir,
        prompt: prompt,
        provider: provider,
        model_name: modelName,
        target_files: targetFiles,
    };

    document.getElementById('start-btn').disabled = true;
    document.getElementById('start-btn').innerText = "Starting...";
    document.getElementById('status-panel').classList.remove('hidden');
    document.getElementById('logs-container').innerHTML = '';

    try {
        const res = await fetch('/api/agents', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });

        if (res.ok) {
            const data = await res.json();
            activeTaskId = data.task_id;
            startPolling();
        } else {
            alert("Error starting task");
        }
    } catch (e) {
        alert("Network error");
    } finally {
        document.getElementById('start-btn').disabled = false;
        document.getElementById('start-btn').innerText = "Start Agent";
    }
}

function startPolling() {
    if (pollInterval) clearInterval(pollInterval);
    pollInterval = setInterval(pollTask, 1000);
}

async function pollTask() {
    if (!activeTaskId) return;

    try {
        const res = await fetch(`/api/agents/${activeTaskId}`);
        if (res.ok) {
            const data = await res.json();

            // Render Status
            let statusText = "";
            if (typeof data.status === 'string') {
                statusText = data.status;
            } else if (data.status.Error) {
                statusText = `Error: ${data.status.Error}`;
            }

            document.getElementById('task-state').innerText = statusText;

            // Render Logs
            const logsContainer = document.getElementById('logs-container');
            logsContainer.innerHTML = data.logs.map(log => `<div>> ${log}</div>`).join('');
            logsContainer.scrollTop = logsContainer.scrollHeight;

            if (statusText === 'Done' || statusText.startsWith('Error')) {
                clearInterval(pollInterval);
                pollInterval = null;
            }
        }
    } catch (e) {
        console.error("Polling error", e);
    }
}
