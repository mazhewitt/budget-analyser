## ADDED Requirements

### Requirement: HTTP server serves static files
The server SHALL serve static files (HTML, JS, CSS) from a `static/` directory at the root path `/`.

#### Scenario: Browser loads chat page
- **WHEN** a browser requests `GET /`
- **THEN** the server returns `static/chat.html` with content-type `text/html`

#### Scenario: Browser loads JavaScript
- **WHEN** a browser requests `GET /app.js`
- **THEN** the server returns the corresponding file from the `static/` directory

### Requirement: Chat API endpoint accepts messages
The server SHALL expose a `POST /api/chat` endpoint that accepts a JSON payload with a message and optional conversation ID.

#### Scenario: New conversation
- **WHEN** a POST request is sent to `/api/chat` with `{"message": "How much did I spend on groceries?"}` and no `conversation_id`
- **THEN** the server creates a new conversation session and returns an SSE stream

#### Scenario: Continuing conversation
- **WHEN** a POST request is sent to `/api/chat` with `{"message": "Break that down by store", "conversation_id": "abc-123"}`
- **THEN** the server retrieves the existing conversation history and returns an SSE stream that includes context from prior turns

### Requirement: Chat endpoint returns SSE stream
The server SHALL respond to `/api/chat` with `Content-Type: text/event-stream` and stream events as the agent processes the request.

#### Scenario: Streaming response
- **WHEN** the agent generates a response
- **THEN** the server streams SSE events (`chunk`, `tool_use`, `chart_artifact`, `done`, `error`) as they occur

### Requirement: Conversation reset
The server SHALL expose a `POST /api/chat/reset` endpoint that clears the current conversation.

#### Scenario: Reset conversation
- **WHEN** a POST request is sent to `/api/chat/reset` with `{"conversation_id": "abc-123"}`
- **THEN** the session for that conversation is deleted and subsequent requests with that ID start fresh

### Requirement: Server configuration via environment
The server SHALL read configuration from environment variables with sensible defaults.

#### Scenario: Default configuration
- **WHEN** the server starts with no environment variables set (except `ANTHROPIC_API_KEY`)
- **THEN** it binds to `127.0.0.1:3000` and uses `data/budget.db` as the database path

#### Scenario: Custom configuration
- **WHEN** `BIND_ADDRESS` is set to `0.0.0.0:8080` and `DATABASE_URL` is set to `/path/to/other.db`
- **THEN** the server binds to that address and uses that database
