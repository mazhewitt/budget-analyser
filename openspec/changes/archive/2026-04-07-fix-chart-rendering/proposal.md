## Why

Frappe Charts never appear in chat responses. The `search_transactions` and other tools emit `ChartSpec` data correctly via SSE `chart_artifact` events, but the charts are destroyed immediately when text chunks arrive — because `contentDiv.innerHTML = ...` replaces the entire container contents, wiping out any previously rendered chart DOM elements.

## What Changes

- Restructure the assistant message DOM so charts, tool status indicators, and text content live in separate areas that don't clobber each other
- Text chunk rendering updates only its own area, preserving charts and tool indicators
- Add system prompt guidance telling the LLM not to generate ASCII/text charts since the frontend renders real ones

## Capabilities

### New Capabilities

_None_ — this is a bug fix to existing chat rendering.

### Modified Capabilities

- `chat-context`: The SSE event rendering contract changes — chart artifacts and tool status must survive text chunk updates. The message DOM structure gains distinct zones for tools, charts, and text.

## Impact

- `static/chat.js` — main fix site. `handleSseEvent`, `renderChart`, `renderToolStatus`, and `createMessageElement` all need to work with a structured message DOM instead of a flat `contentDiv`
- `src/ai/agent.rs` — system prompt update to discourage ASCII chart generation
