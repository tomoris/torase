pub mod parser;
pub mod tokenizer;
pub mod type_checker;

use parser::{Ast, parse};
use tokenizer::{Range, add_eos_to_last_sentence, tokenize};
use type_checker::type_check;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub range: Range,
    pub message: String,
}

pub fn diagnose(text: String) -> Result<Vec<Ast>, Vec<Diagnostic>> {
    // tokenizing
    let tokenize_result = tokenize(&text);
    if let Err(e) = tokenize_result {
        let diagnostics = vec![Diagnostic {
            range: e.range,
            message: "tokenize failed".to_string(),
        }];
        return Err(diagnostics);
    }

    // parsing
    let sentences = add_eos_to_last_sentence(tokenize_result.unwrap());
    let mut asts = Vec::new();
    let mut diagnostics = Vec::new();
    for sentence in sentences {
        let parse_result = parse(sentence.clone());
        if let Err(e) = parse_result {
            let diagnostics_item = Diagnostic {
                range: e.error_token.range(),
                message: format!("parse error: {}", e.error_message),
            };
            diagnostics.push(diagnostics_item);
        } else {
            asts.push(parse_result.unwrap());
        }
    }

    // type checking
    for ast in &asts {
        let type_check_result = type_check(ast.clone());
        if let Err(e) = type_check_result {
            let diagnostics_item = Diagnostic {
                range: e.range,
                message: format!("type error: {}", e.message),
            };
            diagnostics.push(diagnostics_item);
        }
    }

    if !diagnostics.is_empty() {
        Err(diagnostics)
    } else {
        Ok(asts)
    }
}
