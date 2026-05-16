let selectedTransactionId = null;

window.renderTransactionViewer = function renderTransactionViewer() {
  const helpers = window.AgentHubRenderHelpers;
  const details = data.transaction_details || [];
  const tabs = document.querySelector("#transactionViewerTabs");
  const report = document.querySelector("#transactionReport");
  const diff = document.querySelector("#transactionDiff");
  const logs = document.querySelector("#transactionLogs");
  if (!helpers || !tabs || !report || !diff || !logs) return;
  if (details.length === 0) {
    tabs.replaceChildren(helpers.el("span", { class: "muted", text: "No transaction details" }));
    report.textContent = "";
    diff.textContent = "";
    logs.textContent = "";
    return;
  }
  if (!selectedTransactionId || !details.some((item) => item.tx_id === selectedTransactionId)) {
    selectedTransactionId = details[0].tx_id;
  }
  tabs.replaceChildren(...details.map((item) => tabButton(helpers, item)));
  const selected = details.find((item) => item.tx_id === selectedTransactionId) || details[0];
  report.textContent = selected.report_excerpt;
  diff.textContent = selected.diff_excerpt;
  logs.textContent = selected.logs_excerpt;
};

function tabButton(helpers, item) {
  const active = item.tx_id === selectedTransactionId ? " active" : "";
  const button = helpers.el("button", { class: `viewer-tab${active}`, type: "button" }, [
    helpers.el("strong", { text: item.tx_id }),
    helpers.el("span", { text: item.status }),
  ]);
  button.addEventListener("click", () => {
    selectedTransactionId = item.tx_id;
    window.renderTransactionViewer();
  });
  return button;
}
