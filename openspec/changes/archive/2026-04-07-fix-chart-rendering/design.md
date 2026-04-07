## Context

The chat UI renders assistant messages into a single `contentDiv`. Tool status indicators, Frappe Charts, and streamed text all write to this same div. When text chunks arrive via SSE, `contentDiv.innerHTML = renderMarkdown(text)` replaces the entire div contents, destroying any chart DOM that Frappe previously rendered.

The backend pipeline works correctly — `ChartSpec` objects are created by tools, emitted as `AgentEvent::ChartArtifact`, serialized to SSE `chart_artifact` events, and received by the frontend. The break is purely in how the frontend manages the DOM.

## Goals / Non-Goals

**Goals:**
- Frappe Charts survive text chunk rendering and remain visible in the final message
- Tool status indicators survive text chunk rendering
- Event ordering is preserved: tools appear first, then charts, then text — matching the SSE emission order
- Claude stops generating redundant ASCII charts in its text response

**Non-Goals:**
- Changing the SSE protocol or event format
- Adding new chart types or interactive chart features
- Changing the backend tool output or ChartSpec structure
- Making charts responsive/resizable

## Decisions

### Structured message DOM

Split the flat `contentDiv` into three zones:

```
div.message.assistant
└── div.message-content
    ├── div.tool-area      ← tool status indicators
    ├── div.chart-area     ← Frappe Chart containers
    └── div.text-area      ← streamed text (innerHTML updates here only)
```

`renderToolStatus` appends to `.tool-area`. `renderChart` appends to `.chart-area`. Text chunk handling sets `.text-area.innerHTML` only.

**Why not append text nodes instead of innerHTML?** The existing `renderMarkdown` converts markdown to HTML (bold, italic, line breaks). Using `innerHTML` on a contained element is the simplest approach that preserves this behavior without rewriting the markdown renderer.

### System prompt addition

Add a single line to the system prompt in `build_system_prompt`:

> "Charts are rendered visually by the frontend. Do not generate text-based charts, ASCII bar charts, or markdown tables of monthly data — just summarize insights in words."

This prevents Claude from duplicating chart data as ASCII art in its text response.

## Risks / Trade-offs

- **CSS may need minor adjustment** for the new DOM structure → Minimal risk, the zones are just divs with no special styling needed beyond what `.message-content` already provides.
- **Existing tool status querySelector relies on flat DOM** → `renderToolStatus` uses `container.querySelector('#' + id)` which works fine with nested structure since querySelector searches descendants.
