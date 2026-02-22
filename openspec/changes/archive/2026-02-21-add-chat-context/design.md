## Context

The chat backend already has session infrastructure: `SessionStore` maintains a `HashMap<String, SessionEntry>` with conversation history (`Vec<Message>`) and a 2-hour TTL. The agent loop (`Agent::chat`) mutates this history in place — appending user messages, assistant responses, tool calls, and tool results — and the handler saves it back via `sessions.save_history()`.

The problem is a wiring gap: the `done` SSE event doesn't include the `conversation_id`, so the frontend never learns which session it belongs to. Every request sends `conversation_id: null`, creating a fresh empty session each time.

## Goals / Non-Goals

**Goals:**
- Enable multi-turn conversations where the LLM receives full history (messages + tool calls + tool results)
- Require minimal changes — fix the wiring gap, not redesign the session system

**Non-Goals:**
- Context compaction or summarisation (future work)
- Persistent sessions across server restarts (sessions remain in-memory)
- Streaming of individual tool results to the frontend (current batch-after-completion approach is fine)

## Decisions

### 1. Return conversation_id in the done SSE event

Add `conversation_id: String` to `SseDone`. The frontend parses it from the `done` event and stores it as `this.conversationId` for subsequent requests.

**Why not a separate SSE event?** Adding a field to `done` is simpler and the ID is only useful once the turn is complete. A separate `session` event would add complexity for no benefit.

**Why not return it in the HTTP response headers?** SSE responses are streamed; by the time we know the conversation_id (immediately, actually), the headers are already sent. Using the event stream is consistent with the existing protocol.

### 2. Unbounded history — no truncation

The full `Vec<Message>` is sent to the Claude API on every turn. For typical budget analysis sessions (5-15 turns), this stays well within the 200k token context window. Tool results (spending summaries, category breakdowns) are text-based and small.

**Why not add a token budget now?** Premature. The user explicitly wants unbounded context and will add compaction later. Adding truncation logic now would be unnecessary complexity.

### 3. No changes to session storage

The in-memory `SessionStore` with 2-hour TTL is adequate. Sessions don't need database persistence — budget analysis is an interactive session, not a long-lived conversation.

## Risks / Trade-offs

- **Context window exhaustion on very long sessions** → Accepted. The Claude API will return an error if the context is exceeded. The user plans to add compaction later.
- **Cost increase from larger prompts** → Each turn sends the full history, so token usage grows linearly with session length. For typical sessions this is negligible.
