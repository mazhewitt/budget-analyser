// app.js: Application coordinator

let chatManager;

document.addEventListener('DOMContentLoaded', () => {
    chatManager = new ChatManager();

    const chatInput = document.getElementById('chat-input');
    const sendBtn = document.getElementById('send-btn');
    const newChatBtn = document.getElementById('new-chat-btn');

    sendBtn.addEventListener('click', () => {
        const message = chatInput.value;
        chatManager.sendMessage(message);
    });

    chatInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            const message = chatInput.value;
            chatManager.sendMessage(message);
        }
    });

    newChatBtn.addEventListener('click', () => {
        chatManager.reset();
    });

    chatInput.focus();
});
