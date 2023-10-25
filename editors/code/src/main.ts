// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0


import { Configuration } from './configuration';
import { Context } from './context';
import { Extension } from './extension';
import { log } from './log';
import { Reg } from './reg';

import * as vscode from 'vscode';


/**
 * The entry point to this VS Code extension.
 *
 * As per [the VS Code documentation on activation
 * events](https://code.visualstudio.com/api/references/activation-events), 'an extension must
 * export an `activate()` function from its main module and it will be invoked only once by
 * VS Code when any of the specified activation events [are] emitted."
 *
 * Activation events for this extension are listed in its `package.json` file, under the key
 * `"activationEvents"`.
 *
 * In order to achieve synchronous activation, mark the function as an asynchronous function,
 * so that you can wait for the activation to complete by await
 */

export async function activate(
  extensionContext: Readonly<vscode.ExtensionContext>,
): Promise<void> {
  const extension = new Extension();
  log.info(`${extension.identifier} version ${extension.version}`);

  const configuration = new Configuration();
  log.info(`configuration: ${configuration.toString()}`);

  const context = Context.create(extensionContext, configuration);
  // An error here -- for example, if the path to the `move-analyzer` binary that the user
  // specified in their settings is not valid -- prevents the extension from providing any
  // more utility, so return early.
  if (context instanceof Error) {
    void vscode.window.showErrorMessage(
      `Could not activate move-analyzer: ${context.message}.`,
    );
    return;
  }
  context.registerCommand('goto_definition', async (_context, ...args) => {
    const loc = args[0] as { range: vscode.Range; fpath: string };
    const t = await vscode.workspace.openTextDocument(loc.fpath);
    await vscode.window.showTextDocument(t, { selection: loc.range, preserveFocus: false });
  });

  const d = vscode.languages.registerInlayHintsProvider({ scheme: 'file', language: 'move' },
    {
      provideInlayHints(document, range) {
        const client = context.getClient();
        if (client === undefined) {
          return undefined;
        }
        const hints = client.sendRequest<vscode.InlayHint[]>('textDocument/inlayHint',
          { range: range, textDocument: { uri: document.uri.toString() } });
        return hints;
      },
    });
  extensionContext.subscriptions.push(d);
  // Configure other language features.
  context.configureLanguage();

  // All other utilities provided by this extension occur via the language server.
  await context.startClient();

  // Regist all the sui commands.
  Reg.regsui(context);


  // send inlay hints 
  const reload_inlay_hints = function () {
    const client = context.getClient();
    if (client !== undefined) {
      void client.sendRequest('move/lsp/client/inlay_hints/config', configuration.inlay_hints_config());
    }
  };
  reload_inlay_hints();
  vscode.workspace.onDidChangeConfiguration(() => { log.info("reload_inlay_hints ...  "); reload_inlay_hints(); });
}


