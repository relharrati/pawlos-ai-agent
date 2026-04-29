import { marked } from 'marked';

interface ChatResponse {
  session_id: string;
  content: string;
}

const messagesEl   = document.getElementById('messages')!;
const inputEl      = document.getElementById('message-input') as HTMLTextAreaElement;
const sendBtn      = document.getElementById('send-btn')!;
const modelInput   = document.getElementById('model-input') as HTMLInputElement;
const personalitySelect = document.getElementById('personality-select') as HTMLSelectElement;
const agentTitle   = document.getElementById('agent-title')!;
const statusDot    = document.getElementById('status-dot')!;

let sessionId: string | null = null;
let isTyping = false;

// Auto-resize textarea
inputEl.addEventListener('input', () => {
  inputEl.style.height = 'auto';
  inputEl.style.height = Math.min(inputEl.scrollHeight, 180) + 'px';
});

// Send on Enter (Shift+Enter = newline)
inputEl.addEventListener('keydown', (e: KeyboardEvent) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    send();
  }
});

sendBtn.addEventListener('click', send);

function formatTime(): string {
  return new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

function appendMessage(role: 'user' | 'bot', content: string): HTMLElement {
  const wrap = document.createElement('div');
  wrap.className = `msg ${role}`;

  const bubble = document.createElement('div');
  bubble.className = 'bubble';

  if (role === 'bot') {
    bubble.innerHTML = marked.parse(content) as string;
  } else {
    bubble.textContent = content;
  }

  const meta = document.createElement('span');
  meta.className = 'meta';
  meta.textContent = formatTime();

  wrap.appendChild(bubble);
  wrap.appendChild(meta);
  messagesEl.appendChild(wrap);
  messagesEl.scrollTop = messagesEl.scrollHeight;
  return bubble;
}

function showTyping(): HTMLElement {
  const wrap = document.createElement('div');
  wrap.className = 'msg bot';
  wrap.id = 'typing-indicator';
  const typing = document.createElement('div');
  typing.className = 'typing bubble';
  typing.innerHTML = '<div class="dot"></div><div class="dot"></div><div class="dot"></div>';
  wrap.appendChild(typing);
  messagesEl.appendChild(wrap);
  messagesEl.scrollTop = messagesEl.scrollHeight;
  return wrap;
}

function removeTyping(): void {
  document.getElementById('typing-indicator')?.remove();
}

async function send(): Promise<void> {
  const text = inputEl.value.trim();
  if (!text || isTyping) return;

  isTyping = true;
  inputEl.value = '';
  inputEl.style.height = 'auto';

  appendMessage('user', text);
  const typingEl = showTyping();

  // Handle slash commands client-side
  if (text.startsWith('/personality ')) {
    const name = text.slice('/personality '.length).trim();
    personalitySelect.value = name;
    removeTyping();
    appendMessage('bot', `Personality switched to **${name}** for this session.`);
    isTyping = false;
    return;
  }

  if (text.startsWith('/model ')) {
    const m = text.slice('/model '.length).trim();
    modelInput.value = m;
    removeTyping();
    appendMessage('bot', `Model switched to \`${m}\`.`);
    isTyping = false;
    return;
  }

  try {
    const body: Record<string, string> = { message: text };
    if (sessionId) body['session_id'] = sessionId;
    if (modelInput.value) body['model'] = modelInput.value;

    const res = await fetch('/api/chat', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });

    removeTyping();

    if (!res.ok) {
      const err = await res.text();
      appendMessage('bot', `⚠️ Error: ${err}`);
    } else {
      const data: ChatResponse = await res.json();
      sessionId = data.session_id;
      appendMessage('bot', data.content);
    }
  } catch (err) {
    removeTyping();
    appendMessage('bot', `⚠️ Network error: ${err}`);
    statusDot.style.background = '#ff4444';
    statusDot.style.boxShadow = '0 0 6px #ff4444';
  } finally {
    isTyping = false;
  }
}

// Check server on load
fetch('/api/chat', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ message: '' }) })
  .catch(() => {
    statusDot.style.background = '#ffaa00';
    statusDot.style.boxShadow = '0 0 6px #ffaa00';
  });

// Welcome message
appendMessage('bot', '**pawlos is awake.** How can I help you today?\n\nTips: `/model openai/gpt-4o` to switch models, `/personality concise` to change tone.');
