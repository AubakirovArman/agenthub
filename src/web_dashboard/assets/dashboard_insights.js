window.renderInsightPanels = function renderInsightPanels() {
  const helpers = window.AgentHubRenderHelpers;
  if (!helpers) return;
  renderProviders(helpers);
  renderApprovals(helpers);
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
