mod compiler;

fn main() {
    let tokens: Vec<compiler::parser::TerminalType> = vec![
        compiler::parser::TerminalType::BinaryPredicate(Some(compiler::parser::BinaryPredicate {})),
        compiler::parser::TerminalType::Argument(Some(compiler::parser::Argument {})),
        compiler::parser::TerminalType::ArgumentPhraseSeparator(Some(
            compiler::parser::ArgumentPhraseSeparator {},
        )),
        compiler::parser::TerminalType::Argument(Some(compiler::parser::Argument {})),
        compiler::parser::TerminalType::Eos(Some(compiler::parser::Eos {})),
    ];
    println!(
        "Testing parse with tokens: {:?}",
        compiler::parser::parse(tokens)
    );
}
