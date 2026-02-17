## ADDED Requirements

### Requirement: LLM provider connects to Claude API
The agent SHALL use the Anthropic Claude API for language model inference, configured via the `ANTHROPIC_API_KEY` environment variable.

#### Scenario: Successful API call
- **WHEN** the agent sends a message to the Claude API with a valid API key
- **THEN** it receives a response containing text and/or tool use blocks

#### Scenario: Missing API key
- **WHEN** the server starts without `ANTHROPIC_API_KEY` set
- **THEN** it exits with an error message indicating the missing key

### Requirement: Agent loop executes tool calls iteratively
The agent SHALL implement a loop: send messages to the LLM, execute any tool calls in the response, feed results back to the LLM, and repeat until the LLM returns a text-only response or a maximum iteration limit is reached.

#### Scenario: Single tool call
- **WHEN** the user asks "How much did I spend on groceries?"
- **THEN** the agent calls the LLM, the LLM requests `spending_by_category`, the agent executes the tool, feeds the result back, and the LLM generates a narrative response

#### Scenario: Multiple tool calls in sequence
- **WHEN** the user asks "Compare my grocery spending to dining by month"
- **THEN** the agent MAY execute multiple tool calls across iterations before producing a final response

#### Scenario: Maximum iterations reached
- **WHEN** the agent loop reaches 10 iterations without a text-only response
- **THEN** it stops and returns whatever text has been accumulated, plus an indication that the response may be incomplete

### Requirement: Tool definitions are registered with the LLM
The agent SHALL provide tool definitions (name, description, input schema) to the Claude API on each request so the LLM can choose which tools to call.

#### Scenario: Tools available in conversation
- **WHEN** the agent sends a message to the LLM
- **THEN** the request includes definitions for all registered analysis tools

### Requirement: Conversation history is maintained per session
The agent SHALL maintain a message history for each conversation session, including user messages, assistant responses, and tool call/result pairs.

#### Scenario: Multi-turn conversation
- **WHEN** the user sends a follow-up message like "now break that down by merchant"
- **THEN** the agent includes the full conversation history so the LLM has context from prior turns

#### Scenario: Session expiry
- **WHEN** a session has not been accessed for more than 2 hours
- **THEN** the session MAY be evicted from memory

### Requirement: System prompt provides budget context
The agent SHALL include a system prompt that describes the category schema, the data model, the date range of available data, and guidance on when to use each tool.

#### Scenario: LLM knows the categories
- **WHEN** the user asks "what categories do you have?"
- **THEN** the LLM can answer from the system prompt without calling a tool

#### Scenario: Data summary in prompt
- **WHEN** the server starts
- **THEN** it queries the database for a summary (date range, total transactions, category list) and includes it in the system prompt
