mod compiler;

fn main() {
    // let tokens: Vec<compiler::tokenizer::TerminalType> = vec![
    //     compiler::tokenizer::TerminalType::BinaryPredicate(Some(
    //         compiler::tokenizer::BinaryPredicate {},
    //     )),
    //     compiler::tokenizer::TerminalType::Argument(Some(compiler::tokenizer::Argument {})),
    //     compiler::tokenizer::TerminalType::ArgumentPhraseSeparator(Some(
    //         compiler::tokenizer::ArgumentPhraseSeparator {},
    //     )),
    //     compiler::tokenizer::TerminalType::Argument(Some(compiler::tokenizer::Argument {})),
    //     compiler::tokenizer::TerminalType::Eos(Some(compiler::tokenizer::Eos {})),
    // ];
    let input = "binary_predicate argument argument_phrase_separator argument eos";
    let tokens = compiler::tokenizer::tokenize(input);
    println!(
        "Testing parse with tokens: {:?}",
        compiler::parser::parse(tokens)
    );
}
