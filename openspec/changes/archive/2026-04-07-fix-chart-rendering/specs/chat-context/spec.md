## ADDED Requirements

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
