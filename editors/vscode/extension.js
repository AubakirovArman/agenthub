const vscode = require('vscode');
const fs = require('fs');
const path = require('path');
const childProcess = require('child_process');

function activate(context) {
  const txProvider = new TransactionsProvider();
  const memoryProvider = new MemoryProvider();

  context.subscriptions.push(
    vscode.window.registerTreeDataProvider('agenthub.transactions', txProvider),
    vscode.window.registerTreeDataProvider('agenthub.memory', memoryProvider),
    vscode.commands.registerCommand('agenthub.refresh', () => {
      txProvider.refresh();
      memoryProvider.refresh();
    }),
    vscode.commands.registerCommand('agenthub.openLatestReport', openLatestReport),
    vscode.commands.registerCommand('agenthub.openMemory', openMemory),
    vscode.commands.registerCommand('agenthub.createSpecFromPrompt', createSpecFromPrompt),
    vscode.commands.registerCommand('agenthub.openDag', openDag)
  );
}

function deactivate() {}

class TransactionsProvider {
  constructor() {
    this._onDidChangeTreeData = new vscode.EventEmitter();
    this.onDidChangeTreeData = this._onDidChangeTreeData.event;
  }

  refresh() {
    this._onDidChangeTreeData.fire();
  }

  getTreeItem(item) {
    return item;
  }

  getChildren(item) {
    const root = workspaceRoot();
    if (!root) {
      return [];
    }

    if (item && item.txDir) {
      return [
        fileItem('report.md', path.join(item.txDir, 'report.md')),
        fileItem('dag.json', path.join(item.txDir, 'dag.json'), 'agenthub.openDag'),
        fileItem('journal.jsonl', path.join(item.txDir, 'journal.jsonl')),
        fileItem('verifier.log', path.join(item.txDir, 'verifier.log'))
      ].filter((child) => fs.existsSync(child.filePath));
    }

    const txRoot = path.join(root, '.agent', 'tx');
    if (!fs.existsSync(txRoot)) {
      return [];
    }

    return fs.readdirSync(txRoot, { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => {
        const txDir = path.join(txRoot, entry.name);
        const treeItem = new vscode.TreeItem(entry.name, vscode.TreeItemCollapsibleState.Collapsed);
        treeItem.txDir = txDir;
        treeItem.description = latestStatus(path.join(txDir, 'journal.jsonl'));
        treeItem.contextValue = 'agenthubTx';
        treeItem.command = {
          command: 'vscode.open',
          title: 'Open Report',
          arguments: [vscode.Uri.file(path.join(txDir, 'report.md'))]
        };
        return treeItem;
      })
      .sort((a, b) => b.label.localeCompare(a.label));
  }
}

class MemoryProvider {
  constructor() {
    this._onDidChangeTreeData = new vscode.EventEmitter();
    this.onDidChangeTreeData = this._onDidChangeTreeData.event;
  }

  refresh() {
    this._onDidChangeTreeData.fire();
  }

  getTreeItem(item) {
    return item;
  }

  getChildren() {
    const root = workspaceRoot();
    if (!root) {
      return [];
    }

    const memoryRoot = path.join(root, '.agent', 'memory');
    return [
      fileItem('committed.jsonl', path.join(memoryRoot, 'committed.jsonl')),
      fileItem('failed_attempts.jsonl', path.join(memoryRoot, 'failed_attempts.jsonl')),
      fileItem('project_state.json', path.join(memoryRoot, 'compacted', 'project_state.json'))
    ].filter((item) => fs.existsSync(item.filePath));
  }
}

function fileItem(label, filePath, command) {
  const item = new vscode.TreeItem(label, vscode.TreeItemCollapsibleState.None);
  item.filePath = filePath;
  item.resourceUri = vscode.Uri.file(filePath);
  item.contextValue = 'agenthubFile';
  item.command = {
    command: command || 'vscode.open',
    title: 'Open',
    arguments: command ? [filePath] : [vscode.Uri.file(filePath)]
  };
  return item;
}

async function openLatestReport() {
  const txDir = latestTxDir();
  if (!txDir) {
    vscode.window.showInformationMessage('No AgentHub transactions found.');
    return;
  }
  await vscode.commands.executeCommand('vscode.open', vscode.Uri.file(path.join(txDir, 'report.md')));
}

async function openMemory() {
  const root = workspaceRoot();
  if (!root) {
    return;
  }

  const picks = [
    ['Committed Memory', path.join(root, '.agent', 'memory', 'committed.jsonl')],
    ['Failed Attempts', path.join(root, '.agent', 'memory', 'failed_attempts.jsonl')],
    ['Compacted Project State', path.join(root, '.agent', 'memory', 'compacted', 'project_state.json')]
  ].filter(([, file]) => fs.existsSync(file));

  const selected = await vscode.window.showQuickPick(
    picks.map(([label, file]) => ({ label, file })),
    { placeHolder: 'Open AgentHub memory artifact' }
  );
  if (selected) {
    await vscode.commands.executeCommand('vscode.open', vscode.Uri.file(selected.file));
  }
}

async function createSpecFromPrompt() {
  const root = workspaceRoot();
  if (!root) {
    return;
  }

  const prompt = await vscode.window.showInputBox({
    prompt: 'Describe the AgentHub task',
    placeHolder: 'Add /courses page in the current dashboard style'
  });
  if (!prompt) {
    return;
  }

  const yaml = await runAgentHubAsk(root, prompt);
  const specsDir = path.join(root, '.agent', 'specs');
  fs.mkdirSync(specsDir, { recursive: true });
  const specPath = path.join(specsDir, `preview-${timestamp()}.yaml`);
  fs.writeFileSync(specPath, yaml);
  await vscode.commands.executeCommand('vscode.open', vscode.Uri.file(specPath));
}

async function openDag(filePath) {
  const dagPath = typeof filePath === 'string'
    ? filePath
    : latestTxDir() && path.join(latestTxDir(), 'dag.json');
  if (!dagPath || !fs.existsSync(dagPath)) {
    vscode.window.showInformationMessage('No AgentHub DAG found.');
    return;
  }

  const dag = JSON.parse(fs.readFileSync(dagPath, 'utf8'));
  const panel = vscode.window.createWebviewPanel(
    'agenthubDag',
    `AgentHub DAG: ${dag.task_id || path.basename(path.dirname(dagPath))}`,
    vscode.ViewColumn.One,
    { enableScripts: false }
  );
  panel.webview.html = renderDagHtml(dag);
}

function runAgentHubAsk(root, prompt) {
  const executable = vscode.workspace.getConfiguration('agenthub').get('executable', 'agenthub');
  return execFile(executable, ['ask', prompt], root).catch(() => (
    execFile('cargo', ['run', '--quiet', '--', 'ask', prompt], root)
  ));
}

function execFile(command, args, cwd) {
  return new Promise((resolve, reject) => {
    childProcess.execFile(command, args, { cwd }, (error, stdout, stderr) => {
      if (error) {
        reject(new Error(stderr || error.message));
      } else {
        resolve(stdout);
      }
    });
  });
}

function renderDagHtml(dag) {
  const nodes = dag.nodes || [];
  const edges = dag.edges || [];
  const width = 920;
  const rowHeight = 88;
  const height = Math.max(160, nodes.length * rowHeight + 60);
  const nodeById = new Map(nodes.map((node, index) => [node.id, { ...node, index }]));
  const nodeSvg = nodes.map((node, index) => {
    const y = 30 + index * rowHeight;
    return `
      <g>
        <rect x="220" y="${y}" width="480" height="54" rx="6"></rect>
        <text x="240" y="${y + 22}" class="title">${escapeHtml(node.id)}</text>
        <text x="240" y="${y + 42}" class="label">${escapeHtml(node.kind)} - ${escapeHtml(node.label)}</text>
      </g>`;
  }).join('');
  const edgeSvg = edges.map((edge) => {
    const from = nodeById.get(edge.from);
    const to = nodeById.get(edge.to);
    if (!from || !to) {
      return '';
    }
    const y1 = 30 + from.index * rowHeight + 54;
    const y2 = 30 + to.index * rowHeight;
    return `<path d="M460 ${y1} L460 ${y2}" marker-end="url(#arrow)"></path>`;
  }).join('');

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

function latestTxDir() {
  const root = workspaceRoot();
  if (!root) {
    return undefined;
  }
  const txRoot = path.join(root, '.agent', 'tx');
  if (!fs.existsSync(txRoot)) {
    return undefined;
  }
  const dirs = fs.readdirSync(txRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(txRoot, entry.name))
    .sort()
    .reverse();
  return dirs[0];
}

function latestStatus(journalPath) {
  if (!fs.existsSync(journalPath)) {
    return 'UNKNOWN';
  }
  const lines = fs.readFileSync(journalPath, 'utf8').trim().split(/\r?\n/).filter(Boolean);
  if (lines.length === 0) {
    return 'UNKNOWN';
  }
  try {
    return JSON.parse(lines[lines.length - 1]).state || 'UNKNOWN';
  } catch (_) {
    return 'UNKNOWN';
  }
}

function workspaceRoot() {
  const folder = vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders[0];
  if (!folder) {
    vscode.window.showInformationMessage('Open a workspace folder to use AgentHub.');
    return undefined;
  }
  return folder.uri.fsPath;
}

function timestamp() {
  return new Date().toISOString().replace(/[-:]/g, '').replace(/\..+/, 'Z');
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
  activate,
  deactivate
};

