use core::panic;
use std::mem::discriminant;

use regex::Regex;

#[derive(Debug)]
pub struct IncompleteToken {
    pub range: Range,
}

impl IncompleteToken {
    pub fn start(&self) -> Position {
        self.range.start.clone()
    }
    pub fn end(&self) -> Position {
        self.range.end.clone()
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Vec<TerminalType>>, IncompleteToken> {
    let tokens = split_with_utf8_ranges(input);

    let mut doc = Vec::new();

    for token in &tokens {
        if InterrogativeMarker::is_match(token.0) {
            doc.push(TerminalType::InterrogativeMarker(InterrogativeMarker::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if Modality::is_match(token.0) {
            doc.push(TerminalType::Modality(Modality::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if Tense::is_match(token.0) {
            doc.push(TerminalType::Tense(Tense::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if Aspect::is_match(token.0) {
            doc.push(TerminalType::Aspect(Aspect::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if ModalityNegation::is_match(token.0) {
            doc.push(TerminalType::ModalityNegation(ModalityNegation::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if TenseNegation::is_match(token.0) {
            doc.push(TerminalType::TenseNegation(TenseNegation::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if AspectNegation::is_match(token.0) {
            doc.push(TerminalType::AspectNegation(AspectNegation::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if Negation::is_match(token.0) {
            doc.push(TerminalType::Negation(Negation::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if BinaryPredicate::is_match(token.0) {
            doc.push(TerminalType::BinaryPredicate(BinaryPredicate::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if Argument::is_match(token.0) {
            doc.push(TerminalType::Argument(Argument::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if ArgumentPhraseSeparator::is_match(token.0) {
            doc.push(TerminalType::ArgumentPhraseSeparator(
                ArgumentPhraseSeparator::new(token.0, token.1, token.2, token.3, token.4),
            ));
        } else if ArgumentPhraseTransposer::is_match(token.0) {
            doc.push(TerminalType::ArgumentPhraseTransposer(
                ArgumentPhraseTransposer::new(token.0, token.1, token.2, token.3, token.4),
            ));
        } else if BinaryOperator::is_match(token.0) {
            doc.push(TerminalType::BinaryOperator(BinaryOperator::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if ArgumentRetriever::is_match(token.0) {
            doc.push(TerminalType::ArgumentRetriever(ArgumentRetriever::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if Quantifier::is_match(token.0) {
            doc.push(TerminalType::Quantifier(Quantifier::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if Number::is_match(token.0) {
            doc.push(TerminalType::Number(Number::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if Determiner::is_match(token.0) {
            doc.push(TerminalType::Determiner(Determiner::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if MtaEventToArgumentPhraseConverter::is_match(token.0) {
            doc.push(TerminalType::MtaEventToArgumentPhraseConverter(
                MtaEventToArgumentPhraseConverter::new(token.0, token.1, token.2, token.3, token.4),
            ));
        } else if MtaEventToArgumentPhraseConverterEnd::is_match(token.0) {
            doc.push(TerminalType::MtaEventToArgumentPhraseConverterEnd(
                MtaEventToArgumentPhraseConverterEnd::new(
                    token.0, token.1, token.2, token.3, token.4,
                ),
            ));
        } else if PostMtaEventModifier::is_match(token.0) {
            doc.push(TerminalType::PostMtaEventModifier(
                PostMtaEventModifier::new(token.0, token.1, token.2, token.3, token.4),
            ));
        } else if PostBinaryPredicatePhraseModifier::is_match(token.0) {
            doc.push(TerminalType::PostBinaryPredicatePhraseModifier(
                PostBinaryPredicatePhraseModifier::new(token.0, token.1, token.2, token.3, token.4),
            ));
        } else if PostArgumentPhraseModifier::is_match(token.0) {
            doc.push(TerminalType::PostArgumentPhraseModifier(
                PostArgumentPhraseModifier::new(token.0, token.1, token.2, token.3, token.4),
            ));
        } else if Eos::is_match(token.0) {
            doc.push(TerminalType::Eos(Eos::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else {
            return Err(IncompleteToken {
                range: Range {
                    start: Position {
                        line: token.1,
                        character: token.2,
                    },
                    end: Position {
                        line: token.3,
                        character: token.4,
                    },
                },
            });
        }
    }

    let sentences = doc
        .split_inclusive(|x| {
            discriminant(&TerminalType::Eos(Eos::new_omitted_token())) == discriminant(x) // eosで文分割
        })
        .map(|chunk| chunk.to_vec())
        .collect();

    Ok(sentences)
}

// 文の最後にEosトークンがない場合に追加する
pub fn add_eos_to_last_sentence(sentences: Vec<Vec<TerminalType>>) -> Vec<Vec<TerminalType>> {
    let mut sentences = sentences.clone();
    if let Some(last_sentence) = sentences.last_mut()
        && let Some(last_token) = last_sentence.last()
        && discriminant(&TerminalType::Eos(Eos::new_omitted_token())) != discriminant(last_token)
    {
        let start = if let Some(last_token) = last_sentence.last() {
            last_token.end()
        } else {
            Position {
                line: 0,
                character: 0,
            }
        };
        let end = start.clone();
        last_sentence.push(TerminalType::Eos(Eos {
            range: Range { start, end },
        }));
    }
    sentences
}

fn split_with_utf8_ranges(input: &str) -> Vec<(&str, usize, usize, usize, usize)> {
    let mut result = Vec::new();
    let mut in_word = false;
    let mut start_idx = 0;
    let mut start_row = 0;
    let mut start_col = 0;

    let mut row = 0;
    let mut col = 0;

    for (i, c) in input.char_indices() {
        if c.is_whitespace() {
            if in_word {
                result.push((&input[start_idx..i], start_row, start_col, row, col));
                in_word = false;
            }

            if c == '\n' {
                row += 1;
                col = 0;
            } else {
                col += 1;
            }
        } else {
            if !in_word {
                start_idx = i;
                start_row = row;
                start_col = col;
                in_word = true;
            }
            col += 1;
        }
    }

    if in_word {
        result.push((&input[start_idx..], start_row, start_col, row, col));
    }

    result
}

// terminal types

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminalType {
    InterrogativeMarker(InterrogativeMarker),
    Modality(Modality),
    Tense(Tense),
    Aspect(Aspect),
    ModalityNegation(ModalityNegation),
    TenseNegation(TenseNegation),
    AspectNegation(AspectNegation),
    Negation(Negation),
    BinaryPredicate(BinaryPredicate),
    Argument(Argument),
    ArgumentPhraseSeparator(ArgumentPhraseSeparator),
    ArgumentPhraseTransposer(ArgumentPhraseTransposer),
    BinaryOperator(BinaryOperator),
    ArgumentRetriever(ArgumentRetriever),
    Quantifier(Quantifier),
    Number(Number),
    Determiner(Determiner),
    MtaEventToArgumentPhraseConverter(MtaEventToArgumentPhraseConverter),
    MtaEventToArgumentPhraseConverterEnd(MtaEventToArgumentPhraseConverterEnd),
    PostMtaEventModifier(PostMtaEventModifier),
    PostBinaryPredicatePhraseModifier(PostBinaryPredicatePhraseModifier),
    PostArgumentPhraseModifier(PostArgumentPhraseModifier),
    Eos(Eos),
}

#[derive(Clone, Debug)]
pub enum DummyTerminalType {
    InterrogativeMarker,
    Modality,
    Tense,
    Aspect,
    ModalityNegation,
    TenseNegation,
    AspectNegation,
    Negation,
    BinaryPredicate,
    Argument,
    ArgumentPhraseSeparator,
    ArgumentPhraseTransposer,
    BinaryOperator,
    ArgumentRetriever,
    Quantifier,
    Number,
    Determiner,
    MtaEventToArgumentPhraseConverter,
    MtaEventToArgumentPhraseConverterEnd,
    PostMtaEventModifier,
    PostBinaryPredicatePhraseModifier,
    PostArgumentPhraseModifier,
    Eos,
}

pub fn get_dummy_terminal_type(terminal: TerminalType) -> DummyTerminalType {
    match terminal {
        TerminalType::InterrogativeMarker(_) => DummyTerminalType::InterrogativeMarker,
        TerminalType::Modality(_) => DummyTerminalType::Modality,
        TerminalType::Tense(_) => DummyTerminalType::Tense,
        TerminalType::Aspect(_) => DummyTerminalType::Aspect,
        TerminalType::ModalityNegation(_) => DummyTerminalType::ModalityNegation,
        TerminalType::TenseNegation(_) => DummyTerminalType::TenseNegation,
        TerminalType::AspectNegation(_) => DummyTerminalType::AspectNegation,
        TerminalType::Negation(_) => DummyTerminalType::Negation,
        TerminalType::BinaryPredicate(_) => DummyTerminalType::BinaryPredicate,
        TerminalType::Argument(_) => DummyTerminalType::Argument,
        TerminalType::ArgumentPhraseSeparator(_) => DummyTerminalType::ArgumentPhraseSeparator,
        TerminalType::ArgumentPhraseTransposer(_) => DummyTerminalType::ArgumentPhraseTransposer,
        TerminalType::BinaryOperator(_) => DummyTerminalType::BinaryOperator,
        TerminalType::ArgumentRetriever(_) => DummyTerminalType::ArgumentRetriever,
        TerminalType::Quantifier(_) => DummyTerminalType::Quantifier,
        TerminalType::Number(_) => DummyTerminalType::Number,
        TerminalType::Determiner(_) => DummyTerminalType::Determiner,
        TerminalType::MtaEventToArgumentPhraseConverter(_) => {
            DummyTerminalType::MtaEventToArgumentPhraseConverter
        }
        TerminalType::MtaEventToArgumentPhraseConverterEnd(_) => {
            DummyTerminalType::MtaEventToArgumentPhraseConverterEnd
        }
        TerminalType::PostMtaEventModifier(_) => DummyTerminalType::PostMtaEventModifier,
        TerminalType::PostBinaryPredicatePhraseModifier(_) => {
            DummyTerminalType::PostBinaryPredicatePhraseModifier
        }
        TerminalType::PostArgumentPhraseModifier(_) => {
            DummyTerminalType::PostArgumentPhraseModifier
        }
        TerminalType::Eos(_) => DummyTerminalType::Eos,
    }
}

impl TerminalType {
    pub fn start(&self) -> Position {
        match self {
            TerminalType::InterrogativeMarker(marker) => marker.range.start.clone(),
            TerminalType::Modality(modality) => modality.range.start.clone(),
            TerminalType::Tense(tense) => tense.range.start.clone(),
            TerminalType::Aspect(aspect) => aspect.range.start.clone(),
            TerminalType::ModalityNegation(negation) => negation.range.start.clone(),
            TerminalType::TenseNegation(negation) => negation.range.start.clone(),
            TerminalType::AspectNegation(negation) => negation.range.start.clone(),
            TerminalType::Negation(negation) => negation.range.start.clone(),
            TerminalType::BinaryPredicate(predicate) => predicate.range.start.clone(),
            TerminalType::Argument(argument) => argument.range.start.clone(),
            TerminalType::ArgumentPhraseSeparator(separator) => separator.range.start.clone(),
            TerminalType::ArgumentPhraseTransposer(transposer) => transposer.range.start.clone(),
            TerminalType::BinaryOperator(operator) => operator.range.start.clone(),
            TerminalType::ArgumentRetriever(retriever) => retriever.range.start.clone(),
            TerminalType::Quantifier(quantifier) => quantifier.range.start.clone(),
            TerminalType::Number(number) => number.range.start.clone(),
            TerminalType::Determiner(determiner) => determiner.range.start.clone(),
            TerminalType::MtaEventToArgumentPhraseConverter(converter) => {
                converter.range.start.clone()
            }
            TerminalType::MtaEventToArgumentPhraseConverterEnd(converter_end) => {
                converter_end.range.start.clone()
            }
            TerminalType::PostMtaEventModifier(modifier) => modifier.range.start.clone(),
            TerminalType::PostBinaryPredicatePhraseModifier(modifier) => {
                modifier.range.start.clone()
            }
            TerminalType::PostArgumentPhraseModifier(modifier) => modifier.range.start.clone(),
            TerminalType::Eos(eos) => eos.range.start.clone(),
        }
    }
    pub fn end(&self) -> Position {
        match self {
            TerminalType::InterrogativeMarker(marker) => marker.range.end.clone(),
            TerminalType::Modality(modality) => modality.range.end.clone(),
            TerminalType::Tense(tense) => tense.range.end.clone(),
            TerminalType::Aspect(aspect) => aspect.range.end.clone(),
            TerminalType::ModalityNegation(negation) => negation.range.end.clone(),
            TerminalType::TenseNegation(negation) => negation.range.end.clone(),
            TerminalType::AspectNegation(negation) => negation.range.end.clone(),
            TerminalType::Negation(negation) => negation.range.end.clone(),
            TerminalType::BinaryPredicate(predicate) => predicate.range.end.clone(),
            TerminalType::Argument(argument) => argument.range.end.clone(),
            TerminalType::ArgumentPhraseSeparator(separator) => separator.range.end.clone(),
            TerminalType::ArgumentPhraseTransposer(transposer) => transposer.range.end.clone(),
            TerminalType::BinaryOperator(operator) => operator.range.end.clone(),
            TerminalType::ArgumentRetriever(retriever) => retriever.range.end.clone(),
            TerminalType::Quantifier(quantifier) => quantifier.range.end.clone(),
            TerminalType::Number(number) => number.range.end.clone(),
            TerminalType::Determiner(determiner) => determiner.range.end.clone(),
            TerminalType::MtaEventToArgumentPhraseConverter(converter) => {
                converter.range.end.clone()
            }
            TerminalType::MtaEventToArgumentPhraseConverterEnd(converter_end) => {
                converter_end.range.end.clone()
            }
            TerminalType::PostMtaEventModifier(modifier) => modifier.range.end.clone(),
            TerminalType::PostBinaryPredicatePhraseModifier(modifier) => modifier.range.end.clone(),
            TerminalType::PostArgumentPhraseModifier(modifier) => modifier.range.end.clone(),
            TerminalType::Eos(eos) => eos.range.end.clone(),
        }
    }
    pub fn range(&self) -> Range {
        Range {
            start: self.start(),
            end: self.end(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterrogativeMarker {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Modality {
    range: Range,
    kind: ModalityKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ModalityKind {
    Haka, // 命令、しなさい
    Haki, // 勧誘、しませんか
    Haku, // 許可、してよい
    Hame, // 事実性高い
    Hamo, // 事実性低い
    Hase, // 義務、すべき
    Haso, // 許可、してよい
    Hake, // 能力、できる
    Hako, // 可能、ありうる
    Hate, // 願望、したい
    Hato, // 意志、しよう
    Hape, // 必然
    Hapo, // 不可能
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tense {
    range: Range,
    kind: TenseKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TenseKind {
    Hita, // 過去
    Hiti, // 現在
    Hitu, // 未来
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Aspect {
    range: Range,
    kind: AspectKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AspectKind {
    Huda,   // 未然
    Hude,   // 開始
    Hudida, // 進行
    Hudidi, // 習慣
    Hudidu, // 反復
    Hudo,   // 終了
    Hudu,   // 事後
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModalityNegation {
    range: Range,
    kind: ModalityNegationKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ModalityNegationKind {
    Hane, // 否定
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TenseNegation {
    range: Range,
    kind: TenseNegationKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TenseNegationKind {
    Hine, // 否定
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AspectNegation {
    range: Range,
    kind: AspectNegationKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AspectNegationKind {
    Hune, // 否定
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Negation {
    range: Range,
    kind: NegationKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum NegationKind {
    Ne, // 否定
    No, // 正反対
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BinaryPredicate {
    range: Range,
    string: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Argument {
    range: Range,
    kind: ArgumentKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ArgumentKind {
    Ma, // 私たち(あなたを含む)
    Mi, // 私
    Mu, // 私たち(あなたを含まない)
    Me, // これ
    Ba, // あなた
    Bu, // あなたたち
    Be, // それ
    Ga, // 誰か(単数か複数か不明)
    Gi, // 彼、彼女
    Gu, // 彼ら
    Ge, // あれ
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgumentPhraseSeparator {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgumentPhraseTransposer {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BinaryOperator {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgumentRetriever {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quantifier {
    range: Range,
    kind: QuantifierKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum QuantifierKind {
    Ta, // 全ての
    Te, // 多くの
    Ti, // 半分の
    To, // 少しの
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Number {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Determiner {
    range: Range,
    kind: DeterminerKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DeterminerKind {
    Mo, // この
    Bo, // その
    Go, // あの
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MtaEventToArgumentPhraseConverter {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MtaEventToArgumentPhraseConverterEnd {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostMtaEventModifier {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostBinaryPredicatePhraseModifier {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostArgumentPhraseModifier {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Eos {
    range: Range,
}

impl InterrogativeMarker {
    fn is_match(string: &str) -> bool {
        string == "ku"
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        if !Self::is_match(string) {
            panic!("Invalid interrogative marker surface: {}", string);
        }
        InterrogativeMarker {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
        }
    }
    pub fn get_string(&self) -> String {
        "ku".to_string()
    }
}

impl Modality {
    fn is_match(string: &str) -> bool {
        let re_modality = Regex::new(r"^(?:ha)(?:k[aiu]|m[eo]|t[eo]|s[eo]|k[eo]|p[eo])$").unwrap();
        re_modality.is_match(string)
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "haka" => ModalityKind::Haka,
            "haki" => ModalityKind::Haki,
            "haku" => ModalityKind::Haku,
            "hame" => ModalityKind::Hame,
            "hamo" => ModalityKind::Hamo,
            "hase" => ModalityKind::Hase,
            "haso" => ModalityKind::Haso,
            "hake" => ModalityKind::Hake,
            "hako" => ModalityKind::Hako,
            "hate" => ModalityKind::Hate,
            "hato" => ModalityKind::Hato,
            "hape" => ModalityKind::Hape,
            "hapo" => ModalityKind::Hapo,
            _ => panic!("Invalid modality surface: {}", string),
        };
        Modality {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            kind,
        }
    }
    pub fn get_string(&self) -> String {
        match self.kind {
            ModalityKind::Haka => "haka".to_string(),
            ModalityKind::Haki => "haki".to_string(),
            ModalityKind::Haku => "haku".to_string(),
            ModalityKind::Hame => "hame".to_string(),
            ModalityKind::Hamo => "hamo".to_string(),
            ModalityKind::Hase => "hase".to_string(),
            ModalityKind::Haso => "haso".to_string(),
            ModalityKind::Hake => "hake".to_string(),
            ModalityKind::Hako => "hako".to_string(),
            ModalityKind::Hate => "hate".to_string(),
            ModalityKind::Hato => "hato".to_string(),
            ModalityKind::Hape => "hape".to_string(),
            ModalityKind::Hapo => "hapo".to_string(),
        }
    }
}

impl Tense {
    fn is_match(string: &str) -> bool {
        let re_tense = Regex::new(r"^(?:hi)(?:t[aiu])$").unwrap();
        re_tense.is_match(string)
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "hita" => TenseKind::Hita,
            "hiti" => TenseKind::Hiti,
            "hitu" => TenseKind::Hitu,
            _ => panic!("Invalid tense surface: {}", string),
        };
        Tense {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            kind,
        }
    }
    pub fn get_string(&self) -> String {
        match self.kind {
            TenseKind::Hita => "hita".to_string(),
            TenseKind::Hiti => "hiti".to_string(),
            TenseKind::Hitu => "hitu".to_string(),
        }
    }
}

impl Aspect {
    fn is_match(string: &str) -> bool {
        let re_aspect = Regex::new(r"^(?:(?:hu)(?:(?:d[aeou])|(?:did[aiu])))$").unwrap();
        re_aspect.is_match(string)
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "huda" => AspectKind::Huda,
            "hude" => AspectKind::Hude,
            "hudida" => AspectKind::Hudida,
            "hudidi" => AspectKind::Hudidi,
            "hudidu" => AspectKind::Hudidu,
            "hudo" => AspectKind::Hudo,
            "hudu" => AspectKind::Hudu,
            _ => panic!("Invalid aspect surface: {}", string),
        };
        Aspect {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            kind,
        }
    }
    pub fn get_string(&self) -> String {
        match self.kind {
            AspectKind::Huda => "huda".to_string(),
            AspectKind::Hude => "hude".to_string(),
            AspectKind::Hudida => "hudida".to_string(),
            AspectKind::Hudidi => "hudidi".to_string(),
            AspectKind::Hudidu => "hudidu".to_string(),
            AspectKind::Hudo => "hudo".to_string(),
            AspectKind::Hudu => "hudu".to_string(),
        }
    }
}

impl ModalityNegation {
    fn is_match(string: &str) -> bool {
        let re_modality_negation = Regex::new(r"^(?:ha)(?:n[eo])$").unwrap();
        re_modality_negation.is_match(string)
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "hane" => ModalityNegationKind::Hane,
            _ => panic!("Invalid modality negation surface: {}", string),
        };
        ModalityNegation {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            kind,
        }
    }
    pub fn get_string(&self) -> String {
        match self.kind {
            ModalityNegationKind::Hane => "hane".to_string(),
        }
    }
}

impl TenseNegation {
    fn is_match(string: &str) -> bool {
        let re_tense_negation = Regex::new(r"^(?:hi)(?:n[eo])$").unwrap();
        re_tense_negation.is_match(string)
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "hine" => TenseNegationKind::Hine,
            _ => panic!("Invalid tense negation surface: {}", string),
        };
        TenseNegation {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            kind,
        }
    }
    pub fn get_string(&self) -> String {
        match self.kind {
            TenseNegationKind::Hine => "hine".to_string(),
        }
    }
}

impl AspectNegation {
    fn is_match(string: &str) -> bool {
        let re_aspect_negation = Regex::new(r"^(?:hu)(?:n[eo])$").unwrap();
        re_aspect_negation.is_match(string)
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "hune" => AspectNegationKind::Hune,
            _ => panic!("Invalid tense negation surface: {}", string),
        };
        AspectNegation {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            kind,
        }
    }
    pub fn get_string(&self) -> String {
        match self.kind {
            AspectNegationKind::Hune => "hune".to_string(),
        }
    }
}

impl Negation {
    fn is_match(string: &str) -> bool {
        let re_negation = Regex::new(r"^(?:n[eo])$").unwrap();
        re_negation.is_match(string)
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "ne" => NegationKind::Ne,
            "no" => NegationKind::No,
            _ => panic!("Invalid tense negation surface: {}", string),
        };
        Negation {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            kind,
        }
    }
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }
    pub fn get_string(&self) -> String {
        match self.kind {
            NegationKind::Ne => "ne".to_string(),
            NegationKind::No => "no".to_string(),
        }
    }
}

impl BinaryPredicate {
    fn is_match(string: &str) -> bool {
        let re_binary_predicate =
            Regex::new(r"^(?:[bdgmnr][aeiuo]|[pk]a)(?:[ptkshbdgmnr][aeiuo])+$").unwrap();
        re_binary_predicate.is_match(string)
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        BinaryPredicate {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            string: string.to_string(),
        }
    }
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }
    pub(crate) fn get_argument_types(&self) -> (Vec<ArgumentType>, Vec<ArgumentType>) {
        // 名詞的な命題
        let b = Regex::new(r"^(?:b)").unwrap();
        let d = Regex::new(r"^(?:d)").unwrap();
        let n = Regex::new(r"^(?:n)").unwrap();
        let g = Regex::new(r"^(?:g)").unwrap();
        let m = Regex::new(r"^(?:m)").unwrap();
        let r = Regex::new(r"^(?:r)").unwrap();
        // 動詞的な命題
        let pa = Regex::new(r"^(?:pa)").unwrap();
        let pe = Regex::new(r"^(?:pe)").unwrap();
        let pi = Regex::new(r"^(?:pi)").unwrap();
        let po = Regex::new(r"^(?:po)").unwrap();
        let pu = Regex::new(r"^(?:pu)").unwrap();
        let ka = Regex::new(r"^(?:ka)").unwrap();
        let ki = Regex::new(r"^(?:ki)").unwrap();
        let ku = Regex::new(r"^(?:ku)").unwrap();

        if b.is_match(&self.string) {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::Epsilon],
            )
        } else if d.is_match(&self.string) {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact)],
                vec![ArgumentType::Epsilon],
            )
        } else if n.is_match(&self.string) {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::NOther)],
                vec![ArgumentType::Epsilon],
            )
        } else if g.is_match(&self.string) || m.is_match(&self.string) || r.is_match(&self.string) {
            (vec![ArgumentType::Abstraction], vec![ArgumentType::Epsilon])
        } else if pa.is_match(&self.string) {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
            )
        } else if pe.is_match(&self.string) {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![
                    ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                    ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
                ],
            )
        } else if pi.is_match(&self.string) {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::Epsilon],
            )
        } else if po.is_match(&self.string) {
            (
                vec![
                    ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                    ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
                ],
                vec![
                    ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                    ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
                ],
            )
        } else if pu.is_match(&self.string) {
            (
                vec![
                    ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                    ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
                ],
                vec![ArgumentType::Epsilon],
            )
        } else if ka.is_match(&self.string) {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::Abstraction],
            )
        } else if ki.is_match(&self.string) {
            (
                vec![
                    ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                    ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
                ],
                vec![ArgumentType::Abstraction],
            )
        } else if ku.is_match(&self.string) {
            (
                vec![ArgumentType::Abstraction],
                vec![ArgumentType::Abstraction],
            )
        } else {
            panic!("Invalid binary predicate surface: {}", self.string);
        }
    }
    pub fn get_string(&self) -> String {
        self.string.clone()
    }
}

impl Argument {
    fn is_match(string: &str) -> bool {
        let re_argument = Regex::new(r"^(:?(?:m[aiue])|(?:b[aue])|(?:g[aiue]))$").unwrap();
        re_argument.is_match(string)
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "ma" => ArgumentKind::Ma,
            "mi" => ArgumentKind::Mi,
            "mu" => ArgumentKind::Mu,
            "me" => ArgumentKind::Me,
            "ba" => ArgumentKind::Ba,
            "bu" => ArgumentKind::Bu,
            "be" => ArgumentKind::Be,
            "ga" => ArgumentKind::Ga,
            "gi" => ArgumentKind::Gi,
            "gu" => ArgumentKind::Gu,
            "ge" => ArgumentKind::Ge,
            _ => panic!("Invalid argument surface: {}", string),
        };
        Argument {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            kind,
        }
    }
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }
    pub(crate) fn get_type(&self) -> Vec<ArgumentType> {
        match self.kind {
            ArgumentKind::Ma => vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
            ArgumentKind::Mi => vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
            ArgumentKind::Mu => vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
            ArgumentKind::Me => vec![
                ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing),
                ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
                ArgumentType::Abstraction,
            ],
            ArgumentKind::Ba => vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
            ArgumentKind::Bu => vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
            ArgumentKind::Be => vec![
                ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing),
                ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
                ArgumentType::Abstraction,
            ],
            ArgumentKind::Ga => vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
            ArgumentKind::Gi => vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
            ArgumentKind::Gu => vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
            ArgumentKind::Ge => vec![
                ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing),
                ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
                ArgumentType::Abstraction,
            ],
        }
    }
    pub fn get_string(&self) -> String {
        match self.kind {
            ArgumentKind::Ma => "ma".to_string(),
            ArgumentKind::Mi => "mi".to_string(),
            ArgumentKind::Mu => "mu".to_string(),
            ArgumentKind::Me => "me".to_string(),
            ArgumentKind::Ba => "ba".to_string(),
            ArgumentKind::Bu => "bu".to_string(),
            ArgumentKind::Be => "be".to_string(),
            ArgumentKind::Ga => "ga".to_string(),
            ArgumentKind::Gi => "gi".to_string(),
            ArgumentKind::Gu => "gu".to_string(),
            ArgumentKind::Ge => "ge".to_string(),
        }
    }
}

impl ArgumentPhraseSeparator {
    fn is_match(string: &str) -> bool {
        string == "si"
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        if !Self::is_match(string) {
            panic!("Invalid argument phrase separator surface: {}", string);
        }
        ArgumentPhraseSeparator {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
        }
    }
    pub fn get_string(&self) -> String {
        "si".to_string()
    }
}

impl ArgumentPhraseTransposer {
    fn is_match(string: &str) -> bool {
        string == "re"
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        if !Self::is_match(string) {
            panic!("Invalid argument phrase transposer surface: {}", string);
        }
        ArgumentPhraseTransposer {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
        }
    }
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }
    pub fn get_string(&self) -> String {
        "re".to_string()
    }
}

impl BinaryOperator {
    fn is_match(string: &str) -> bool {
        string == "binary_operator"
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        if !Self::is_match(string) {
            panic!("Invalid binary operator surface: {}", string);
        }
        BinaryOperator {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
        }
    }
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }
    pub fn get_string(&self) -> String {
        "binary_operator".to_string()
    }
}

impl ArgumentRetriever {
    fn is_match(string: &str) -> bool {
        string == "ro"
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        if !Self::is_match(string) {
            panic!("Invalid argument retriever surface: {}", string);
        }
        ArgumentRetriever {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
        }
    }
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }
    pub fn get_string(&self) -> String {
        "ro".to_string()
    }
}

impl Quantifier {
    fn is_match(string: &str) -> bool {
        let re_quantifier = Regex::new(r"^(?:t[aeio])$").unwrap();
        re_quantifier.is_match(string)
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "ta" => QuantifierKind::Ta,
            "te" => QuantifierKind::Te,
            "ti" => QuantifierKind::Ti,
            "to" => QuantifierKind::To,
            _ => panic!("Invalid quantifier surface: {}", string),
        };
        Quantifier {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            kind,
        }
    }
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }
    pub fn get_string(&self) -> String {
        match self.kind {
            QuantifierKind::Ta => "ta".to_string(),
            QuantifierKind::Te => "te".to_string(),
            QuantifierKind::Ti => "ti".to_string(),
            QuantifierKind::To => "to".to_string(),
        }
    }
}

impl Number {
    fn is_match(string: &str) -> bool {
        string == "number"
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        if !Self::is_match(string) {
            panic!("Invalid number surface: {}", string);
        }
        Number {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
        }
    }
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }
    pub fn get_string(&self) -> String {
        "number".to_string()
    }
}

impl Determiner {
    fn is_match(string: &str) -> bool {
        let re_determiner = Regex::new(r"^(?:[mbg]o)$").unwrap();
        re_determiner.is_match(string)
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "mo" => DeterminerKind::Mo,
            "bo" => DeterminerKind::Bo,
            "go" => DeterminerKind::Go,
            _ => panic!("Invalid determiner surface: {}", string),
        };
        Determiner {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            kind,
        }
    }
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }
    pub fn get_string(&self) -> String {
        match self.kind {
            DeterminerKind::Mo => "mo".to_string(),
            DeterminerKind::Bo => "bo".to_string(),
            DeterminerKind::Go => "go".to_string(),
        }
    }
}

impl MtaEventToArgumentPhraseConverter {
    fn is_match(string: &str) -> bool {
        string == "se"
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        if !Self::is_match(string) {
            panic!(
                "Invalid mta event to argument phrase converter surface: {}",
                string
            );
        }
        MtaEventToArgumentPhraseConverter {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
        }
    }
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }
    pub fn get_string(&self) -> String {
        "se".to_string()
    }
}

impl MtaEventToArgumentPhraseConverterEnd {
    fn is_match(string: &str) -> bool {
        string == "so"
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        if !Self::is_match(string) {
            panic!(
                "Invalid mta event to argument phrase converter end surface: {}",
                string
            );
        }
        MtaEventToArgumentPhraseConverterEnd {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
        }
    }
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }
    pub fn get_string(&self) -> String {
        "so".to_string()
    }
}

impl PostMtaEventModifier {
    fn is_match(string: &str) -> bool {
        string == "da"
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        if !Self::is_match(string) {
            panic!("Invalid post mta event modifier surface: {}", string);
        }
        PostMtaEventModifier {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
        }
    }
    pub fn get_string(&self) -> String {
        "da".to_string()
    }
}

impl PostBinaryPredicatePhraseModifier {
    fn is_match(string: &str) -> bool {
        string == "di"
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        if !Self::is_match(string) {
            panic!(
                "Invalid post binary predicate phrase modifier surface: {}",
                string
            );
        }
        PostBinaryPredicatePhraseModifier {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
        }
    }
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }
    pub fn get_string(&self) -> String {
        "di".to_string()
    }
}

impl PostArgumentPhraseModifier {
    fn is_match(string: &str) -> bool {
        string == "du"
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        if !Self::is_match(string) {
            panic!("Invalid post argument phrase modifier surface: {}", string);
        }
        PostArgumentPhraseModifier {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
        }
    }
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }
    pub fn get_string(&self) -> String {
        "du".to_string()
    }
}

impl Eos {
    pub fn new_omitted_token() -> Self {
        Eos {
            range: Range {
                start: Position {
                    line: usize::MAX,
                    character: usize::MAX,
                },
                end: Position {
                    line: usize::MAX,
                    character: usize::MAX,
                },
            },
        }
    }
    fn is_match(string: &str) -> bool {
        string == "eos"
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        if !Self::is_match(string) {
            panic!("Invalid eos surface: {}", string);
        }
        Eos {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Range {
    start: Position,
    end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Range { start, end }
    }
    pub fn start(&self) -> Position {
        self.start.clone()
    }
    pub fn end(&self) -> Position {
        self.end.clone()
    }
}

// 並び順は、フィールドの順番に従う。
// つまり、lineが同じならcharacterの順番に従う。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    line: usize,
    character: usize,
}

impl Position {
    pub fn line(&self) -> usize {
        self.line
    }
    pub fn character(&self) -> usize {
        self.character
    }
    pub fn diff_length(&self, other: Position) -> usize {
        if self.line == other.line {
            self.character.abs_diff(other.character)
        } else {
            0 // 異なる行の場合は0を返す（意味のある差分を計算できないため）
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArgumentType {
    PhysicalEntity(PhysicalEntity),
    Abstraction,
    Epsilon,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicalEntity {
    BLivingThing,
    DArtifact,
    NOther,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_with_utf8_ranges() {
        let input = "binary_predicate argument argument_phrase_separator argument eos\nbinary_predicate argument argument_phrase_separator argument eos";
        let tokens = split_with_utf8_ranges(input);
        assert_eq!(tokens.len(), 10);
        assert_eq!(tokens[0], ("binary_predicate", 0, 0, 0, 16));
        assert_eq!(tokens[1], ("argument", 0, 17, 0, 25));
        assert_eq!(tokens[2], ("argument_phrase_separator", 0, 26, 0, 51));
        assert_eq!(tokens[3], ("argument", 0, 52, 0, 60));
        assert_eq!(tokens[4], ("eos", 0, 61, 0, 64));
        assert_eq!(tokens[5], ("binary_predicate", 1, 0, 1, 16));
        assert_eq!(tokens[6], ("argument", 1, 17, 1, 25));
        assert_eq!(tokens[7], ("argument_phrase_separator", 1, 26, 1, 51));
        assert_eq!(tokens[8], ("argument", 1, 52, 1, 60));
        assert_eq!(tokens[9], ("eos", 1, 61, 1, 64));
    }

    #[test]
    fn test_1文のトークン分割が成功する() {
        let input = "parobe ma si ba eos";
        let sentences = tokenize(input).unwrap();
        let sentences = add_eos_to_last_sentence(sentences);
        assert_eq!(sentences.len(), 1);
        assert_eq!(sentences[0].len(), 5);
        assert_eq!(
            sentences[0][0],
            TerminalType::BinaryPredicate(BinaryPredicate::new("parobe", 0, 0, 0, 6))
        );
        assert_eq!(
            sentences[0][1],
            TerminalType::Argument(Argument::new("ma", 0, 7, 0, 9))
        );
        assert_eq!(
            sentences[0][2],
            TerminalType::ArgumentPhraseSeparator(ArgumentPhraseSeparator::new("si", 0, 10, 0, 12))
        );
        assert_eq!(
            sentences[0][3],
            TerminalType::Argument(Argument::new("ba", 0, 13, 0, 15))
        );
        assert_eq!(
            sentences[0][4],
            TerminalType::Eos(Eos::new("eos", 0, 16, 0, 19))
        );
    }

    #[test]
    fn test_eosなし1文のトークン分割が成功する() {
        let input = "hate hitu hudidi parobe me si gi";
        let sentences = tokenize(input).unwrap();
        let sentences = add_eos_to_last_sentence(sentences);
        assert_eq!(sentences.len(), 1);
        assert_eq!(sentences[0].len(), 8);
        assert_eq!(
            sentences[0][0],
            TerminalType::Modality(Modality::new("hate", 0, 0, 0, 4))
        );
        assert_eq!(
            sentences[0][1],
            TerminalType::Tense(Tense::new("hitu", 0, 5, 0, 9))
        );
        assert_eq!(
            sentences[0][2],
            TerminalType::Aspect(Aspect::new("hudidi", 0, 10, 0, 16))
        );
        assert_eq!(
            sentences[0][3],
            TerminalType::BinaryPredicate(BinaryPredicate::new("parobe", 0, 17, 0, 23))
        );
        assert_eq!(
            sentences[0][4],
            TerminalType::Argument(Argument::new("me", 0, 24, 0, 26))
        );
        assert_eq!(
            sentences[0][5],
            TerminalType::ArgumentPhraseSeparator(ArgumentPhraseSeparator {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 27
                    },
                    end: Position {
                        line: 0,
                        character: 29
                    },
                }
            })
        );
        assert_eq!(
            sentences[0][6],
            TerminalType::Argument(Argument::new("gi", 0, 30, 0, 32))
        );
    }

    #[test]
    fn test_2文のトークン分割が成功する() {
        let input = "parobe me si ba eos ro parobe bu si gu";
        let sentences = tokenize(input).unwrap();
        let sentences = add_eos_to_last_sentence(sentences);
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].len(), 5);
        assert_eq!(sentences[1].len(), 6);
    }
}
