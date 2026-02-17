## ADDED Requirements

### Requirement: Chart specification format
Chart specifications SHALL use a JSON structure compatible with Frappe Charts, containing `type`, `title`, `data`, and optional `height`.

#### Scenario: Bar chart spec
- **WHEN** a tool produces a vertical bar chart
- **THEN** the chart spec is `{"type": "bar", "title": "...", "data": {"labels": [...], "datasets": [{"name": "...", "values": [...]}]}, "height": 300}`

#### Scenario: Horizontal bar chart spec
- **WHEN** a tool produces a horizontal bar chart
- **THEN** the chart spec uses `"type": "bar_h"` with the same data structure

#### Scenario: Pie chart spec
- **WHEN** a tool produces a pie chart
- **THEN** the chart spec uses `"type": "pie"` with the same data structure

#### Scenario: Grouped bar chart spec
- **WHEN** a tool produces a grouped bar chart (e.g. income vs spending)
- **THEN** the chart spec uses `"type": "bar"` with multiple datasets: `"datasets": [{"name": "Income", "values": [...]}, {"name": "Spending", "values": [...]}]`

### Requirement: Supported chart types
The system SHALL support four chart types: `bar` (vertical), `bar_h` (horizontal), `pie`, and `grouped_bar` (vertical bars with multiple datasets).

#### Scenario: Type mapping to Frappe Charts
- **WHEN** the frontend receives a chart spec with `type: "bar"`
- **THEN** it creates a Frappe Chart with `type: "bar"`

#### Scenario: Horizontal bar mapping
- **WHEN** the frontend receives a chart spec with `type: "bar_h"`
- **THEN** it creates a Frappe Chart with `type: "bar"` and `barOptions: {spaceRatio: 0.3}` oriented horizontally

### Requirement: Chart artifacts transmitted via SSE
Chart specifications SHALL be transmitted as `chart_artifact` SSE events, separate from the text stream.

#### Scenario: Chart event format
- **WHEN** a tool returns a chart specification
- **THEN** the server emits `event: chart_artifact\ndata: {"type": "bar", "title": "...", "data": {...}}\n\n`

#### Scenario: Chart position in conversation
- **WHEN** a `chart_artifact` event arrives during a response
- **THEN** the chart is inserted at the current position in the assistant's message flow (after any preceding text, before any following text)

### Requirement: Chart data stays out of LLM context
Chart data SHALL NOT be included in the tool result sent back to the LLM. Only the text summary is fed back to the LLM.

#### Scenario: Tool result separation
- **WHEN** `spending_by_category` returns `{"summary": "Top category is Groceries at CHF 45,000", "chart": {"type": "bar_h", ...}}`
- **THEN** the LLM receives only the summary text, and the chart object is emitted as an SSE event to the frontend
