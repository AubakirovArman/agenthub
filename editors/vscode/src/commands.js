const vscode = require('vscode');
const childProcess = require('child_process');
const fs = require('fs');
const path = require('path');
const { renderDagHtml } = require('./dagView');
const { latestTxDir, timestamp, workspaceRoot } = require('./utils');

function registerCommands() {
  return [
    vscode.commands.registerCommand('agenthub.openLatestReport', openLatestReport),
    vscode.commands.registerCommand('agenthub.openMemory', openMemory),
    vscode.commands.registerCommand('agenthub.createSpecFromPrompt', createSpecFromPrompt),
    vscode.commands.registerCommand('agenthub.openDag', openDag)
  ];
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
  const txDir = latestTxDir();
  const dagPath = typeof filePath === 'string' ? filePath : txDir && path.join(txDir, 'dag.json');
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

module.exports = {
  registerCommands
};
