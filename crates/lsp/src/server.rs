//! The server loop: receives LSP messages and answers them.

use std::collections::HashMap;

use ekips_i18n::Lang;
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
    PublishDiagnostics,
};
use lsp_types::request::{Completion, HoverRequest, Request as _};
use lsp_types::{
    CompletionOptions, CompletionParams, CompletionResponse, HoverParams, HoverProviderCapability,
    InitializeParams, InitializeResult, PositionEncodingKind, PublishDiagnosticsParams,
    ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
};
use serde::Deserialize;

use crate::analysis::{self, CompletionOptions as AnalysisOptions};
use crate::position::{Encoding, LineIndex};

/// Settings that do not come from the editor.
#[derive(Debug, Clone, Copy, Default)]
pub struct Config {
    /// Language given on the command line (`--lang`) or in `EKIPS_LANG`
    pub lang: Option<Lang>,
}

/// Options the editor may send in `initializationOptions`.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitOptions {
    /// `"en"` or `"de"`
    lang: Option<String>,
}

struct Document {
    text: String,
    version: i32,
    index: LineIndex,
}

struct Server<'a> {
    connection: &'a Connection,
    lang: Lang,
    encoding: Encoding,
    snippets: bool,
    documents: HashMap<Uri, Document>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;

/// Runs the server on `connection` until the editor shuts it down.
pub fn run(connection: &Connection, config: Config) -> Result<(), Error> {
    let (id, params) = connection.initialize_start()?;
    let params: InitializeParams = serde_json::from_value(params)?;

    let init_options: InitOptions = params
        .initialization_options
        .clone()
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    // Priority: editor setting, then command line / environment, then the
    // editor's display language, then the system language.
    let lang = init_options
        .lang
        .as_deref()
        .and_then(ekips_i18n::parse_lang)
        .or(config.lang)
        .or_else(|| params.locale.as_deref().and_then(ekips_i18n::parse_lang))
        .unwrap_or_else(ekips_i18n::detect_lang);

    let utf8 = params
        .capabilities
        .general
        .as_ref()
        .and_then(|g| g.position_encodings.as_ref())
        .is_some_and(|encodings| encodings.contains(&PositionEncodingKind::UTF8));
    let encoding = if utf8 {
        Encoding::Utf8
    } else {
        Encoding::Utf16
    };

    let snippets = params
        .capabilities
        .text_document
        .as_ref()
        .and_then(|t| t.completion.as_ref())
        .and_then(|c| c.completion_item.as_ref())
        .and_then(|i| i.snippet_support)
        .unwrap_or(false);

    let result = InitializeResult {
        capabilities: ServerCapabilities {
            position_encoding: Some(match encoding {
                Encoding::Utf8 => PositionEncodingKind::UTF8,
                Encoding::Utf16 => PositionEncodingKind::UTF16,
            }),
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            completion_provider: Some(CompletionOptions {
                trigger_characters: Some(vec![".".to_string()]),
                ..CompletionOptions::default()
            }),
            ..ServerCapabilities::default()
        },
        server_info: Some(ServerInfo {
            name: "ekips-lsp".to_string(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }),
    };
    connection.initialize_finish(id, serde_json::to_value(result)?)?;

    let mut server = Server {
        connection,
        lang,
        encoding,
        snippets,
        documents: HashMap::new(),
    };
    for message in &connection.receiver {
        match message {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                server.request(req)?;
            }
            Message::Notification(note) => server.notification(note)?,
            Message::Response(_) => {}
        }
    }
    Ok(())
}

impl Server<'_> {
    fn request(&mut self, req: Request) -> Result<(), Error> {
        let response = match req.method.as_str() {
            HoverRequest::METHOD => {
                let params: HoverParams = serde_json::from_value(req.params)?;
                let pos = params.text_document_position_params;
                let hover = self.documents.get(&pos.text_document.uri).and_then(|doc| {
                    let offset = doc.index.offset(&doc.text, pos.position);
                    analysis::hover(&doc.text, offset, &doc.index, self.lang)
                });
                Response::new_ok(req.id, hover)
            }
            Completion::METHOD => {
                let params: CompletionParams = serde_json::from_value(req.params)?;
                let pos = params.text_document_position;
                let items = self
                    .documents
                    .get(&pos.text_document.uri)
                    .map(|doc| {
                        let offset = doc.index.offset(&doc.text, pos.position);
                        analysis::completion(
                            &doc.text,
                            offset,
                            &doc.index,
                            self.lang,
                            AnalysisOptions {
                                snippets: self.snippets,
                            },
                        )
                    })
                    .unwrap_or_default();
                Response::new_ok(req.id, CompletionResponse::Array(items))
            }
            _ => Response::new_err(
                req.id,
                ErrorCode::MethodNotFound as i32,
                format!("unsupported request: {}", req.method),
            ),
        };
        self.connection.sender.send(response.into())?;
        Ok(())
    }

    fn notification(&mut self, note: Notification) -> Result<(), Error> {
        match note.method.as_str() {
            DidOpenTextDocument::METHOD => {
                let params: lsp_types::DidOpenTextDocumentParams =
                    serde_json::from_value(note.params)?;
                let doc = params.text_document;
                self.update(doc.uri, doc.text, doc.version)?;
            }
            DidChangeTextDocument::METHOD => {
                let params: lsp_types::DidChangeTextDocumentParams =
                    serde_json::from_value(note.params)?;
                // Full sync: the last change holds the whole text.
                if let Some(change) = params.content_changes.into_iter().last() {
                    let doc = params.text_document;
                    self.update(doc.uri, change.text, doc.version)?;
                }
            }
            DidCloseTextDocument::METHOD => {
                let params: lsp_types::DidCloseTextDocumentParams =
                    serde_json::from_value(note.params)?;
                let uri = params.text_document.uri;
                self.documents.remove(&uri);
                // Remove the problems of the closed file from the editor.
                self.publish(uri, Vec::new(), None)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn update(&mut self, uri: Uri, text: String, version: i32) -> Result<(), Error> {
        let index = LineIndex::new(&text, self.encoding);
        let diagnostics = analysis::diagnostics(&text, &index, self.lang);
        self.documents.insert(
            uri.clone(),
            Document {
                text,
                version,
                index,
            },
        );
        let version = self.documents.get(&uri).map(|d| d.version);
        self.publish(uri, diagnostics, version)
    }

    fn publish(
        &self,
        uri: Uri,
        diagnostics: Vec<lsp_types::Diagnostic>,
        version: Option<i32>,
    ) -> Result<(), Error> {
        let params = PublishDiagnosticsParams {
            uri,
            diagnostics,
            version,
        };
        let note = Notification::new(PublishDiagnostics::METHOD.to_string(), params);
        self.connection.sender.send(note.into())?;
        Ok(())
    }
}
