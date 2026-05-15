const data = window.AGENTHUB_DATA;
const $ = (selector) => document.querySelector(selector);

function el(name, attrs = {}, children = []) {
  const node = document.createElement(name);
  Object.entries(attrs).forEach(([key, value]) => {
    if (key === "class") node.className = value;
    else if (key === "text") node.textContent = value;
    else node.setAttribute(key, value);
  });
  children.forEach((child) => node.append(child));
  return node;
}

function metric(label, value) {
  return el("div", { class: "metric" }, [
    el("strong", { text: String(value) }),
    el("span", { text: label }),
  ]);
}

function badge(status) {
  const klass = status.includes("FAIL") || status.includes("ERROR")
    ? "bad"
    : status === "COMMITTED" || status === "CLOSED"
      ? "ok"
      : "warn";
  return el("span", { class: `badge ${klass}`, text: status });
}

function renderHeader() {
  $("#project").textContent = data.project;
  $("#generated").textContent = new Date(data.generated_at).toLocaleString();
}

function renderMetrics() {
  const s = data.summary;
  $("#metrics").replaceChildren(
    metric("Transactions", s.transaction_count),
    metric("Open", s.open_count),
    metric("Failed", s.failed_count),
    metric("Memory", s.memory_records),
    metric("Skills", s.skill_count),
    metric("Cost", `$${s.total_cost_usd.toFixed(4)}`),
  );
}

function renderTransactions() {
  $("#transactions").replaceChildren(...data.transactions.map((tx) => {
    const link = el("a", { href: tx.report_href, text: "open" });
    return el("tr", {}, [
      el("td", { text: tx.id }),
      el("td", {}, [badge(tx.status)]),
      el("td", { text: `${tx.dag_nodes} nodes / ${tx.dag_edges} edges` }),
      el("td", { text: tx.cost_usd == null ? "unknown" : `$${tx.cost_usd.toFixed(6)}` }),
      el("td", {}, [link]),
    ]);
  }));
}

function renderCost() {
  $("#cost").replaceChildren(
    el("div", { class: "item" }, [
      el("strong", { text: `$${data.cost.total_usd.toFixed(6)}` }),
      el("small", { text: " total USD" }),
    ]),
    el("div", { class: "item" }, [
      el("strong", { text: String(data.cost.estimated_tokens) }),
      el("small", { text: " estimated tokens" }),
    ]),
  );
}

function renderMetricsDashboard() {
  const m = data.metrics;
  const pct = (value) => `${(value * 100).toFixed(1)}%`;
  const money = (value) => `$${value.toFixed(6)}`;
  const groups = [
    ["Reliability", [["Committed", m.reliability.committed], ["Failed", m.reliability.failed], ["Blocked", m.reliability.blocked], ["Success", pct(m.reliability.success_rate)]]],
    ["Context", [["Memory", m.context.memory_records], ["Failed attempts", m.context.failed_attempts], ["Tokens", m.context.estimated_tokens], ["Avg DAG", m.context.average_dag_nodes.toFixed(1)]]],
    ["Quality", [["Verifier", `${m.quality.verifier_passed}/${m.quality.verifier_total}`], ["Review", `${m.quality.review_passed}/${m.quality.review_total}`], ["Gate pass", pct(m.quality.gate_pass_rate)]]],
    ["Trust", [["Plugins", m.trust.installed_plugins], ["Signed", m.trust.signed_plugins], ["Verified", m.trust.verified_signatures], ["Trusted", m.trust.trusted_plugins]]],
    ["Cost", [["Total", money(m.cost.total_usd)], ["Average", money(m.cost.average_usd)], ["Tokens", m.cost.estimated_tokens]]],
  ];
  $("#metricsDashboard").replaceChildren(...groups.map(([title, items]) =>
    el("div", { class: "metric-group" }, [
      el("strong", { text: title }),
      ...items.map(([label, value]) => el("div", { class: "metric-row" }, [
        el("span", { text: label }),
        el("b", { text: String(value) }),
      ])),
    ])
  ));
}

function renderTimeline() {
  const items = data.timeline.map((event) => el("li", {}, [
    el("time", { text: new Date(event.ts).toLocaleString() }),
    el("div", {}, [badge(event.state)]),
    el("strong", { text: event.tx_id }),
    el("p", { text: event.message }),
  ]));
  $("#timeline").replaceChildren(...items);
}

function renderTrace() {
  const latest = data.transactions[0];
  if (!latest || latest.dag_roles.length === 0) {
    $("#trace").replaceChildren(el("div", { class: "item", text: "No trace data" }));
    return;
  }
  $("#trace").replaceChildren(...latest.dag_roles.map((role, index) =>
    el("span", { class: "chip", text: `${index + 1}. ${role}` })
  ));
}

function renderGraph() {
  const svg = $("#memoryGraph");
  svg.replaceChildren();
  const nodes = data.memory_graph.nodes.slice(0, 16);
  const edges = data.memory_graph.edges.filter((edge) =>
    nodes.some((node) => node.id === edge.from) && nodes.some((node) => node.id === edge.to)
  );
  const positions = new Map(nodes.map((node, index) => {
    const angle = (Math.PI * 2 * index) / Math.max(nodes.length, 1);
    return [node.id, { x: 360 + Math.cos(angle) * 250, y: 160 + Math.sin(angle) * 110, node }];
  }));
  edges.forEach((edge) => {
    const a = positions.get(edge.from);
    const b = positions.get(edge.to);
    svg.append(svgEl("line", { x1: a.x, y1: a.y, x2: b.x, y2: b.y, stroke: "#9ab2aa" }));
  });
  positions.forEach(({ x, y, node }) => {
    svg.append(svgEl("circle", { cx: x, cy: y, r: 16, class: `node-${node.kind}` }));
    svg.append(svgEl("text", { x: x + 22, y: y + 4, fill: "#17201c" }, node.label));
  });
}

function svgEl(name, attrs, text) {
  const node = document.createElementNS("http://www.w3.org/2000/svg", name);
  Object.entries(attrs).forEach(([key, value]) => node.setAttribute(key, value));
  if (text) node.textContent = text;
  return node;
}

function renderSkills() {
  $("#skills").replaceChildren(...data.skills.map((skill) =>
    el("div", { class: "item" }, [
      el("strong", { text: skill.id }),
      el("small", { text: ` ${skill.version}` }),
      el("p", { text: skill.description }),
    ])
  ));
}

function renderPolicies() {
  const roleItems = data.policies.roles.map((role) =>
    el("div", { class: "item", text: `${role.name}: ${role.permissions} permissions` })
  );
  $("#policies").replaceChildren(
    el("div", { class: "item", text: `${data.policies.source_mode}: ${data.policies.source_path}` }),
    el("div", { class: "item", text: `default role: ${data.policies.default_role}` }),
    el("div", { class: "item", text: `runner: ${data.policies.runner_default}` }),
    ...roleItems,
  );
}

function renderReports() {
  $("#reports").replaceChildren(...data.reports.map((report) =>
    el("div", { class: "item" }, [
      el("a", { href: report.href, text: report.title }),
      el("small", { text: ` ${report.kind}` }),
    ])
  ));
}

renderHeader();
renderMetrics();
renderTransactions();
renderCost();
renderMetricsDashboard();
renderTimeline();
renderTrace();
renderGraph();
renderSkills();
renderPolicies();
renderReports();
