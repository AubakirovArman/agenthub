const vscode = require('vscode');
const fs = require('fs');
const path = require('path');
const { latestStatus, workspaceRoot } = require('./utils');

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
      return transactionFiles(item.txDir);
    }

    const txRoot = path.join(root, '.agent', 'tx');
    if (!fs.existsSync(txRoot)) {
      return [];
    }

    return fs.readdirSync(txRoot, { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => transactionItem(txRoot, entry.name))
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

function transactionFiles(txDir) {
  return [
    fileItem('report.md', path.join(txDir, 'report.md')),
    fileItem('dag.json', path.join(txDir, 'dag.json'), 'agenthub.openDag'),
    fileItem('journal.jsonl', path.join(txDir, 'journal.jsonl')),
    fileItem('verifier.log', path.join(txDir, 'verifier.log'))
  ].filter((child) => fs.existsSync(child.filePath));
}

function transactionItem(txRoot, txId) {
  const txDir = path.join(txRoot, txId);
  const item = new vscode.TreeItem(txId, vscode.TreeItemCollapsibleState.Collapsed);
  item.txDir = txDir;
  item.description = latestStatus(path.join(txDir, 'journal.jsonl'));
  item.contextValue = 'agenthubTx';
  item.command = {
    command: 'vscode.open',
    title: 'Open Report',
    arguments: [vscode.Uri.file(path.join(txDir, 'report.md'))]
  };
  return item;
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

module.exports = {
  MemoryProvider,
  TransactionsProvider
};
