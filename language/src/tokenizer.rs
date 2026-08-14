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
        } else if ModalityBase::is_match(token.0) {
            doc.push(TerminalType::ModalityBase(ModalityBase::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if ModalityForce::is_match(token.0) {
            doc.push(TerminalType::ModalityForce(ModalityForce::new(
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
        } else if CaseLabel::is_match(token.0) {
            doc.push(TerminalType::CaseLabel(CaseLabel::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if TypeOverride::is_match(token.0) {
            doc.push(TerminalType::TypeOverride(TypeOverride::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if Argument::is_match(token.0) {
            doc.push(TerminalType::Argument(Argument::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if PredicateOperator::is_match(token.0) {
            doc.push(TerminalType::PredicateOperator(PredicateOperator::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if ArgumentOperator::is_match(token.0) {
            doc.push(TerminalType::ArgumentOperator(ArgumentOperator::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if MtaEventOperator::is_match(token.0) {
            doc.push(TerminalType::MtaEventOperator(MtaEventOperator::new(
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
    ModalityBase(ModalityBase),
    ModalityForce(ModalityForce),
    Tense(Tense),
    Aspect(Aspect),
    ModalityNegation(ModalityNegation),
    TenseNegation(TenseNegation),
    AspectNegation(AspectNegation),
    Negation(Negation),
    BinaryPredicate(BinaryPredicate),
    Argument(Argument),
    CaseLabel(CaseLabel),
    TypeOverride(TypeOverride),
    PredicateOperator(PredicateOperator),
    ArgumentOperator(ArgumentOperator),
    MtaEventOperator(MtaEventOperator),
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
    ModalityBase,
    ModalityForce,
    Tense,
    Aspect,
    ModalityNegation,
    TenseNegation,
    AspectNegation,
    Negation,
    BinaryPredicate,
    Argument,
    CaseLabel,
    TypeOverride,
    PredicateOperator,
    ArgumentOperator,
    MtaEventOperator,
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
        TerminalType::ModalityBase(_) => DummyTerminalType::ModalityBase,
        TerminalType::ModalityForce(_) => DummyTerminalType::ModalityForce,
        TerminalType::Tense(_) => DummyTerminalType::Tense,
        TerminalType::Aspect(_) => DummyTerminalType::Aspect,
        TerminalType::ModalityNegation(_) => DummyTerminalType::ModalityNegation,
        TerminalType::TenseNegation(_) => DummyTerminalType::TenseNegation,
        TerminalType::AspectNegation(_) => DummyTerminalType::AspectNegation,
        TerminalType::Negation(_) => DummyTerminalType::Negation,
        TerminalType::BinaryPredicate(_) => DummyTerminalType::BinaryPredicate,
        TerminalType::Argument(_) => DummyTerminalType::Argument,
        TerminalType::CaseLabel(_) => DummyTerminalType::CaseLabel,
        TerminalType::TypeOverride(_) => DummyTerminalType::TypeOverride,
        TerminalType::PredicateOperator(_) => DummyTerminalType::PredicateOperator,
        TerminalType::ArgumentOperator(_) => DummyTerminalType::ArgumentOperator,
        TerminalType::MtaEventOperator(_) => DummyTerminalType::MtaEventOperator,
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
            TerminalType::ModalityBase(modality) => modality.range.start.clone(),
            TerminalType::ModalityForce(force) => force.range.start.clone(),
            TerminalType::Tense(tense) => tense.range.start.clone(),
            TerminalType::Aspect(aspect) => aspect.range.start.clone(),
            TerminalType::ModalityNegation(negation) => negation.range.start.clone(),
            TerminalType::TenseNegation(negation) => negation.range.start.clone(),
            TerminalType::AspectNegation(negation) => negation.range.start.clone(),
            TerminalType::Negation(negation) => negation.range.start.clone(),
            TerminalType::BinaryPredicate(predicate) => predicate.range.start.clone(),
            TerminalType::Argument(argument) => argument.range.start.clone(),
            TerminalType::CaseLabel(label) => label.range.start.clone(),
            TerminalType::TypeOverride(type_override) => type_override.range.start.clone(),
            TerminalType::PredicateOperator(operator) => operator.range.start.clone(),
            TerminalType::ArgumentOperator(operator) => operator.range.start.clone(),
            TerminalType::MtaEventOperator(operator) => operator.range.start.clone(),
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
            TerminalType::ModalityBase(modality) => modality.range.end.clone(),
            TerminalType::ModalityForce(force) => force.range.end.clone(),
            TerminalType::Tense(tense) => tense.range.end.clone(),
            TerminalType::Aspect(aspect) => aspect.range.end.clone(),
            TerminalType::ModalityNegation(negation) => negation.range.end.clone(),
            TerminalType::TenseNegation(negation) => negation.range.end.clone(),
            TerminalType::AspectNegation(negation) => negation.range.end.clone(),
            TerminalType::Negation(negation) => negation.range.end.clone(),
            TerminalType::BinaryPredicate(predicate) => predicate.range.end.clone(),
            TerminalType::Argument(argument) => argument.range.end.clone(),
            TerminalType::CaseLabel(label) => label.range.end.clone(),
            TerminalType::TypeOverride(type_override) => type_override.range.end.clone(),
            TerminalType::PredicateOperator(operator) => operator.range.end.clone(),
            TerminalType::ArgumentOperator(operator) => operator.range.end.clone(),
            TerminalType::MtaEventOperator(operator) => operator.range.end.clone(),
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
pub struct ModalityBase {
    range: Range,
    kind: ModalityBaseKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ModalityBaseKind {
    Hapa, // 認識
    Hapi, // 義務
    Hapu, // 能力・状況
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
pub struct ModalityForce {
    range: Range,
    kind: ModalityForceKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ModalityForceKind {
    Necessary, // □ 必然
    Possible,  // ◇ 可能
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
pub struct CaseLabel {
    range: Range,
    kind: CaseLabelKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaseLabelKind {
    Sa, // 主格
    Si, // 対格
    // 生物系
    Subu, // 随伴格 だれと
    Sube, // 奪格(人間起点格) だれから、だれの
    Subo, // 与格(人間目標格) だれまで、だれに
    // 空間系
    Suni, // 空間格 どこで
    Sune, // 空間起点格 どこから
    Suno, // 空間目標格 どこまで
    // 時間系
    Sugi, // 時間格 いつ
    Suge, // 時間起点格 いつから
    Sugo, // 時間目標格 いつまで
    // 道具系
    Sudu, // 具格 何々を使って
    // 抽象系
    Sute, // 原因格(理由起点格) 何々のせいで
    Suto, // 目的格(理由目標格) 何々のために
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeOverride {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PredicateOperator {
    range: Range,
    kind: PredicateOperatorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PredicateOperatorKind {
    And, // hoba
    Or,  // hobi
    Xor, // hobu
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgumentOperator {
    range: Range,
    kind: ArgumentOperatorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArgumentOperatorKind {
    And, // hoka
    Or,  // hoki
    Xor, // hoku
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MtaEventOperator {
    range: Range,
    kind: MtaEventOperatorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MtaEventOperatorKind {
    And, // homa
    Or,  // homi
    Xor, // homu
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgumentRetriever {
    range: Range,
    kind: ArgumentRetrieverKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArgumentRetrieverKind {
    Nominative, // ra
    Accusative, // ri
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
    value: u64,
    is_hex: bool,
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
    kind: PostMtaEventModifierKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PostMtaEventModifierKind {
    Da,       // 命題修飾句の前置詞
    Hesute,   // 接続詞(結果 -> 原因)
    Henesute, // 接続詞(結果 -> 反対の原因)
    Hesuto,   // 接続詞(原因 -> 結果)
    Henesuto, // 接続詞(反対の原因 -> 結果)
    Hehoma,   // 接続詞(and)
    Hehomi,   // 接続詞(or)
    Hehomu,   // 接続詞(xor)
    Hehita,   // 接続詞(時間前)
    Hehiti,   // 接続詞(同時)
    Hehitu,   // 接続詞(時間後)
    Hezu,     // 接続詞(条件)
    Hehema,   // 接続詞(例示)
    Hehegi,   // 接続詞(言い換え)
    Hehenu,   // 接続詞(対比)
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
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }
}

impl ModalityBase {
    fn is_match(string: &str) -> bool {
        matches!(string, "hapa" | "hapi" | "hapu")
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "hapa" => ModalityBaseKind::Hapa,
            "hapi" => ModalityBaseKind::Hapi,
            "hapu" => ModalityBaseKind::Hapu,
            _ => panic!("Invalid modality surface: {}", string),
        };
        ModalityBase {
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
            ModalityBaseKind::Hapa => "hapa".to_string(),
            ModalityBaseKind::Hapi => "hapi".to_string(),
            ModalityBaseKind::Hapu => "hapu".to_string(),
        }
    }
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }
}

impl ModalityForce {
    fn is_match(string: &str) -> bool {
        matches!(string, "□" | "◇" | "za" | "zi")
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "□" | "za" => ModalityForceKind::Necessary,
            "◇" | "zi" => ModalityForceKind::Possible,
            _ => panic!("Invalid modality force surface: {}", string),
        };
        ModalityForce {
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
            ModalityForceKind::Necessary => "□".to_string(),
            ModalityForceKind::Possible => "◇".to_string(),
        }
    }
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
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
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
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
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
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
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
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
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
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
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }
}

impl Negation {
    fn is_match(string: &str) -> bool {
        let re_negation = Regex::new(r"^(?:ne)$").unwrap();
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
        }
    }
}

impl BinaryPredicate {
    fn is_match(string: &str) -> bool {
        let re_binary_predicate =
            Regex::new(r"^(?:[bdgmnrpkz][aeiuo])(?:[ptkshbdgmnrz][aeiuo])+$").unwrap();
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
        // 空間・時間に関する名詞的な命題
        let za = Regex::new(r"^(?:za)").unwrap();
        let zi = Regex::new(r"^(?:zi)").unwrap();
        let zu = Regex::new(r"^(?:zu)").unwrap();
        let ze = Regex::new(r"^(?:ze)").unwrap();
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
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
            )
        } else if d.is_match(&self.string) {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact)],
            )
        } else if n.is_match(&self.string) {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::NOther)],
            )
        } else if r.is_match(&self.string) {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::Abstraction],
            )
        } else if za.is_match(&self.string) {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::Space],
            )
        } else if zi.is_match(&self.string) {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::Time],
            )
        } else if ze.is_match(&self.string) {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::Space],
            )
        } else if zu.is_match(&self.string) {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::Time],
            )
        } else if g.is_match(&self.string) || m.is_match(&self.string) || r.is_match(&self.string) {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::Abstraction],
            )
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
                abstraction_family_types(),
            )
        } else if ki.is_match(&self.string) {
            (
                vec![
                    ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                    ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
                ],
                abstraction_family_types(),
            )
        } else if ku.is_match(&self.string) {
            (abstraction_family_types(), abstraction_family_types())
        } else {
            panic!("Invalid binary predicate surface: {}", self.string);
        }
    }
    pub fn get_string(&self) -> String {
        self.string.clone()
    }
}

/// `ka`/`ki`/`ku`(生物・非生物が抽象概念を対象に取る動詞)が受け付ける抽象概念系や空間・時間系の型全体。
pub(crate) fn abstraction_family_types() -> Vec<ArgumentType> {
    vec![
        ArgumentType::Abstraction,
        ArgumentType::Space,
        ArgumentType::Time,
    ]
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
                ArgumentType::Space,
                ArgumentType::Time,
            ],
            ArgumentKind::Ba => vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
            ArgumentKind::Bu => vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
            ArgumentKind::Be => vec![
                ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing),
                ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
                ArgumentType::Abstraction,
                ArgumentType::Space,
                ArgumentType::Time,
            ],
            ArgumentKind::Ga => vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
            ArgumentKind::Gi => vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
            ArgumentKind::Gu => vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
            ArgumentKind::Ge => vec![
                ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing),
                ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
                ArgumentType::Abstraction,
                ArgumentType::Space,
                ArgumentType::Time,
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

impl CaseLabel {
    fn is_match(string: &str) -> bool {
        let re = Regex::new(r"^(?:sa|si|su(?:bu|be|bo|ni|ne|no|gi|ge|go|du|te|to))$").unwrap();
        re.is_match(string)
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "sa" => CaseLabelKind::Sa,
            "si" => CaseLabelKind::Si,
            "subu" => CaseLabelKind::Subu,
            "sube" => CaseLabelKind::Sube,
            "subo" => CaseLabelKind::Subo,
            "suni" => CaseLabelKind::Suni,
            "sune" => CaseLabelKind::Sune,
            "suno" => CaseLabelKind::Suno,
            "sugi" => CaseLabelKind::Sugi,
            "suge" => CaseLabelKind::Suge,
            "sugo" => CaseLabelKind::Sugo,
            "sudu" => CaseLabelKind::Sudu,
            "sute" => CaseLabelKind::Sute,
            "suto" => CaseLabelKind::Suto,
            _ => panic!("Invalid case label surface: {}", string),
        };
        CaseLabel {
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
            CaseLabelKind::Sa => "sa".to_string(),
            CaseLabelKind::Si => "si".to_string(),
            CaseLabelKind::Subu => "subu".to_string(),
            CaseLabelKind::Sube => "sube".to_string(),
            CaseLabelKind::Subo => "subo".to_string(),
            CaseLabelKind::Suni => "suni".to_string(),
            CaseLabelKind::Sune => "sune".to_string(),
            CaseLabelKind::Suno => "suno".to_string(),
            CaseLabelKind::Sugi => "sugi".to_string(),
            CaseLabelKind::Suge => "suge".to_string(),
            CaseLabelKind::Sugo => "sugo".to_string(),
            CaseLabelKind::Sudu => "sudu".to_string(),
            CaseLabelKind::Sute => "sute".to_string(),
            CaseLabelKind::Suto => "suto".to_string(),
        }
    }
    pub fn get_kind(&self) -> &CaseLabelKind {
        &self.kind
    }
}

impl TypeOverride {
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
            panic!("Invalid type override surface: {}", string);
        }
        TypeOverride {
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
        "se".to_string()
    }
}

impl PredicateOperator {
    fn is_match(string: &str) -> bool {
        matches!(string, "hoba" | "hobi" | "hobu")
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "hoba" => PredicateOperatorKind::And,
            "hobi" => PredicateOperatorKind::Or,
            "hobu" => PredicateOperatorKind::Xor,
            _ => panic!("Invalid predicate operator surface: {}", string),
        };
        PredicateOperator {
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
            PredicateOperatorKind::And => "hoba".to_string(),
            PredicateOperatorKind::Or => "hobi".to_string(),
            PredicateOperatorKind::Xor => "hobu".to_string(),
        }
    }
    pub fn get_kind(&self) -> &PredicateOperatorKind {
        &self.kind
    }
}

impl ArgumentOperator {
    fn is_match(string: &str) -> bool {
        matches!(string, "hoka" | "hoki" | "hoku")
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "hoka" => ArgumentOperatorKind::And,
            "hoki" => ArgumentOperatorKind::Or,
            "hoku" => ArgumentOperatorKind::Xor,
            _ => panic!("Invalid argument operator surface: {}", string),
        };
        ArgumentOperator {
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
            ArgumentOperatorKind::And => "hoka".to_string(),
            ArgumentOperatorKind::Or => "hoki".to_string(),
            ArgumentOperatorKind::Xor => "hoku".to_string(),
        }
    }
    pub fn get_kind(&self) -> &ArgumentOperatorKind {
        &self.kind
    }
}

impl MtaEventOperator {
    fn is_match(string: &str) -> bool {
        matches!(string, "homa" | "homi" | "homu")
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "homa" => MtaEventOperatorKind::And,
            "homi" => MtaEventOperatorKind::Or,
            "homu" => MtaEventOperatorKind::Xor,
            _ => panic!("Invalid proposition operator surface: {}", string),
        };
        MtaEventOperator {
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
            MtaEventOperatorKind::And => "homa".to_string(),
            MtaEventOperatorKind::Or => "homi".to_string(),
            MtaEventOperatorKind::Xor => "homu".to_string(),
        }
    }
    pub fn get_kind(&self) -> &MtaEventOperatorKind {
        &self.kind
    }
}

impl ArgumentRetriever {
    fn is_match(string: &str) -> bool {
        matches!(string, "ra" | "ri")
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "ra" => ArgumentRetrieverKind::Nominative,
            "ri" => ArgumentRetrieverKind::Accusative,
            _ => panic!("Invalid argument retriever surface: {}", string),
        };
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
            kind,
        }
    }
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }
    pub fn get_kind(&self) -> &ArgumentRetrieverKind {
        &self.kind
    }
    pub fn get_string(&self) -> String {
        match self.kind {
            ArgumentRetrieverKind::Nominative => "ra".to_string(),
            ArgumentRetrieverKind::Accusative => "ri".to_string(),
        }
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
    // 数字テーブル: 子音p/k/m/s x 母音a/e/i/o で1桁(0〜15)を表す。
    const DIGIT_CONSONANTS: [char; 4] = ['p', 'k', 'm', 's'];
    // 桁位置テーブル: 子音d/g/z/n x 母音a/e/i/o で指数(0〜15)を表す。
    const PLACE_CONSONANTS: [char; 4] = ['d', 'g', 'z', 'n'];
    const VOWELS: [char; 4] = ['a', 'e', 'i', 'o'];

    fn is_match(string: &str) -> bool {
        if Self::is_arabic_numeral_match(string) {
            return true;
        }
        let re_number = Regex::new(
            r"^tu(?:ru)?(?:(?:[pkms][aeio][dgzn][aeio])+(?:[pkms][aeio])?|[pkms][aeio])$",
        )
        .unwrap();
        if !re_number.is_match(string) {
            return false;
        }
        let rest = &string[2..];
        let explicit_hex = rest.starts_with("ru");
        if explicit_hex {
            return true;
        }
        // ruが無ければ10進数として扱うので、桁の値はすべて0〜9でなければならない
        // (10〜15を含む桁を言いたい場合は必ずruを付ける)。
        Self::parse_terms(rest).iter().all(|&(digit, _)| digit <= 9)
    }
    // アラビア数字表記(算用数字)。"□"がzaの代替表記であるのと同様、
    // 数詞の代替入力表記として1桁目〜任意桁の数字列、または"1_000"のように
    // 3桁ごとにアンダースコアで区切った数字列を受け付ける(常に10進数として解釈する)。
    fn is_arabic_numeral_match(string: &str) -> bool {
        let re_arabic = Regex::new(r"^(?:[0-9]+|[0-9]{1,3}(?:_[0-9]{3})*)$").unwrap();
        if !re_arabic.is_match(string) {
            return false;
        }
        let digits_only: String = string.chars().filter(|&c| c != '_').collect();
        digits_only.parse::<u64>().is_ok()
    }
    fn parse_terms(rest: &str) -> Vec<(u64, u32)> {
        let chars: Vec<char> = rest.chars().collect();
        let mut terms: Vec<(u64, u32)> = Vec::new();
        let mut i = 0;
        while i < chars.len() {
            let digit = Self::digit_from_syllable(chars[i], chars[i + 1]);
            i += 2;
            if i + 1 < chars.len() && Self::PLACE_CONSONANTS.contains(&chars[i]) {
                let exponent = Self::place_from_syllable(chars[i], chars[i + 1]);
                i += 2;
                terms.push((digit, exponent));
            } else {
                terms.push((digit, 0));
            }
        }
        terms
    }
    fn digit_from_syllable(consonant: char, vowel: char) -> u64 {
        let row = Self::DIGIT_CONSONANTS
            .iter()
            .position(|&c| c == consonant)
            .unwrap();
        let col = Self::VOWELS.iter().position(|&v| v == vowel).unwrap();
        (row * 4 + col) as u64
    }
    fn place_from_syllable(consonant: char, vowel: char) -> u32 {
        let row = Self::PLACE_CONSONANTS
            .iter()
            .position(|&c| c == consonant)
            .unwrap();
        let col = Self::VOWELS.iter().position(|&v| v == vowel).unwrap();
        (row * 4 + col) as u32
    }
    fn digit_to_syllable(digit: u64) -> String {
        let row = (digit / 4) as usize;
        let col = (digit % 4) as usize;
        format!("{}{}", Self::DIGIT_CONSONANTS[row], Self::VOWELS[col])
    }
    fn place_to_syllable(exponent: u32) -> String {
        let row = (exponent / 4) as usize;
        let col = (exponent % 4) as usize;
        format!("{}{}", Self::PLACE_CONSONANTS[row], Self::VOWELS[col])
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
        let (value, is_hex) = if Self::is_arabic_numeral_match(string) {
            let digits_only: String = string.chars().filter(|&c| c != '_').collect();
            (digits_only.parse::<u64>().unwrap(), false)
        } else {
            let mut rest = &string[2..]; // "tu"を読み飛ばす
            let is_hex = rest.starts_with("ru");
            if is_hex {
                rest = &rest[2..];
            }
            let terms = Self::parse_terms(rest);
            let base: u64 = if is_hex { 16 } else { 10 };
            let value = terms
                .iter()
                .map(|(digit, exponent)| digit * base.pow(*exponent))
                .sum();
            (value, is_hex)
        };
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
            value,
            is_hex,
        }
    }
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }
    pub fn get_value(&self) -> u64 {
        self.value
    }
    pub fn is_hex(&self) -> bool {
        self.is_hex
    }
    pub fn get_string(&self) -> String {
        let base: u64 = if self.is_hex { 16 } else { 10 };
        let mut digits: Vec<u64> = Vec::new();
        if self.value == 0 {
            digits.push(0);
        } else {
            let mut v = self.value;
            while v > 0 {
                digits.push(v % base);
                v /= base;
            }
            digits.reverse();
        }
        let digit_count = digits.len();
        let mut result = String::from("tu");
        if self.is_hex {
            result.push_str("ru");
        }
        for (index, &digit) in digits.iter().enumerate() {
            let exponent = (digit_count - 1 - index) as u32;
            if exponent == 0 {
                if digit != 0 || digit_count == 1 {
                    result.push_str(&Self::digit_to_syllable(digit));
                }
            } else if digit != 0 {
                result.push_str(&Self::digit_to_syllable(digit));
                result.push_str(&Self::place_to_syllable(exponent));
            }
        }
        result
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
        string == "he"
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
        "he".to_string()
    }
}

impl MtaEventToArgumentPhraseConverterEnd {
    fn is_match(string: &str) -> bool {
        string == "ho"
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
        "ho".to_string()
    }
}

impl PostMtaEventModifier {
    fn is_match(string: &str) -> bool {
        matches!(
            string,
            "da" | "hesute"
                | "henesute"
                | "hesuto"
                | "henesuto"
                | "hehoma"
                | "hehomi"
                | "hehomu"
                | "hehita"
                | "hehiti"
                | "hehitu"
                | "hezu"
                | "hehema"
                | "hehegi"
                | "hehenu"
        )
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "da" => PostMtaEventModifierKind::Da,
            "hesute" => PostMtaEventModifierKind::Hesute,
            "henesute" => PostMtaEventModifierKind::Henesute,
            "hesuto" => PostMtaEventModifierKind::Hesuto,
            "henesuto" => PostMtaEventModifierKind::Henesuto,
            "hehoma" => PostMtaEventModifierKind::Hehoma,
            "hehomi" => PostMtaEventModifierKind::Hehomi,
            "hehomu" => PostMtaEventModifierKind::Hehomu,
            "hehita" => PostMtaEventModifierKind::Hehita,
            "hehiti" => PostMtaEventModifierKind::Hehiti,
            "hehitu" => PostMtaEventModifierKind::Hehitu,
            "hezu" => PostMtaEventModifierKind::Hezu,
            "hehema" => PostMtaEventModifierKind::Hehema,
            "hehegi" => PostMtaEventModifierKind::Hehegi,
            "hehenu" => PostMtaEventModifierKind::Hehenu,
            _ => panic!("Invalid post mta event modifier surface: {}", string),
        };
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
            kind,
        }
    }
    pub fn get_string(&self) -> String {
        match self.kind {
            PostMtaEventModifierKind::Da => "da".to_string(),
            PostMtaEventModifierKind::Hesute => "hesute".to_string(),
            PostMtaEventModifierKind::Henesute => "henesute".to_string(),
            PostMtaEventModifierKind::Hesuto => "hesuto".to_string(),
            PostMtaEventModifierKind::Henesuto => "henesuto".to_string(),
            PostMtaEventModifierKind::Hehoma => "hehoma".to_string(),
            PostMtaEventModifierKind::Hehomi => "hehomi".to_string(),
            PostMtaEventModifierKind::Hehomu => "hehomu".to_string(),
            PostMtaEventModifierKind::Hehita => "hehita".to_string(),
            PostMtaEventModifierKind::Hehiti => "hehiti".to_string(),
            PostMtaEventModifierKind::Hehitu => "hehitu".to_string(),
            PostMtaEventModifierKind::Hezu => "hezu".to_string(),
            PostMtaEventModifierKind::Hehema => "hehema".to_string(),
            PostMtaEventModifierKind::Hehegi => "hehegi".to_string(),
            PostMtaEventModifierKind::Hehenu => "hehenu".to_string(),
        }
    }
    pub fn get_kind(&self) -> &PostMtaEventModifierKind {
        &self.kind
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
    Space, // SpatialRelation, Distanceから構成
    Time,  // TemporalRelation, Durationから構成
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
        let input = "binary_predicate argument accusative_preposition argument eos\nbinary_predicate argument accusative_preposition argument eos";
        let tokens = split_with_utf8_ranges(input);
        assert_eq!(tokens.len(), 10);
        assert_eq!(tokens[0], ("binary_predicate", 0, 0, 0, 16));
        assert_eq!(tokens[1], ("argument", 0, 17, 0, 25));
        assert_eq!(tokens[2], ("accusative_preposition", 0, 26, 0, 48));
        assert_eq!(tokens[3], ("argument", 0, 49, 0, 57));
        assert_eq!(tokens[4], ("eos", 0, 58, 0, 61));
        assert_eq!(tokens[5], ("binary_predicate", 1, 0, 1, 16));
        assert_eq!(tokens[6], ("argument", 1, 17, 1, 25));
        assert_eq!(tokens[7], ("accusative_preposition", 1, 26, 1, 48));
        assert_eq!(tokens[8], ("argument", 1, 49, 1, 57));
        assert_eq!(tokens[9], ("eos", 1, 58, 1, 61));
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
            TerminalType::CaseLabel(CaseLabel::new("si", 0, 10, 0, 12))
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
        let input = "hapa □ hitu hudidi parobe me si gi";
        let sentences = tokenize(input).unwrap();
        let sentences = add_eos_to_last_sentence(sentences);
        assert_eq!(sentences.len(), 1);
        assert_eq!(sentences[0].len(), 9);
        assert_eq!(
            sentences[0][0],
            TerminalType::ModalityBase(ModalityBase::new("hapa", 0, 0, 0, 4))
        );
        assert_eq!(
            sentences[0][1],
            TerminalType::ModalityForce(ModalityForce::new("□", 0, 5, 0, 6))
        );
        assert_eq!(
            sentences[0][2],
            TerminalType::Tense(Tense::new("hitu", 0, 7, 0, 11))
        );
        assert_eq!(
            sentences[0][3],
            TerminalType::Aspect(Aspect::new("hudidi", 0, 12, 0, 18))
        );
        assert_eq!(
            sentences[0][4],
            TerminalType::BinaryPredicate(BinaryPredicate::new("parobe", 0, 19, 0, 25))
        );
        assert_eq!(
            sentences[0][5],
            TerminalType::Argument(Argument::new("me", 0, 26, 0, 28))
        );
        assert_eq!(
            sentences[0][6],
            TerminalType::CaseLabel(CaseLabel::new("si", 0, 29, 0, 31))
        );
        assert_eq!(
            sentences[0][7],
            TerminalType::Argument(Argument::new("gi", 0, 32, 0, 34))
        );
    }

    #[test]
    fn test_2文のトークン分割が成功する() {
        let input = "parobe me si ba eos ri parobe bu si gu";
        let sentences = tokenize(input).unwrap();
        let sentences = add_eos_to_last_sentence(sentences);
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].len(), 5);
        assert_eq!(sentences[1].len(), 6);
    }

    fn assert_number_roundtrip(string: &str, expected_value: u64, expected_is_hex: bool) {
        assert!(Number::is_match(string), "should match: {}", string);
        let number = Number::new(string, 0, 0, 0, 0);
        assert_eq!(number.get_value(), expected_value, "value of {}", string);
        assert_eq!(number.is_hex(), expected_is_hex, "is_hex of {}", string);
        assert_eq!(number.get_string(), string, "canonical form of {}", string);
    }

    #[test]
    fn test_数詞_1桁の値が桁位置なしで往復変換できる() {
        assert_number_roundtrip("tupa", 0, false);
        assert_number_roundtrip("tuka", 4, false);
        assert_number_roundtrip("tume", 9, false);
    }

    #[test]
    fn test_数詞_複数桁_ゼロを含む値が桁位置省略ルール込みで往復変換できる() {
        assert_number_roundtrip("tupede", 10, false);
        assert_number_roundtrip("tupedi", 100, false);
        assert_number_roundtrip("tupedo", 1000, false);
        assert_number_roundtrip("tupedopidipodeka", 1234, false);
    }

    #[test]
    fn test_数詞_16進数はruを付けて表す() {
        // Aに相当するmi(値10)を含むので、ruを付けて明示的に16進として読む。
        assert_number_roundtrip("turupedimidepi", 0x1A2, true);
        assert_number_roundtrip("turuso", 15, true);
        // 0xFF = F(15)×16 + F(15)。上位桁には桁位置(16^1 = de)が必要で、末尾の1の位だけ桁位置なし。
        assert_number_roundtrip("turusodeso", 0xFF, true);
    }

    #[test]
    fn test_数詞_桁が全て0から9でもruがあれば16進として解釈される() {
        // ruなしなら10進の12だが、ru付きだと16進の12(=18)になる。
        let with_ru = Number::new("turupedepi", 0, 0, 0, 0);
        assert_eq!(with_ru.get_value(), 0x12);
        assert!(with_ru.is_hex());
        assert_eq!(with_ru.get_string(), "turupedepi");

        let without_ru = Number::new("tupedepi", 0, 0, 0, 0);
        assert_eq!(without_ru.get_value(), 12);
        assert!(!without_ru.is_hex());
    }

    #[test]
    fn test_数詞_不正な文字列はis_matchで弾かれる() {
        assert!(!Number::is_match("tu")); // マーカーのみ
        assert!(!Number::is_match("turu")); // マーカー+フラグのみで数字が無い
        assert!(!Number::is_match("tude")); // 数字が無く桁位置だけ
        assert!(!Number::is_match("tup")); // 末尾が半端(母音が無い)
        assert!(!Number::is_match("tuba")); // 数字テーブルに無い子音(b)
        assert!(!Number::is_match("tururu")); // ruが2回連続
        assert!(!Number::is_match("pa")); // マーカーが無い
        assert!(!Number::is_match("tuso")); // ruなしで10〜15の桁(so=15)を使っている
        assert!(!Number::is_match("tupedimidepi")); // ruなしでmi(=10)を使っている
    }

    #[test]
    fn test_数詞_アラビア数字でも代替表記できる() {
        // "□"がzaの代替表記であるのと同様、アラビア数字は数詞の代替入力表記になる。
        // ただし正準形(get_string)は常に数詞の語(tu...)。
        let n = Number::new("12345", 0, 0, 0, 0);
        assert_eq!(n.get_value(), 12345);
        assert!(!n.is_hex());
        assert_eq!(n.get_string(), "tupegapidopodikadeke");

        let n = Number::new("0", 0, 0, 0, 0);
        assert_eq!(n.get_value(), 0);
        assert_eq!(n.get_string(), "tupa");
    }

    #[test]
    fn test_数詞_アラビア数字は3桁ごとのアンダースコア区切りを受け付ける() {
        assert_eq!(Number::new("1_000", 0, 0, 0, 0).get_value(), 1_000);
        assert_eq!(Number::new("1_000_000", 0, 0, 0, 0).get_value(), 1_000_000);
        assert_eq!(Number::new("12_345", 0, 0, 0, 0).get_value(), 12345);
        // アンダースコアが無くても任意の桁数を受け付ける。
        assert_eq!(Number::new("1000000", 0, 0, 0, 0).get_value(), 1_000_000);
    }

    #[test]
    fn test_数詞_不正なアラビア数字表記はis_matchで弾かれる() {
        assert!(!Number::is_match("1_00")); // 3桁区切りになっていない
        assert!(!Number::is_match("1_0000")); // 3桁区切りになっていない
        assert!(!Number::is_match("_1000")); // 先頭がアンダースコア
        assert!(!Number::is_match("1000_")); // 末尾がアンダースコア
        assert!(!Number::is_match("1__000")); // アンダースコアが連続
        assert!(!Number::is_match("12a45")); // 数字以外の文字を含む
    }
}
