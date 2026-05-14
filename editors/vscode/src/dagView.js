function renderDagHtml(dag) {
  const nodes = dag.nodes || [];
  const edges = dag.edges || [];
  const width = 920;
  const rowHeight = 88;
  const height = Math.max(160, nodes.length * rowHeight + 60);
  const nodeById = new Map(nodes.map((node, index) => [node.id, { ...node, index }]));
  const nodeSvg = nodes.map((node, index) => renderNode(node, index, rowHeight)).join('');
  const edgeSvg = edges.map((edge) => renderEdge(edge, nodeById, rowHeight)).join('');

  return `<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    body { font-family: var(--vscode-font-family); color: var(--vscode-foreground); background: var(--vscode-editor-background); }
    h1 { font-size: 18px; font-weight: 600; }
    svg { width: 100%; height: auto; }
    rect { fill: var(--vscode-editorWidget-background); stroke: var(--vscode-editorWidget-border); }
    path { stroke: var(--vscode-foreground); stroke-width: 1.5; fill: none; opacity: 0.7; }
    text { fill: var(--vscode-foreground); }
    .title { font-size: 14px; font-weight: 600; }
    .label { font-size: 12px; opacity: 0.75; }
  </style>
</head>
<body>
  <h1>${escapeHtml(dag.task_id || 'AgentHub DAG')}</h1>
  <svg viewBox="0 0 ${width} ${height}">
    <defs>
      <marker id="arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
        <path d="M 0 0 L 10 5 L 0 10 z" fill="currentColor"></path>
      </marker>
    </defs>
    ${edgeSvg}
    ${nodeSvg}
  </svg>
</body>
</html>`;
}

function renderNode(node, index, rowHeight) {
  const y = 30 + index * rowHeight;
  return `
      <g>
        <rect x="220" y="${y}" width="480" height="54" rx="6"></rect>
        <text x="240" y="${y + 22}" class="title">${escapeHtml(node.id)}</text>
        <text x="240" y="${y + 42}" class="label">${escapeHtml(node.kind)} - ${escapeHtml(node.label)}</text>
      </g>`;
}

function renderEdge(edge, nodeById, rowHeight) {
  const from = nodeById.get(edge.from);
  const to = nodeById.get(edge.to);
  if (!from || !to) {
    return '';
  }
  const y1 = 30 + from.index * rowHeight + 54;
  const y2 = 30 + to.index * rowHeight;
  return `<path d="M460 ${y1} L460 ${y2}" marker-end="url(#arrow)"></path>`;
}

function escapeHtml(value) {
  return String(value)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

module.exports = {
  renderDagHtml
};
