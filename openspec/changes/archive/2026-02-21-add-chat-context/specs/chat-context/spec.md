## ADDED Requirements

### Requirement: Backend returns conversation ID in done event
The `done` SSE event SHALL include a `conversation_id` field containing the session UUID for the current conversation.

#### Scenario: First message in a new conversation
- **WHEN** the frontend sends a chat request with `conversation_id: null`
- **THEN** the backend creates a new session and the `done` SSE event includes the newly generated `conversation_id`

#### Scenario: Subsequent message in an existing conversation
- **WHEN** the frontend sends a chat request with a valid `conversation_id`
- **THEN** the `done` SSE event includes the same `conversation_id`

### Requirement: Frontend persists conversation ID across messages
The frontend SHALL store the `conversation_id` received from the `done` SSE event and include it in all subsequent chat requests within the same session.

#### Scenario: Conversation ID stored after first response
- **WHEN** the frontend receives a `done` event with a `conversation_id`
- **THEN** `this.conversationId` is set to that value and the next chat request includes it in the JSON body

#### Scenario: New chat resets conversation ID
- **WHEN** the user clicks "New Chat"
- **THEN** `this.conversationId` is reset to `null` and the next message starts a fresh session

### Requirement: Full conversation history sent to LLM
The agent SHALL pass the complete conversation history — including all prior user messages, assistant responses, tool use blocks, and tool result blocks — to the Claude API on every turn.

#### Scenario: Follow-up question references prior tool results
- **WHEN** the user asks "break that down by merchant" after a spending-by-category response
- **THEN** the LLM receives the full history including the prior tool call and result, and can resolve "that" to the previous category context

#### Scenario: Multi-turn tool chaining
- **WHEN** the user asks a sequence of related questions across 3+ turns
- **THEN** each LLM call includes all prior turns' messages, tool calls, and tool results

### Requirement: Conversation history is unbounded
The system SHALL NOT truncate, summarise, or limit the conversation history within a session. The full history is sent on every turn regardless of length.

#### Scenario: Long session with many tool calls
- **WHEN** a session accumulates 15+ turns with multiple tool calls per turn
- **THEN** the complete history is sent to the Claude API without truncation
