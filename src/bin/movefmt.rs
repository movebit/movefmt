// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

// TODO the memory profiling not working,figure it out.
// Sometimes I want run profiling on my local machine.
use anyhow::Result;
use clap::Parser;
use crossbeam::channel::bounded;
use crossbeam::channel::select;
use crossbeam::channel::Sender;
use log::{Level, Metadata, Record};
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    notification::Notification as _, request::Request as _, CompletionOptions,
    HoverProviderCapability, OneOf, SaveOptions, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, TypeDefinitionProviderCapability, WorkDoneProgressOptions,
};

use movefmt::lsp_fmt;
use move_command_line_common::files::FileHash;
use move_compiler::{diagnostics::Diagnostics, shared::*, PASS_TYPING};

use std::sync::{Arc, Mutex};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use movefmt::{
    context::{Context, FileDiags},
    utils::*,
};

use url::Url;

struct SimpleLogger;
impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Error
    }
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            eprintln!("{} - {}", record.level(), record.args());
        }
    }
    fn flush(&self) {}
}
const LOGGER: SimpleLogger = SimpleLogger;

pub fn init_log() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Error))
        .unwrap()
}

#[derive(Parser)]
#[clap(author, version, about)]
struct Options {}

fn main() {
    #[cfg(feature = "pprof")]
    cpu_pprof(20);

    // For now, movefmt only responds to options built-in to clap,
    // such as `--help` or `--version`.
    Options::parse();
    init_log();
    // stdio is used to communicate Language Server Protocol requests and responses.
    // stderr is used for logging (and, when Visual Studio Code is used to communicate with this
    // server, it captures this output in a dedicated "output channel").
    let exe = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .to_string();
    log::error!(
        "Starting language server '{}' communicating via stdio...",
        exe
    );

    let (connection, io_threads) = Connection::stdio();
    let mut context = Context {
        connection,
        diag_version: FileDiags::new(),
    };

    let (id, _client_response) = context
        .connection
        .initialize_start()
        .expect("could not start connection initialization");

    let capabilities = serde_json::to_value(lsp_types::ServerCapabilities {
        // The server receives notifications from the client as users open, close,
        // and modify documents.
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                // TODO: We request that the language server client send us the entire text of any
                // files that are modified. We ought to use the "incremental" sync kind, which would
                // have clients only send us what has changed and where, thereby requiring far less
                // data be sent "over the wire." However, to do so, our language server would need
                // to be capable of applying deltas to its view of the client's open files. See the
                // 'movefmt::vfs' module for details.
                change: Some(TextDocumentSyncKind::FULL),
                will_save: None,
                will_save_wait_until: None,
                save: Some(
                    SaveOptions {
                        include_text: Some(true),
                    }
                    .into(),
                ),
            },
        )),
        selection_range_provider: None,
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        // The server provides completions as a user is typing.
        completion_provider: Some(CompletionOptions {
            resolve_provider: None,
            trigger_characters: Some({
                let mut c = vec![":".to_string(), ".".to_string()];
                for x in 'a'..='z' {
                    c.push(String::from(x as char));
                }
                for x in 'A'..='Z' {
                    c.push(String::from(x as char));
                }
                c.push(String::from("0"));
                c
            }),
            all_commit_characters: None,
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
            completion_item: None,
        }),
        definition_provider: Some(OneOf::Left(true)),
        type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
        references_provider: Some(OneOf::Left(true)),
        document_symbol_provider: Some(OneOf::Left(true)),
        inlay_hint_provider: Some(OneOf::Left(true)),
        code_lens_provider: Some(lsp_types::CodeLensOptions {
            resolve_provider: Some(true),
        }),
        document_formatting_provider: Some(OneOf::Left(true)),
        // semantic_tokens_provider: Some(
        //     lsp_types::SemanticTokensServerCapabilities::SemanticTokensOptions(
        //         lsp_types::SemanticTokensOptions {
        //             range: Some(true),
        //             full: None,
        //             ..Default::default()
        //         },
        //     ),
        // ),
        ..Default::default()
    })
    .expect("could not serialize server capabilities");
    context
        .connection
        .initialize_finish(
            id,
            serde_json::json!({
                "capabilities": capabilities,
            }),
        )
        .expect("could not finish connection initialization");
    let (diag_sender, diag_receiver) = bounded::<(PathBuf, Diagnostics)>(1);
    let diag_sender = Arc::new(Mutex::new(diag_sender));

    loop {
        select! {
            recv(diag_receiver) -> message => {
                match message {
                    Ok ((mani ,x)) => {
                        log::error!("IDE diag message error")
                    }
                    Err(error) => log::error!("IDE diag message error: {:?}", error),
                }
            }
            recv(context.connection.receiver) -> message => {
                match message {
                    Ok(Message::Request(request)) => on_request(&mut context, &request),
                    Ok(Message::Response(response)) => on_response(&context, &response),
                    Ok(Message::Notification(notification)) => {
                        match notification.method.as_str() {
                            lsp_types::notification::Exit::METHOD => break,
                            lsp_types::notification::Cancel::METHOD => {
                            }
                            _ => on_notification(&mut context, &notification ,diag_sender.clone()),
                        }
                    }
                    Err(error) => log::error!("IDE lsp client message error: {:?}", error),
                }
            }
        };
    }
    io_threads.join().expect("I/O threads could not finish");
    log::error!("Shut down language server '{}'.", exe);
}

fn on_request(context: &mut Context, request: &Request) {
    log::info!("receive method:{}", request.method.as_str());
    match request.method.as_str() {
        lsp_types::request::Formatting::METHOD => {
            lsp_fmt::on_fmt_request(context, request);
        }
        _ => log::error!("handle request '{}' from client", request.method),
    }
}

fn on_response(_context: &Context, _response: &Response) {
    log::error!("handle response from client");
}

type DiagSender = Arc<Mutex<Sender<(PathBuf, Diagnostics)>>>;

fn on_notification(context: &mut Context, notification: &Notification, diag_sender: DiagSender) {
    match notification.method.as_str() {
        lsp_types::notification::DidSaveTextDocument::METHOD => {
            use lsp_types::DidSaveTextDocumentParams;
            let parameters =
                serde_json::from_value::<DidSaveTextDocumentParams>(notification.params.clone())
                    .expect("could not deserialize DidSaveTextDocumentParams request");
            let fpath = parameters.text_document.uri.to_file_path().unwrap();
            let fpath = path_concat(&PathBuf::from(std::env::current_dir().unwrap()), &fpath);
            let content = std::fs::read_to_string(fpath.as_path());
            let content = match content {
                Ok(x) => x,
                Err(err) => {
                    log::error!("read file failed,err:{:?}", err);
                    return;
                }
            };
            make_diag(context, diag_sender, fpath);
        }
        _ => log::error!("handle notification '{}' from client", notification.method),
    }
}

fn get_package_compile_diagnostics(
    pkg_path: &Path,
) -> Result<Diagnostics> {
    use anyhow::*;
    use move_package::compilation::build_plan::BuildPlan;
    use tempfile::tempdir;
    let build_config = move_package::BuildConfig {
        test_mode: true,
        install_dir: Some(tempdir().unwrap().path().to_path_buf()),
        skip_fetch_latest_git_deps: true,
        ..Default::default()
    };
    // resolution graph diagnostics are only needed for CLI commands so ignore them by passing a
    // vector as the writer
    let resolution_graph = build_config.resolution_graph_for_package(pkg_path, &mut Vec::new())?;
    let build_plan = BuildPlan::create(resolution_graph)?;
    let mut diagnostics = None;
    let compile_cfg: move_package::CompilerConfig = Default::default();
    build_plan.compile_with_driver(&mut std::io::sink(), &compile_cfg,
        |compiler| { 
            let (_, compilation_result) = compiler.run::<PASS_TYPING>()?;
            match compilation_result {
                std::result::Result::Ok(_) => {},
                std::result::Result::Err(diags) => {
                    eprintln!("get_package_compile_diagnostics compilate failed");
                    diagnostics = Some(diags);
                },
            };
            Ok(Default::default())
        },
        |compiler| {
            Ok(Default::default())
        }
    )?;
    match diagnostics {
        Some(x) => Ok(x),
        None => Ok(Default::default()),
    }
}

fn make_diag(context: &Context, diag_sender: DiagSender, fpath: PathBuf) {
    let (mani, _) = match movefmt::utils::discover_manifest_and_kind(fpath.as_path()) {
        Some(x) => x,
        None => {
            log::error!("manifest not found.");
            return;
        }
    };
    std::thread::spawn(move || {
        let x = match get_package_compile_diagnostics(mani.as_path()) {
            Ok(x) => x,
            Err(err) => {
                log::error!("get_package_compile_diagnostics failed,err:{:?}", err);
                return;
            }
        };
        diag_sender.lock().unwrap().send((mani, x)).unwrap();
    });
}
