window.renderInsightPanels = function renderInsightPanels() {
  const helpers = window.AgentHubRenderHelpers;
  if (!helpers) return;
  renderProviders(helpers);
  renderApprovals(helpers);
  renderObservability(helpers);
  renderMemoryBrowser(helpers);
  renderHistoryBrowser(helpers);
};

function renderProviders({ el, badge }) {
  const panel = document.querySelector("#providerPanel");
  if (!panel) return;
  const providers = (data.providers && data.providers.statuses) || [];
  const roles = (data.providers && data.providers.roles) || [];
  panel.replaceChildren(
    el("div", { class: "provider-section" }, providers.map((provider) =>
      el("div", { class: "item" }, [
        el("strong", { text: provider.id }),
        provider.is_default ? el("small", { text: " default" }) : el("small", { text: "" }),
        el("div", {}, [badge(provider.state)]),
        el("p", { text: provider.detail }),
      ])
    )),
    el("div", { class: "provider-section" }, roles.map((role) =>
      el("div", { class: "item" }, [
        el("strong", { text: `${role.role}: ${role.provider}` }),
        el("small", { text: role.available === false ? " missing" : " ready" }),
        el("p", { text: role.fallback.length ? `fallback: ${role.fallback.join(", ")}` : "no fallback" }),
      ])
    )),
  );
}

function renderApprovals({ el, badge }) {
  const panel = document.querySelector("#approvalInbox");
  if (!panel) return;
  const items = data.approvals || [];
  if (items.length === 0) {
    panel.replaceChildren(el("div", { class: "item muted", text: "No pending approvals" }));
    return;
  }
  panel.replaceChildren(...items.map((item) => el("div", { class: "item" }, [
    el("strong", { text: item.id }),
    el("div", {}, [badge(item.status)]),
    el("small", { text: item.kind }),
    el("p", { text: item.detail }),
  ])));
}

function renderObservability({ el, badge }) {
  renderContextReceipt(el);
  renderChatEvents(el, badge);
  renderToolLoopReceipts(el, badge);
  renderToolResultReceipts(el, badge);
  renderToolLogs(el);
}

function renderContextReceipt(el) {
  const panel = document.querySelector("#contextReceipt");
  if (!panel) return;
  const receipt = data.observability && data.observability.context_receipt;
  if (!receipt) {
    panel.replaceChildren(el("div", { class: "item muted", text: "No context receipt yet" }));
    return;
  }
  const budget = receipt.budget || {};
  panel.replaceChildren(
    receiptMetric(el, "Prompt tokens", receipt.prompt_tokens),
    receiptMetric(el, "Memory tokens", receipt.memory_tokens),
    receiptMetric(el, "Memory selected", receipt.memory_records_selected),
    receiptMetric(el, "Memory available", receipt.memory_records_available),
    receiptMetric(el, "Budget dropped", receipt.memory_records_budget_dropped),
    receiptMetric(el, "Recent dropped", receipt.recent_messages_dropped),
    receiptMetric(el, "Max prompt", budget.max_prompt_tokens),
    receiptMetric(el, "Compressed", receipt.compressed ? "yes" : "no"),
  );
}

function receiptMetric(el, label, value) {
  return el("div", { class: "item compact" }, [
    el("span", { text: label }),
    el("strong", { text: value == null ? "unknown" : String(value) }),
  ]);
}

function renderChatEvents(el, badge) {
  const panel = document.querySelector("#chatEvents");
  if (!panel) return;
  const events = ((data.observability && data.observability.chat_events) || []).slice(0, 12);
  if (events.length === 0) {
    panel.replaceChildren(el("div", { class: "item muted", text: "No chat events yet" }));
    return;
  }
  panel.replaceChildren(...events.map((event) => el("div", { class: "item" }, [
    el("strong", { text: event.kind }),
    el("div", {}, [badge(event.status || event.mode || "event")]),
    el("small", { text: chatEventMeta(event) }),
    event.kind === "session_recovery" ? el("p", { class: "bad-text", text: event.reason || event.text }) : el("p", { text: event.text || event.reason || "" }),
  ])));
}

function chatEventMeta(event) {
  const pieces = [event.chat_id];
  if (event.provider) pieces.push(event.provider);
  if (event.at) pieces.push(new Date(event.at).toLocaleString());
  if (event.path) pieces.push(event.path);
  return pieces.join(" | ");
}

function renderToolLoopReceipts(el, badge) {
  const panel = document.querySelector("#toolLoopReceipts");
  if (!panel) return;
  const receipts = (data.observability && data.observability.tool_loop_receipts) || [];
  const chatPermissions = (data.observability && data.observability.tool_permissions) || [];
  if (receipts.length === 0 && chatPermissions.length === 0) {
    panel.replaceChildren(el("div", { class: "item muted", text: "No tool loop receipts yet" }));
    return;
  }
  const receiptNodes = receipts.map((receipt) => el("div", { class: "item" }, [
    el("a", { href: receipt.href, text: `${receipt.tx_id} ${receipt.role}` }),
    el("div", {}, [badge(receipt.blocked ? "blocked" : receipt.status)]),
    el("small", { text: `source ${receipt.plan_source || "unknown"} | native calls ${receipt.native_tool_calls}` }),
    receipt.blocked_reason ? el("p", { class: "bad-text", text: receipt.blocked_reason }) : el("p", { text: toolPermissionSummary(receipt.command_permissions) }),
  ]));
  const permissionNodes = chatPermissions.slice(0, 8).map((item) => el("div", { class: "item" }, [
    el("strong", { text: `${item.tool}: ${item.profile}` }),
    el("div", {}, [badge(item.approval_required ? "approval" : item.risk)]),
    el("small", { text: item.source }),
    el("p", { text: item.action || item.reason || "" }),
  ]));
  panel.replaceChildren(...receiptNodes, ...permissionNodes);
}

function renderToolResultReceipts(el, badge) {
  const panel = document.querySelector("#toolResultReceipts");
  if (!panel) return;
  const receipts = (data.observability && data.observability.tool_result_receipts) || [];
  if (receipts.length === 0) {
    panel.replaceChildren(el("div", { class: "item muted", text: "No tool result receipts yet" }));
    return;
  }
  panel.replaceChildren(...receipts.slice(0, 8).map((receipt) => el("div", { class: "item" }, [
    el("a", { href: receipt.href, text: `${receipt.tx_id} ${receipt.role}` }),
    el("div", {}, [badge(receipt.blocked ? "blocked" : receipt.status)]),
    el("small", { text: `${receipt.rounds} rounds | ${receipt.results} results` }),
    receipt.blocked_reason ? el("p", { class: "bad-text", text: receipt.blocked_reason }) : el("p", { text: "redacted builtin tool results recorded" }),
  ])));
}

function toolPermissionSummary(items) {
  if (!items || items.length === 0) return "no command permissions";
  return items.slice(0, 3).map((item) =>
    `${item.action || item.tool}: ${item.profile}/${item.risk}${item.approval_required ? " approval" : ""}`
  ).join(" | ");
}

function renderToolLogs(el) {
  const panel = document.querySelector("#toolLogs");
  if (!panel) return;
  const logs = (data.observability && data.observability.tool_logs) || [];
  if (logs.length === 0) {
    panel.replaceChildren(el("div", { class: "item muted", text: "No tool logs yet" }));
    return;
  }
  panel.replaceChildren(...logs.slice(0, 8).map((log) => el("div", { class: "item" }, [
    el("a", { href: log.href, text: `${log.tx_id}/${log.name}` }),
    el("pre", { text: log.excerpt }),
  ])));
}

function renderMemoryBrowser({ el }) {
  const panel = document.querySelector("#memoryBrowser");
  if (!panel) return;
  const items = data.memory_browser || [];
  panel.replaceChildren(...items.map((item) => el("div", { class: "item" }, [
    el("strong", { text: `${item.kind}: ${item.id}` }),
    el("small", { text: ` ${item.status || "unknown"} ${item.schema || ""}` }),
    el("p", { text: item.summary }),
  ])));
}

function renderHistoryBrowser({ el, badge }) {
  const panel = document.querySelector("#historyBrowser");
  if (!panel) return;
  const items = data.history || [];
  panel.replaceChildren(...items.map((item) => el("div", { class: "item" }, [
    el("a", { href: item.report_href, text: item.tx_id }),
    el("div", {}, [badge(item.status)]),
    el("small", { text: historyMeta(item) }),
    el("p", { text: item.latest_event || "no journal events" }),
  ])));
}

function historyMeta(item) {
  const pieces = [];
  if (item.provider) pieces.push(`provider ${item.provider}`);
  if (item.domain_runtime) pieces.push(item.domain_runtime);
  if (item.cost_usd != null) pieces.push(`$${item.cost_usd.toFixed(6)}`);
  if (item.latest_ts) pieces.push(new Date(item.latest_ts).toLocaleString());
  return pieces.join(" | ");
}
