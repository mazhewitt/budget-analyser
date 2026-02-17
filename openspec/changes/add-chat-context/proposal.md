## Why

The chat interface currently has no memory between messages. Every user message starts a fresh LLM conversation because the `conversation_id` is never returned to the frontend — so the backend's session infrastructure (which already stores history) is never utilised. This makes follow-up questions impossible: the user can't say "break that down by merchant" after asking about spending by category, because the LLM has no idea what "that" refers to.

## What Changes

- Return `conversation_id` from the backend to the frontend in the `done` SSE event
- Store `conversation_id` on the frontend and send it with every subsequent message
- Full conversation history (user messages, assistant responses, tool calls, and tool results) flows to the LLM on every turn via the existing session store
- Context is unbounded — no message truncation or compaction (to be added later if needed)

## Capabilities

### New Capabilities
- `chat-context`: Multi-turn conversation continuity — the LLM receives the full history of messages, tool calls, and tool results from the current session, enabling follow-up questions and contextual references

### Modified Capabilities

## Impact

- **Backend**: `SseDone` struct in `src/chat/handler.rs` gains a `conversation_id` field; the `done` SSE event payload changes shape (additive, not breaking)
- **Frontend**: `chat.js` parses `conversation_id` from `done` events and includes it in subsequent requests
- **LLM context window**: Unbounded history means long sessions will eventually hit the Claude API's context limit; accepted for now, compaction is a future concern
- **No database changes**: Sessions remain in-memory with 2-hour TTL as today
