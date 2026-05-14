const vscode = require('vscode');
const fs = require('fs');
const path = require('path');

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

module.exports = {
  latestStatus,
  latestTxDir,
  timestamp,
  workspaceRoot
};
