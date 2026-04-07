### Requirement: Chart artifacts persist through text streaming
The frontend SHALL render chart artifacts into a dedicated chart area within the message DOM that is not affected by subsequent text chunk updates.

#### Scenario: Chart survives text chunks
- **WHEN** the frontend receives a `chart_artifact` SSE event followed by `chunk` SSE events
- **THEN** the Frappe Chart remains visible in the final rendered message alongside the text response

#### Scenario: Multiple charts in one response
- **WHEN** a tool returns multiple ChartSpecs (e.g. merchant_breakdown returns bar + pie)
- **THEN** all charts render in the chart area and remain visible after text streaming completes

### Requirement: Tool status indicators persist through text streaming
The frontend SHALL render tool status indicators into a dedicated tool area within the message DOM that is not affected by subsequent text chunk updates.

#### Scenario: Tool status survives text chunks
- **WHEN** the frontend receives `tool_use` SSE events followed by `chunk` SSE events
- **THEN** the tool status indicators remain visible in the final rendered message

### Requirement: Message DOM has structured zones
Each assistant message SHALL contain three distinct DOM zones: a tool area, a chart area, and a text area. SSE event handlers SHALL write only to their respective zone.

#### Scenario: Text chunks update only the text area
- **WHEN** a `chunk` SSE event is received
- **THEN** only the text area's innerHTML is updated; the tool area and chart area are untouched

### Requirement: System prompt discourages text-based charts
The system prompt SHALL instruct the LLM not to generate ASCII charts, text-based bar charts, or markdown tables duplicating chart data, since the frontend renders charts visually.

#### Scenario: LLM responds with narrative instead of ASCII chart
- **WHEN** a tool returns chart data and the LLM generates a text response
- **THEN** the text response contains narrative insights (e.g., "spending peaked in November") rather than an ASCII representation of the chart data

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
