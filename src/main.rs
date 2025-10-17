mod compiler;

// fn main() {
//     let input = "binary_predicate argument argument_phrase_separator argument eos";
//     let tokens = compiler::tokenizer::tokenize(input);
//     println!(
//         "Testing parse with tokens: {:?}",
//         compiler::parser::parse(tokens)
//     );
// }

use dashmap::DashMap;
use serde_json::Value;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use compiler::tokenizer::TerminalType;

#[derive(Debug)]
struct Backend {
    client: Client,
    document_map: DashMap<Url, String>,
}

pub const LEGEND_TYPE: &[SemanticTokenType] = &[
    SemanticTokenType::VARIABLE,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::KEYWORD,
];

struct TextDocumentItem<'a> {
    uri: Url,
    text: &'a str,
    version: Option<i32>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    ..Default::default()
                }),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["dummy.do_something".to_string()],
                    work_done_progress_options: Default::default(),
                }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                        SemanticTokensRegistrationOptions {
                            text_document_registration_options: {
                                TextDocumentRegistrationOptions {
                                    document_selector: Some(vec![DocumentFilter {
                                        language: Some("plaintext".to_string()),
                                        scheme: Some("file".to_string()),
                                        pattern: None,
                                    }]),
                                }
                            },
                            semantic_tokens_options: SemanticTokensOptions {
                                work_done_progress_options: WorkDoneProgressOptions::default(),
                                legend: SemanticTokensLegend {
                                    token_types: LEGEND_TYPE.into(),
                                    token_modifiers: vec![],
                                },
                                range: Some(false),
                                full: Some(SemanticTokensFullOptions::Bool(true)),
                            },
                            static_registration_options: StaticRegistrationOptions::default(),
                        },
                    ),
                ),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        self.client
            .log_message(MessageType::INFO, "workspace folders changed!")
            .await;
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        self.client
            .log_message(MessageType::INFO, "configuration changed!")
            .await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        self.client
            .log_message(MessageType::INFO, "watched files have changed!")
            .await;
    }

    async fn execute_command(&self, _: ExecuteCommandParams) -> Result<Option<Value>> {
        self.client
            .log_message(MessageType::INFO, "command executed!")
            .await;

        match self.client.apply_edit(WorkspaceEdit::default()).await {
            Ok(res) if res.applied => self.client.log_message(MessageType::INFO, "applied").await,
            Ok(_) => self.client.log_message(MessageType::INFO, "rejected").await,
            Err(err) => self.client.log_message(MessageType::ERROR, err).await,
        }

        Ok(None)
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: &params.text_document.text,
            version: Some(params.text_document.version),
        })
        .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: &params.content_changes[0].text,
            version: Some(params.text_document.version),
        })
        .await;
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file saved!")
            .await;
    }

    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
            CompletionItem::new_simple("aaaaaaaaaaa".to_string(), "More detail".to_string()),
        ])))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        let semantic_tokens = || -> Option<Vec<SemanticToken>> {
            let text = self.document_map.get(&uri)?;
            let tokenize_result = compiler::tokenizer::tokenize(&text);
            if tokenize_result.is_err() {
                return None;
            }
            let sentences = compiler::tokenizer::add_eos_to_last_sentence(tokenize_result.unwrap());
            let mut semantic_tokens = Vec::new();
            let mut pre_line = 0;
            let mut pre_start = 0;
            for sentence in sentences {
                let mut im_complete_tokens = sentence.clone();
                im_complete_tokens.sort_by_key(|t| t.start());
                let tmp_semantic_tokens = im_complete_tokens
                    .iter()
                    .filter_map(|token| {
                        let line = token.start().line() as u32;
                        let start = token.start().character() as u32;
                        let delta_line = line - pre_line;
                        let delta_start = if delta_line == 0 {
                            start - pre_start
                        } else {
                            start
                        };
                        let length = (token.end().diff_length(token.start())) as u32;
                        let semantic_token_type = get_semantic_token_type(token);
                        let ret = Some(SemanticToken {
                            delta_line,
                            delta_start,
                            length,
                            token_type: LEGEND_TYPE
                                .iter()
                                .position(|t| *t == semantic_token_type)
                                .unwrap() as u32,
                            token_modifiers_bitset: 0,
                        });
                        pre_line = line;
                        pre_start = start;
                        ret
                    })
                    .collect::<Vec<_>>();
                semantic_tokens.extend(tmp_semantic_tokens);
            }
            Some(semantic_tokens)
        }();
        if let Some(semantic_token) = semantic_tokens {
            return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: semantic_token,
            })));
        }
        Ok(None)
    }
}

impl Backend {
    async fn on_change<'a>(&self, params: TextDocumentItem<'a>) {
        dbg!(&params.version);
        let url = params.uri.clone();
        self.document_map
            .insert(url.clone(), params.text.to_string());
        self.diagnose(url.clone(), params.version).await;
        _ = self.client.semantic_tokens_refresh().await;
    }
    async fn diagnose(&self, url: Url, version: Option<i32>) {
        let text = self.document_map.get(&url).unwrap();

        // tokenizing
        let tokenize_result = compiler::tokenizer::tokenize(&text);
        if let Err(e) = tokenize_result {
            let diagnostics = vec![Diagnostic {
                range: Range {
                    start: Position {
                        line: e.start().line() as u32,
                        character: e.start().character() as u32,
                    },
                    end: Position {
                        line: e.end().line() as u32,
                        character: e.end().character() as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                code_description: None,
                source: None,
                message: "tokenize failed".to_string(),
                related_information: None,
                tags: None,
                data: None,
            }];
            self.client
                .publish_diagnostics(url, diagnostics, version)
                .await;
            return;
        }

        // parsing
        let sentences = compiler::tokenizer::add_eos_to_last_sentence(tokenize_result.unwrap());
        let mut asts = Vec::new();
        let mut diagnostics = Vec::new();
        for sentence in sentences {
            let parse_result = compiler::parser::parse(sentence.clone());
            if let Err(e) = parse_result {
                let diagnostics_item = Diagnostic {
                    range: Range {
                        start: Position {
                            line: e.error_token.start().line() as u32,
                            character: e.error_token.start().character() as u32,
                        },
                        end: Position {
                            line: e.error_token.end().line() as u32,
                            character: e.error_token.end().character() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: None,
                    message: format!("parse error: {}", e.error_message),
                    related_information: None,
                    tags: None,
                    data: None,
                };
                diagnostics.push(diagnostics_item);
            } else {
                asts.push(parse_result.unwrap());
            }
        }

        // type checking
        for ast in asts {
            let type_check_result = compiler::type_checker::type_check(ast);
            if let Err(e) = type_check_result {
                let diagnostics_item = Diagnostic {
                    range: Range {
                        start: Position {
                            line: e.range.start().line() as u32,
                            character: e.range.start().character() as u32,
                        },
                        end: Position {
                            line: e.range.end().line() as u32,
                            character: e.range.end().character() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: None,
                    message: format!("type error: {}", e.message),
                    related_information: None,
                    tags: None,
                    data: None,
                };
                diagnostics.push(diagnostics_item);
            }
        }

        self.client
            .publish_diagnostics(url, diagnostics, version)
            .await;
    }
}

fn get_semantic_token_type(token: &TerminalType) -> SemanticTokenType {
    match token {
        TerminalType::BinaryPredicate(_) => SemanticTokenType::FUNCTION,
        TerminalType::Argument(_) => SemanticTokenType::VARIABLE,
        TerminalType::ArgumentPhraseSeparator(_) => SemanticTokenType::KEYWORD,
        TerminalType::Eos(_) => SemanticTokenType::KEYWORD,
        _ => SemanticTokenType::KEYWORD,
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    let (service, socket) = LspService::new(|client| Backend {
        client,
        document_map: DashMap::new(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
