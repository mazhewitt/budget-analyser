## 1. Restructure message DOM

- [x] 1.1 Update `createMessageElement` in `static/chat.js` to create three child zones inside `.message-content`: `.tool-area`, `.chart-area`, `.text-area`
- [x] 1.2 Update `handleSseEvent` for `chunk` events to set innerHTML on `.text-area` only instead of the parent `contentDiv`
- [x] 1.3 Update `renderToolStatus` to target `.tool-area` within the message container
- [x] 1.4 Update `renderChart` to target `.chart-area` within the message container

## 2. System prompt

- [x] 2.1 Add guidance to `build_system_prompt` in `src/ai/agent.rs` telling the LLM not to generate ASCII/text charts since the frontend renders them visually

## 3. Verify

- [x] 3.1 Test that a search query (e.g. "Digitec spending") renders a Frappe bar chart that remains visible after the text response streams in (manual browser test)
- [x] 3.2 Test that tool status indicators remain visible after text streaming (manual browser test)
- [x] 3.3 Test that multi-chart responses (e.g. merchant_breakdown with bar + pie) both render and persist (manual browser test)
