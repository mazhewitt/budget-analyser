## 1. Backend — return conversation_id in done event

- [x] 1.1 Add `conversation_id: String` field to `SseDone` struct in `src/chat/handler.rs`
- [x] 1.2 Pass `conversation_id` into the SSE stream and include it in the `done` event payload

## 2. Frontend — persist and send conversation_id

- [x] 2.1 Parse `conversation_id` from the `done` SSE event data in `chat.js` and store it as `this.conversationId`
- [x] 2.2 Verify `this.conversationId` is already sent in chat requests (existing code) and reset to `null` on "New Chat" (existing code)

## 3. Verification

- [x] 3.1 Test: send first message, confirm `done` event contains a `conversation_id` UUID
- [x] 3.2 Test: send follow-up message, confirm it references prior context (e.g. "what was that?" after a category breakdown)
- [x] 3.3 Test: click "New Chat", confirm next message starts a fresh session with no prior history
