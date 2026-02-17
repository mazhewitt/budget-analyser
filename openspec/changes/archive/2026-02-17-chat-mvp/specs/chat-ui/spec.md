## ADDED Requirements

### Requirement: Chat message input
The UI SHALL provide a text input area and send button for the user to type messages.

#### Scenario: Send message
- **WHEN** the user types a message and presses Enter or clicks Send
- **THEN** the message is sent via POST to `/api/chat` and the input is cleared

#### Scenario: Disable during response
- **WHEN** the agent is processing a response (SSE stream is active)
- **THEN** the input is disabled until the `done` event is received

### Requirement: Streaming text display
The UI SHALL display assistant text as it arrives via SSE `chunk` events, creating a streaming effect.

#### Scenario: Text chunks arrive
- **WHEN** `chunk` events are received with `{"text": "..."}`
- **THEN** the text is appended to the current assistant message in real-time

#### Scenario: Markdown rendering
- **WHEN** the complete assistant message contains markdown (bold, lists, tables)
- **THEN** the UI renders it as formatted HTML

### Requirement: Tool use indicators
The UI SHALL display a visual indicator when the agent is executing a tool.

#### Scenario: Tool execution
- **WHEN** a `tool_use` event is received with `{"tool": "spending_by_category", "status": "running"}`
- **THEN** the UI shows an indicator (e.g. "Analysing spending by category...")

#### Scenario: Tool completion
- **WHEN** a `tool_use` event is received with `{"status": "completed"}`
- **THEN** the indicator updates to reflect completion

### Requirement: Chart rendering from artifacts
The UI SHALL render charts inline in the conversation when `chart_artifact` events are received.

#### Scenario: Bar chart artifact
- **WHEN** a `chart_artifact` event is received with `{"type": "bar", "title": "...", "data": {...}}`
- **THEN** the UI creates a Frappe Chart of the specified type and inserts it into the conversation flow

#### Scenario: Multiple charts in one response
- **WHEN** multiple `chart_artifact` events are received in a single response
- **THEN** each chart is rendered in order within the conversation

### Requirement: Conversation history display
The UI SHALL display the full conversation history with user messages and assistant responses visually distinguished.

#### Scenario: Message distinction
- **WHEN** the conversation has multiple turns
- **THEN** user messages and assistant responses are visually distinct (different alignment or styling)

### Requirement: New conversation
The UI SHALL provide a way to start a new conversation, clearing the chat history.

#### Scenario: Reset conversation
- **WHEN** the user clicks a "New Chat" button
- **THEN** the UI clears the displayed messages and sends a reset request to the server

### Requirement: Auto-scroll during streaming
The UI SHALL auto-scroll to the bottom as new content arrives during streaming.

#### Scenario: Long response
- **WHEN** a streaming response extends beyond the visible area
- **THEN** the chat container scrolls to keep the latest content visible
