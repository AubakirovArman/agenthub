const vscode = require('vscode');
const { registerCommands } = require('./src/commands');
const { MemoryProvider, TransactionsProvider } = require('./src/providers');

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
    ...registerCommands()
  );
}

function deactivate() {}

module.exports = {
  activate,
  deactivate
};
