// chat.js: SSE stream parsing and message rendering

class ChatManager {
    constructor() {
        this.messagesContainer = document.getElementById('chat-messages');
        this.input = document.getElementById('chat-input');
        this.sendBtn = document.getElementById('send-btn');
        this.conversationId = null;
        this.isWaiting = false;
    }

    async sendMessage(message) {
        if (!message.trim() || this.isWaiting) return;

        this.isWaiting = true;
        this.sendBtn.disabled = true;
        this.input.disabled = true;

        const userEl = this.createMessageElement('user', message);
        this.messagesContainer.appendChild(userEl);
        this.input.value = '';
        this.autoScroll();

        const assistantEl = this.createMessageElement('assistant', '');
        this.messagesContainer.appendChild(assistantEl);
        const contentDiv = assistantEl.querySelector('.message-content');
        this._textContent = '';

        try {
            const response = await fetch('/api/chat', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ message, conversation_id: this.conversationId }),
            });

            if (!response.ok) {
                contentDiv.innerHTML = '<p style="color: red;">Error: ' + response.status + '</p>';
                this.unlockInput();
                return;
            }

            const reader = response.body.getReader();
            const decoder = new TextDecoder();
            let buffer = '';
            let currentEvent = 'message';

            while (true) {
                const { value, done } = await reader.read();
                if (done) break;

                buffer += decoder.decode(value, { stream: true });
                const lines = buffer.split('\n');
                buffer = lines.pop() || '';

                for (const line of lines) {
                    if (line.startsWith('event: ')) {
                        currentEvent = line.slice(7).trim();
                    } else if (line.startsWith('data: ')) {
                        const jsonData = line.slice(6).trim();
                        this.handleSseEvent(currentEvent, jsonData, contentDiv);
                    } else if (line === '') {
                        currentEvent = 'message';
                    }
                }
            }
        } catch (err) {
            contentDiv.innerHTML = '<p style="color: red;">Error: ' + err.message + '</p>';
            this.unlockInput();
        }
    }

    handleSseEvent(eventType, jsonData, contentDiv) {
        if (eventType === 'chunk') {
            try {
                const chunk = JSON.parse(jsonData);
                this._textContent = (this._textContent || '') + chunk.text;
                contentDiv.innerHTML = this.renderMarkdown(this._textContent);
                this.autoScroll();
            } catch (e) {
                console.error('Failed to parse chunk:', e);
            }
        } else if (eventType === 'tool_use') {
            try {
                const tool = JSON.parse(jsonData);
                this.renderToolStatus(contentDiv, tool);
                this.autoScroll();
            } catch (e) {
                console.error('Failed to parse tool_use:', e);
            }
        } else if (eventType === 'chart_artifact') {
            try {
                const artifact = JSON.parse(jsonData);
                this.renderChart(contentDiv, artifact);
                this.autoScroll();
            } catch (e) {
                console.error('Failed to parse chart:', e);
            }
        } else if (eventType === 'done') {
            try {
                const doneData = JSON.parse(jsonData);
                if (doneData.conversation_id) {
                    this.conversationId = doneData.conversation_id;
                }
            } catch (e) {
                console.error('Failed to parse done event:', e);
            }
            this.unlockInput();
        } else if (eventType === 'error') {
            try {
                const err = JSON.parse(jsonData);
                contentDiv.innerHTML += '<p style="color: red;">' + this.escapeHtml(err.message) + '</p>';
            } catch (_e) {
                contentDiv.innerHTML += '<p style="color: red;">Unknown error</p>';
            }
            this.unlockInput();
        }
    }

    unlockInput() {
        this.isWaiting = false;
        this.sendBtn.disabled = false;
        this.input.disabled = false;
        this.input.focus();
        this._textContent = '';
    }

    renderToolStatus(container, tool) {
        const id = 'tool-' + tool.tool.replace(/[^a-z0-9]/gi, '-');
        let el = container.querySelector('#' + id);
        if (!el) {
            el = document.createElement('div');
            el.id = id;
            el.className = 'tool-status';
            container.appendChild(el);
        }
        if (tool.status === 'running') {
            el.textContent = 'Running ' + tool.tool + '...';
            el.classList.remove('completed');
            el.classList.add('running');
        } else {
            el.textContent = tool.tool + ' done';
            el.classList.remove('running');
            el.classList.add('completed');
        }
    }

    createMessageElement(role, text) {
        const el = document.createElement('div');
        el.className = 'message ' + role;
        const content = document.createElement('div');
        content.className = 'message-content';
        content.innerHTML = role === 'user' ? this.escapeHtml(text) : '';
        el.appendChild(content);
        return el;
    }

    renderMarkdown(text) {
        let html = this.escapeHtml(text);
        html = html.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>');
        html = html.replace(/\*(.*?)\*/g, '<em>$1</em>');
        html = html.replace(/\n/g, '<br>');
        return html;
    }

    renderChart(container, spec) {
        const chartDiv = document.createElement('div');
        chartDiv.className = 'chart-container';
        chartDiv.style.height = (spec.height || 300) + 'px';
        container.appendChild(chartDiv);

        // Map chart types: bar_h and grouped_bar both render as 'bar' in Frappe
        const frappeType = (spec.type === 'bar_h' || spec.type === 'grouped_bar') ? 'bar' : spec.type;

        const chartConfig = {
            title: spec.title,
            type: frappeType,
            height: spec.height || 300,
            data: {
                labels: spec.data.labels,
                datasets: spec.data.datasets.map(ds => ({
                    name: ds.name,
                    values: ds.values,
                })),
            },
        };

        if (spec.type === 'bar_h') {
            chartConfig.barOptions = { spaceRatio: 0.3 };
        }

        try {
            new frappe.Chart(chartDiv, chartConfig);
        } catch (e) {
            chartDiv.innerHTML = '<p style="color: red;">Chart error: ' + e.message + '</p>';
        }
    }

    autoScroll() {
        this.messagesContainer.scrollTop = this.messagesContainer.scrollHeight;
    }

    escapeHtml(text) {
        const map = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#039;' };
        return text.replace(/[&<>"']/g, c => map[c]);
    }

    async reset() {
        if (this.conversationId) {
            await fetch('/api/chat/reset', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ conversation_id: this.conversationId }),
            });
        }
        this.conversationId = null;
        this.messagesContainer.innerHTML = '';
        this.isWaiting = false;
        this.sendBtn.disabled = false;
        this.input.disabled = false;
    }
}
