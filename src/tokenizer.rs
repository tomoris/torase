use core::panic;
use std::mem::discriminant;

use crate::types::{ArgumentType, PhysicalEntity};
use regex_engine::Regex;

mod number;
pub(crate) mod regex_engine;
pub use number::Number;

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

fn make_range(
    start_line: usize,
    start_character: usize,
    end_line: usize,
    end_character: usize,
) -> Range {
    Range {
        start: Position {
            line: start_line,
            character: start_character,
        },
        end: Position {
            line: end_line,
            character: end_character,
        },
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Vec<TerminalType>>, IncompleteToken> {
    let tokens = split_with_utf8_ranges(input)?;

    let mut doc = Vec::new();
    // 直前が固有名詞の開始マーカーだった場合、次のトークンは無条件に生テキストとして
    // 扱う(通常のis_matchカスケードを一切通さない)。
    let mut expect_proper_noun_body = false;

    for token in &tokens {
        if expect_proper_noun_body {
            doc.push(TerminalType::ProperNounBody(ProperNounBody::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
            expect_proper_noun_body = false;
            continue;
        }
        if InterrogativeMarker::is_match(token.0) {
            doc.push(TerminalType::InterrogativeMarker(InterrogativeMarker::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if UniversalMarker::is_match(token.0) {
            doc.push(TerminalType::UniversalMarker(UniversalMarker::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if ModalityBase::is_match(token.0) {
            doc.push(TerminalType::ModalityBase(ModalityBase::new(
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
        } else if PhraseOperator::is_match(token.0) {
            doc.push(TerminalType::PhraseOperator(PhraseOperator::new(
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
        } else if AdjunctArgumentRetriever::is_match(token.0) {
            doc.push(TerminalType::AdjunctArgumentRetriever(
                AdjunctArgumentRetriever::new(token.0, token.1, token.2, token.3, token.4),
            ));
        } else if Quantifier::is_match(token.0) {
            doc.push(TerminalType::Quantifier(Quantifier::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if Number::is_match(token.0) {
            doc.push(TerminalType::Number(Number::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if ProperNoun::is_match(token.0) {
            doc.push(TerminalType::ProperNoun(ProperNoun::new(
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
        } else if ProperNounOpener::is_match(token.0) {
            doc.push(TerminalType::ProperNounOpener(ProperNounOpener::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
            expect_proper_noun_body = true;
        } else if ProperNounCloser::is_match(token.0) {
            doc.push(TerminalType::ProperNounCloser(ProperNounCloser::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if ConnectiveHead::is_match(token.0) {
            doc.push(TerminalType::ConnectiveHead(ConnectiveHead::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if RoleLabel::is_match(token.0) {
            doc.push(TerminalType::RoleLabel(RoleLabel::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
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

type WordSpan<'a> = (&'a str, usize, usize, usize, usize);

/// 固有名詞の引用区間(`he[CV] ... hoho`)の追跡状態。
enum QuoteState {
    None,
    AfterOpener,
    InQuote(usize, usize, usize),
}

struct WordBoundary<'a> {
    word: &'a str,
    byte_idx: usize,
    start_row: usize,
    start_col: usize,
    end_row: usize,
    end_col: usize,
}

fn handle_completed_word<'a>(
    boundary: WordBoundary<'a>,
    input: &'a str,
    result: &mut Vec<WordSpan<'a>>,
    quote_state: &mut QuoteState,
) {
    let WordBoundary {
        word,
        byte_idx,
        start_row,
        start_col,
        end_row,
        end_col,
    } = boundary;

    match *quote_state {
        QuoteState::InQuote(quote_idx, quote_row, quote_col) => {
            if word == "hoho" {
                let raw = input[quote_idx..byte_idx].trim_end();
                result.push((raw, quote_row, quote_col, start_row, start_col));
                result.push((word, start_row, start_col, end_row, end_col));
                *quote_state = QuoteState::None;
            }
        }
        QuoteState::AfterOpener => {
            if word == "hoho" {
                // 開始マーカーの直後にすぐ終了マーカーが続く場合(空の固有名詞)。
                result.push(("", start_row, start_col, start_row, start_col));
                result.push((word, start_row, start_col, end_row, end_col));
                *quote_state = QuoteState::None;
            } else {
                *quote_state = QuoteState::InQuote(byte_idx, start_row, start_col);
            }
        }
        QuoteState::None => {
            result.push((word, start_row, start_col, end_row, end_col));
            if ProperNounOpener::is_match(word) {
                *quote_state = QuoteState::AfterOpener;
            }
        }
    }
}

fn split_with_utf8_ranges(input: &str) -> Result<Vec<WordSpan<'_>>, IncompleteToken> {
    let mut result = Vec::new();
    let mut in_word = false;
    let mut start_idx = 0;
    let mut start_row = 0;
    let mut start_col = 0;

    let mut row = 0;
    let mut col = 0;

    // 固有名詞の引用区間(`he[CV] ... hoho`)を追跡する状態。
    let mut quote_state = QuoteState::None;

    for (i, c) in input.char_indices() {
        if c.is_whitespace() {
            if in_word {
                handle_completed_word(
                    WordBoundary {
                        word: &input[start_idx..i],
                        byte_idx: start_idx,
                        start_row,
                        start_col,
                        end_row: row,
                        end_col: col,
                    },
                    input,
                    &mut result,
                    &mut quote_state,
                );
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
        handle_completed_word(
            WordBoundary {
                word: &input[start_idx..],
                byte_idx: start_idx,
                start_row,
                start_col,
                end_row: row,
                end_col: col,
            },
            input,
            &mut result,
            &mut quote_state,
        );
    }

    if let QuoteState::InQuote(_, quote_row, quote_col) = quote_state {
        return Err(IncompleteToken {
            range: make_range(quote_row, quote_col, row, col),
        });
    }
    if matches!(quote_state, QuoteState::AfterOpener) {
        // 開始マーカーの直後に何も続かないまま入力が終わった場合。
        if let Some(last) = result.last() {
            return Err(IncompleteToken {
                range: make_range(last.1, last.2, last.3, last.4),
            });
        }
    }

    Ok(result)
}

// terminal types

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminalType {
    InterrogativeMarker(InterrogativeMarker),
    UniversalMarker(UniversalMarker),
    ModalityBase(ModalityBase),
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
    PhraseOperator(PhraseOperator),
    MtaEventOperator(MtaEventOperator),
    ArgumentRetriever(ArgumentRetriever),
    AdjunctArgumentRetriever(AdjunctArgumentRetriever),
    Quantifier(Quantifier),
    Number(Number),
    Determiner(Determiner),
    MtaEventToArgumentPhraseConverter(MtaEventToArgumentPhraseConverter),
    MtaEventToArgumentPhraseConverterEnd(MtaEventToArgumentPhraseConverterEnd),
    ProperNoun(ProperNoun),
    ProperNounOpener(ProperNounOpener),
    ProperNounBody(ProperNounBody),
    ProperNounCloser(ProperNounCloser),
    ConnectiveHead(ConnectiveHead),
    RoleLabel(RoleLabel),
    PostBinaryPredicatePhraseModifier(PostBinaryPredicatePhraseModifier),
    PostArgumentPhraseModifier(PostArgumentPhraseModifier),
    Eos(Eos),
}

#[derive(Clone, Debug)]
pub enum DummyTerminalType {
    InterrogativeMarker,
    UniversalMarker,
    ModalityBase,
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
    PhraseOperator,
    MtaEventOperator,
    ArgumentRetriever,
    AdjunctArgumentRetriever,
    Quantifier,
    Number,
    Determiner,
    MtaEventToArgumentPhraseConverter,
    MtaEventToArgumentPhraseConverterEnd,
    ProperNoun,
    ProperNounOpener,
    ProperNounBody,
    ProperNounCloser,
    ConnectiveHead,
    RoleLabel,
    PostBinaryPredicatePhraseModifier,
    PostArgumentPhraseModifier,
    Eos,
}

pub fn get_dummy_terminal_type(terminal: TerminalType) -> DummyTerminalType {
    match terminal {
        TerminalType::InterrogativeMarker(_) => DummyTerminalType::InterrogativeMarker,
        TerminalType::UniversalMarker(_) => DummyTerminalType::UniversalMarker,
        TerminalType::ModalityBase(_) => DummyTerminalType::ModalityBase,
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
        TerminalType::PhraseOperator(_) => DummyTerminalType::PhraseOperator,
        TerminalType::MtaEventOperator(_) => DummyTerminalType::MtaEventOperator,
        TerminalType::ArgumentRetriever(_) => DummyTerminalType::ArgumentRetriever,
        TerminalType::AdjunctArgumentRetriever(_) => DummyTerminalType::AdjunctArgumentRetriever,
        TerminalType::Quantifier(_) => DummyTerminalType::Quantifier,
        TerminalType::Number(_) => DummyTerminalType::Number,
        TerminalType::Determiner(_) => DummyTerminalType::Determiner,
        TerminalType::MtaEventToArgumentPhraseConverter(_) => {
            DummyTerminalType::MtaEventToArgumentPhraseConverter
        }
        TerminalType::MtaEventToArgumentPhraseConverterEnd(_) => {
            DummyTerminalType::MtaEventToArgumentPhraseConverterEnd
        }
        TerminalType::ProperNoun(_) => DummyTerminalType::ProperNoun,
        TerminalType::ProperNounOpener(_) => DummyTerminalType::ProperNounOpener,
        TerminalType::ProperNounBody(_) => DummyTerminalType::ProperNounBody,
        TerminalType::ProperNounCloser(_) => DummyTerminalType::ProperNounCloser,
        TerminalType::ConnectiveHead(_) => DummyTerminalType::ConnectiveHead,
        TerminalType::RoleLabel(_) => DummyTerminalType::RoleLabel,
        TerminalType::PostBinaryPredicatePhraseModifier(_) => {
            DummyTerminalType::PostBinaryPredicatePhraseModifier
        }
        TerminalType::PostArgumentPhraseModifier(_) => {
            DummyTerminalType::PostArgumentPhraseModifier
        }
        TerminalType::Eos(_) => DummyTerminalType::Eos,
    }
}

/// 内部の`get_range()`に委譲する`start`/`end`/`range`を型名リストから一括生成する。
macro_rules! impl_terminal_range {
    ($($ty:ident),+ $(,)?) => {
        impl TerminalType {
            pub fn start(&self) -> Position {
                match self {
                    $(TerminalType::$ty(inner) => inner.get_range().start(),)+
                }
            }
            pub fn end(&self) -> Position {
                match self {
                    $(TerminalType::$ty(inner) => inner.get_range().end(),)+
                }
            }
            pub fn range(&self) -> Range {
                Range {
                    start: self.start(),
                    end: self.end(),
                }
            }
        }
    };
}

impl_terminal_range!(
    InterrogativeMarker,
    UniversalMarker,
    ModalityBase,
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
    PhraseOperator,
    MtaEventOperator,
    ArgumentRetriever,
    AdjunctArgumentRetriever,
    Quantifier,
    Number,
    Determiner,
    MtaEventToArgumentPhraseConverter,
    MtaEventToArgumentPhraseConverterEnd,
    ProperNoun,
    ProperNounOpener,
    ProperNounBody,
    ProperNounCloser,
    ConnectiveHead,
    RoleLabel,
    PostBinaryPredicatePhraseModifier,
    PostArgumentPhraseModifier,
    Eos,
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterrogativeMarker {
    range: Range,
}

/// 定義文の先頭に置く全称量化マーカー(`ze`)。`za`/`zi`(スキーマ変数)を含む平叙文を
/// `∀za. ∀zi. (...)`として閉じる。詳細は`docs/language_design/morphology.md`参照。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UniversalMarker {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModalityBase {
    range: Range,
    kind: ModalityBaseKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModalityBaseKind {
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
pub enum TenseKind {
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
pub enum AspectKind {
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
pub enum NegationKind {
    Ne, // 否定(命題・事象レベル。¬P)
    No, // 反意化(述語レベル。Pとは独立した新しい原子述語@antiを作る)
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
    Za, // 定義用のスキーマ変数(x1)
    Zi, // 定義用のスキーマ変数(x2)
    Ko, // 疑問詞(何、誰、いつ、どこ)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseLabel {
    range: Range,
    kind: CaseLabelKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
pub struct PhraseOperator {
    range: Range,
    kind: OperatorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MtaEventOperator {
    range: Range,
    kind: OperatorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperatorKind {
    And,
    Or,
    Xor,
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
    Event,      // re
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdjunctArgumentRetriever {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quantifier {
    range: Range,
    kind: QuantifierKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuantifierKind {
    Ta, // 全ての
    Te, // 多くの
    Ti, // 半分の
    To, // 少なくとも1つは存在する
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

/// t で始まる1語の固有名詞。2文字目の母音が型を表す
/// (`ta`=生物、`te`=物理的実体、`ti`=空間、`to`=抽象)。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProperNoun {
    range: Range,
    string: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProperNounOpener {
    range: Range,
    kind: ProperNounOpenerKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ProperNounOpenerKind {
    Heba, // 人名
    Hebi, // 動物名
    Heza, // 地名など空間的固有名詞
    Hezi, // 記念日・元号など時間的固有名詞
    Hemu, // 団体・組織・企業名
    Heda, // 道具・乗り物などの固有名
    Hedi, // 建造物・ランドマークの固有名
    Hedu, // その他人工物の固有名
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProperNounBody {
    range: Range,
    text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProperNounCloser {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectiveHead {
    range: Range,
    kind: ConnectiveHeadKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectiveHeadKind {
    Hesuteto,   // 接続詞(原因⇔結果)
    Henosuteto, // 接続詞(反対の原因⇔結果)
    Hehitatu,   // 接続詞(時間前後)
    Hehiti,     // 接続詞(同時)
    Hehoma,     // 接続詞(and)
    Hehomi,     // 接続詞(or)
    Hehomu,     // 接続詞(xor)
    Hehito,     // 接続詞(条件)
    Henohito,   // 接続詞(反実仮想条件)
    Hehenu,     // 接続詞(対比)
    Hehezu,     // 接続詞(定義。P↔Q)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleLabel {
    range: Range,
    kind: RoleLabelKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RoleLabelKind {
    Hesa,
    Hesi,
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

/// 表層文字列が固定の1語のみで、kindを持たない終端記号型向けのボイラープレート生成マクロ。
macro_rules! define_fixed_token {
    ($ty:ident, $label:literal, $surface:literal) => {
        impl $ty {
            fn is_match(string: &str) -> bool {
                string == $surface
            }
            fn new(
                string: &str,
                start_line: usize,
                start_character: usize,
                end_line: usize,
                end_character: usize,
            ) -> Self {
                if !Self::is_match(string) {
                    panic!(concat!("Invalid ", $label, " surface: {}"), string);
                }
                $ty {
                    range: make_range(start_line, start_character, end_line, end_character),
                }
            }
            pub fn get_string(&self) -> String {
                $surface.to_string()
            }
        }
    };
}

/// 表層文字列の有限集合とkind enumのバリアントが1対1(または1対多)で対応する
/// 終端記号型向けのボイラープレート生成マクロ。各バリアントの最初のsurfaceが
/// get_string()の正規表現とする。
macro_rules! define_surface_token {
    ($ty:ident, $kind:ident, $label:literal, {
        $($variant:ident : $canonical:literal $(| $alias:literal)*),+ $(,)?
    }) => {
        impl $ty {
            fn is_match(string: &str) -> bool {
                matches!(string, $($canonical $(| $alias)*)|+)
            }
            fn new(
                string: &str,
                start_line: usize,
                start_character: usize,
                end_line: usize,
                end_character: usize,
            ) -> Self {
                let kind = match string {
                    $($canonical $(| $alias)* => $kind::$variant,)+
                    _ => panic!(concat!("Invalid ", $label, " surface: {}"), string),
                };
                $ty {
                    range: make_range(start_line, start_character, end_line, end_character),
                    kind,
                }
            }
            pub fn get_string(&self) -> String {
                match self.kind {
                    $($kind::$variant => $canonical.to_string(),)+
                }
            }
        }
    };
}

/// `self.range.clone()` を返すだけの `get_range` を型名リストから一括生成する。
macro_rules! impl_get_range {
    ($($ty:ident),+ $(,)?) => {
        $(impl $ty {
            pub(crate) fn get_range(&self) -> Range {
                self.range.clone()
            }
        })+
    };
}

impl_get_range!(
    InterrogativeMarker,
    UniversalMarker,
    ModalityBase,
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
    ConnectiveHead,
    RoleLabel,
    PhraseOperator,
    MtaEventOperator,
    ArgumentRetriever,
    AdjunctArgumentRetriever,
    Quantifier,
    Determiner,
    MtaEventToArgumentPhraseConverter,
    MtaEventToArgumentPhraseConverterEnd,
    ProperNoun,
    ProperNounOpener,
    ProperNounBody,
    ProperNounCloser,
    PostBinaryPredicatePhraseModifier,
    PostArgumentPhraseModifier,
    Eos,
);

define_fixed_token!(InterrogativeMarker, "interrogative marker", "ku");
define_fixed_token!(UniversalMarker, "universal marker", "ze");

define_surface_token!(ModalityBase, ModalityBaseKind, "modality", {
    Hapa: "hapa",
    Hapi: "hapi",
    Hapu: "hapu",
});

define_surface_token!(Tense, TenseKind, "tense", {
    Hita: "hita",
    Hiti: "hiti",
    Hitu: "hitu",
});

define_surface_token!(Aspect, AspectKind, "aspect", {
    Huda: "huda",
    Hude: "hude",
    Hudida: "hudida",
    Hudidi: "hudidi",
    Hudidu: "hudidu",
    Hudo: "hudo",
    Hudu: "hudu",
});

define_surface_token!(ModalityNegation, ModalityNegationKind, "modality negation", {
    Hane: "hane",
});

define_surface_token!(TenseNegation, TenseNegationKind, "tense negation", {
    Hine: "hine",
});

define_surface_token!(AspectNegation, AspectNegationKind, "aspect negation", {
    Hune: "hune",
});

define_surface_token!(Negation, NegationKind, "negation", {
    Ne: "ne",
    No: "no",
});

impl ModalityBase {
    pub fn get_kind(&self) -> &ModalityBaseKind {
        &self.kind
    }
}

impl Tense {
    pub fn get_kind(&self) -> &TenseKind {
        &self.kind
    }
}

impl Aspect {
    pub fn get_kind(&self) -> &AspectKind {
        &self.kind
    }
}

impl Negation {
    pub fn get_kind(&self) -> &NegationKind {
        &self.kind
    }
}

impl BinaryPredicate {
    fn is_match(string: &str) -> bool {
        const RE_BINARY_PREDICATE: Regex =
            Regex::compile(r"^(?:[bdgmnrpkz][aeiuo])(?:[ptkshbdgmnrz][aeiuo])+$");
        RE_BINARY_PREDICATE.is_match(string)
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        BinaryPredicate {
            range: make_range(start_line, start_character, end_line, end_character),
            string: string.to_string(),
        }
    }
    pub(crate) fn get_argument_types(&self) -> (Vec<ArgumentType>, Vec<ArgumentType>) {
        let s = self.string.as_str();
        // 名詞的な命題 / 空間・時間に関する名詞的な命題 / 動詞的な命題。
        // `BinaryPredicate::is_match`の正規表現により先頭子音は[bdgmnrpkz]に限られ、
        // z/p/kは2文字目の母音まで見て初めて曖昧さが解消するため、それぞれ2文字で判定する。
        if s.starts_with("b") {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
            )
        } else if s.starts_with("d") {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact)],
            )
        } else if s.starts_with("n") {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::NOther)],
            )
        } else if s.starts_with("r") {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::Abstraction],
            )
        } else if s.starts_with("za") {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::Space],
            )
        } else if s.starts_with("zi") {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::Time],
            )
        } else if s.starts_with("ze") {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::Space],
            )
        } else if s.starts_with("zu") {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::Time],
            )
        } else if s.starts_with("g") || s.starts_with("m") {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::Abstraction],
            )
        } else if s.starts_with("pa") {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
            )
        } else if s.starts_with("pe") {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![
                    ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                    ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
                ],
            )
        } else if s.starts_with("pi") {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                vec![ArgumentType::Epsilon],
            )
        } else if s.starts_with("po") {
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
        } else if s.starts_with("pu") {
            (
                vec![
                    ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                    ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
                ],
                vec![ArgumentType::Epsilon],
            )
        } else if s.starts_with("ka") {
            (
                vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
                abstraction_family_types(),
            )
        } else if s.starts_with("ki") {
            (
                vec![
                    ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                    ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
                ],
                abstraction_family_types(),
            )
        } else if s.starts_with("ku") {
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

define_surface_token!(Argument, ArgumentKind, "argument", {
    Ma: "ma",
    Mi: "mi",
    Mu: "mu",
    Me: "me",
    Ba: "ba",
    Bu: "bu",
    Be: "be",
    Ga: "ga",
    Gi: "gi",
    Gu: "gu",
    Ge: "ge",
    Za: "za",
    Zi: "zi",
    Ko: "ko",
});

impl Argument {
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
            ArgumentKind::Za => vec![
                ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing),
                ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
                ArgumentType::Abstraction,
                ArgumentType::Space,
                ArgumentType::Time,
            ],
            ArgumentKind::Zi => vec![
                ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing),
                ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
                ArgumentType::Abstraction,
                ArgumentType::Space,
                ArgumentType::Time,
            ],
            ArgumentKind::Ko => vec![
                ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing),
                ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
                ArgumentType::Abstraction,
                ArgumentType::Space,
                ArgumentType::Time,
            ],
        }
    }

    pub(crate) fn is_schema_variable(&self) -> bool {
        matches!(self.kind, ArgumentKind::Za | ArgumentKind::Zi)
    }

    /// 不定代名詞`ga`(誰か)。特定の個体ではなく、存在量化される項を表す。
    pub(crate) fn is_indefinite(&self) -> bool {
        matches!(self.kind, ArgumentKind::Ga)
    }

    /// 疑問詞`ko`。`ku`で始まる疑問文の中でのみ使え、尋ねる項を表す。
    pub(crate) fn is_interrogative(&self) -> bool {
        matches!(self.kind, ArgumentKind::Ko)
    }
}

define_surface_token!(CaseLabel, CaseLabelKind, "case label", {
    Sa: "sa",
    Si: "si",
    Subu: "subu",
    Sube: "sube",
    Subo: "subo",
    Suni: "suni",
    Sune: "sune",
    Suno: "suno",
    Sugi: "sugi",
    Suge: "suge",
    Sugo: "sugo",
    Sudu: "sudu",
    Sute: "sute",
    Suto: "suto",
});

impl CaseLabel {
    pub fn get_kind(&self) -> &CaseLabelKind {
        &self.kind
    }
}

define_fixed_token!(TypeOverride, "type override", "se");

define_surface_token!(PhraseOperator, OperatorKind, "phrase operator", {
    And: "hoba",
    Or: "hobi",
    Xor: "hobu",
});

impl PhraseOperator {
    pub fn get_kind(&self) -> &OperatorKind {
        &self.kind
    }
}

define_surface_token!(MtaEventOperator, OperatorKind, "proposition operator", {
    And: "homa",
    Or: "homi",
    Xor: "homu",
});

impl MtaEventOperator {
    pub fn get_kind(&self) -> &OperatorKind {
        &self.kind
    }
}

define_surface_token!(ArgumentRetriever, ArgumentRetrieverKind, "argument retriever", {
    Nominative: "ra",
    Accusative: "ri",
    Event: "re",
});

impl ArgumentRetriever {
    pub fn get_kind(&self) -> &ArgumentRetrieverKind {
        &self.kind
    }
    /// 抽出される項に対応する格。`re`は事象そのものを抽出するので空所を持たない。
    pub fn gap_case(&self) -> Option<CaseLabelKind> {
        match self.kind {
            ArgumentRetrieverKind::Nominative => Some(CaseLabelKind::Sa),
            ArgumentRetrieverKind::Accusative => Some(CaseLabelKind::Si),
            ArgumentRetrieverKind::Event => None,
        }
    }
}

define_fixed_token!(AdjunctArgumentRetriever, "adjunct argument retriever", "ru");

define_surface_token!(Quantifier, QuantifierKind, "quantifier", {
    Ta: "ta",
    Te: "te",
    Ti: "ti",
    To: "to",
});

impl Quantifier {
    pub fn get_kind(&self) -> &QuantifierKind {
        &self.kind
    }
}

define_surface_token!(Determiner, DeterminerKind, "determiner", {
    Mo: "mo",
    Bo: "bo",
    Go: "go",
});

define_fixed_token!(
    MtaEventToArgumentPhraseConverter,
    "mta event to argument phrase converter",
    "he"
);

define_fixed_token!(
    MtaEventToArgumentPhraseConverterEnd,
    "mta event to argument phrase converter end",
    "ho"
);

define_surface_token!(ProperNounOpener, ProperNounOpenerKind, "proper noun opener", {
    Heba: "heba",
    Hebi: "hebi",
    Heza: "heza",
    Hezi: "hezi",
    Hemu: "hemu",
    Heda: "heda",
    Hedi: "hedi",
    Hedu: "hedu",
});

impl ProperNoun {
    fn is_match(string: &str) -> bool {
        const RE_PROPER_NOUN: Regex = Regex::compile(r"^t[aeio](?:[ptkbdgmnrszh][aeiou])+$");
        RE_PROPER_NOUN.is_match(string)
    }
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        ProperNoun {
            range: make_range(start_line, start_character, end_line, end_character),
            string: string.to_string(),
        }
    }
    pub fn get_string(&self) -> String {
        self.string.clone()
    }
    pub(crate) fn get_type(&self) -> ArgumentType {
        match self.string.as_bytes()[1] {
            b'a' => ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing),
            b'e' => ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
            b'i' => ArgumentType::Space,
            b'o' => ArgumentType::Abstraction,
            _ => unreachable!("`ProperNoun::is_match` guarantees t[aeio]"),
        }
    }
}

impl ProperNounOpener {
    pub fn get_kind(&self) -> &ProperNounOpenerKind {
        &self.kind
    }
    pub(crate) fn get_type(&self) -> ArgumentType {
        match self.kind {
            ProperNounOpenerKind::Heba | ProperNounOpenerKind::Hebi => {
                ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)
            }
            ProperNounOpenerKind::Heda
            | ProperNounOpenerKind::Hedi
            | ProperNounOpenerKind::Hedu => ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
            ProperNounOpenerKind::Hemu => ArgumentType::Abstraction,
            ProperNounOpenerKind::Heza => ArgumentType::Space,
            ProperNounOpenerKind::Hezi => ArgumentType::Time,
        }
    }
}

impl ProperNounBody {
    fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        ProperNounBody {
            range: make_range(start_line, start_character, end_line, end_character),
            text: string.to_string(),
        }
    }
    pub fn get_string(&self) -> String {
        self.text.clone()
    }
}

define_fixed_token!(ProperNounCloser, "proper noun closer", "hoho");

define_surface_token!(ConnectiveHead, ConnectiveHeadKind, "connective head", {
    Hesuteto: "hesuteto",
    Henosuteto: "henosuteto",
    Hehitatu: "hehitatu",
    Hehiti: "hehiti",
    Hehoma: "hehoma",
    Hehomi: "hehomi",
    Hehomu: "hehomu",
    Hehito: "hehito",
    Henohito: "henohito",
    Hehenu: "hehenu",
    Hehezu: "hehezu",
});

impl ConnectiveHead {
    pub fn get_kind(&self) -> &ConnectiveHeadKind {
        &self.kind
    }
}

define_surface_token!(RoleLabel, RoleLabelKind, "role label", {
    Hesa: "hesa",
    Hesi: "hesi",
});

impl RoleLabel {
    pub fn get_kind(&self) -> &RoleLabelKind {
        &self.kind
    }
}

define_fixed_token!(
    PostBinaryPredicatePhraseModifier,
    "post binary predicate phrase modifier",
    "di"
);

define_fixed_token!(
    PostArgumentPhraseModifier,
    "post argument phrase modifier",
    "du"
);

define_fixed_token!(Eos, "eos", "do");

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_with_utf8_ranges() {
        let input = "binary_predicate argument accusative_preposition argument do\nbinary_predicate argument accusative_preposition argument do";
        let tokens = split_with_utf8_ranges(input).unwrap();
        assert_eq!(tokens.len(), 10);
        assert_eq!(tokens[0], ("binary_predicate", 0, 0, 0, 16));
        assert_eq!(tokens[1], ("argument", 0, 17, 0, 25));
        assert_eq!(tokens[2], ("accusative_preposition", 0, 26, 0, 48));
        assert_eq!(tokens[3], ("argument", 0, 49, 0, 57));
        assert_eq!(tokens[4], ("do", 0, 58, 0, 60));
        assert_eq!(tokens[5], ("binary_predicate", 1, 0, 1, 16));
        assert_eq!(tokens[6], ("argument", 1, 17, 1, 25));
        assert_eq!(tokens[7], ("accusative_preposition", 1, 26, 1, 48));
        assert_eq!(tokens[8], ("argument", 1, 49, 1, 57));
        assert_eq!(tokens[9], ("do", 1, 58, 1, 60));
    }

    #[test]
    fn test_split_with_utf8_ranges_固有名詞の引用区間は1つのチャンクにまとまる() {
        let input = "heba Kanye West hoho";
        let tokens = split_with_utf8_ranges(input).unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], ("heba", 0, 0, 0, 4));
        assert_eq!(tokens[1], ("Kanye West", 0, 5, 0, 16));
        assert_eq!(tokens[2], ("hoho", 0, 16, 0, 20));
    }

    #[test]
    fn test_split_with_utf8_ranges_空の固有名詞も1トークンとして扱われる() {
        let input = "heba hoho";
        let tokens = split_with_utf8_ranges(input).unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[1].0, "");
    }

    #[test]
    fn test_split_with_utf8_ranges_hohoが無いと未完了トークンエラーになる() {
        let input = "heba Kanye";
        let result = split_with_utf8_ranges(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_split_with_utf8_ranges_開始マーカー直後に何も続かないと未完了トークンエラーになる() {
        let input = "heba";
        let result = split_with_utf8_ranges(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_tokenize_固有名詞が正しくトークナイズされる() {
        let input = "heba Kanye West hoho do";
        let sentences = tokenize(input).unwrap();
        assert_eq!(sentences[0].len(), 4);
        assert_eq!(
            sentences[0][0],
            TerminalType::ProperNounOpener(ProperNounOpener::new("heba", 0, 0, 0, 4))
        );
        assert_eq!(
            sentences[0][1],
            TerminalType::ProperNounBody(ProperNounBody::new("Kanye West", 0, 5, 0, 16))
        );
        assert_eq!(
            sentences[0][2],
            TerminalType::ProperNounCloser(ProperNounCloser::new("hoho", 0, 16, 0, 20))
        );
    }

    #[test]
    fn test_tokenize_固有名詞の中身が他のトークンと同じ文字列でも生テキストになる() {
        // "mi"は代名詞、"ta"は量化詞と同じ表層形だが、引用区間の中身は無条件に
        // ProperNounBodyとして扱われる。
        let sentences = tokenize("heba mi hoho do").unwrap();
        assert_eq!(sentences[0].len(), 4);
        assert!(
            matches!(&sentences[0][1], TerminalType::ProperNounBody(body) if body.get_string() == "mi")
        );

        let sentences = tokenize("hedi ta hoho do").unwrap();
        assert_eq!(sentences[0].len(), 4);
        assert!(
            matches!(&sentences[0][1], TerminalType::ProperNounBody(body) if body.get_string() == "ta")
        );

        let sentences = tokenize("heba A do B hoho do").unwrap();
        assert_eq!(sentences.len(), 1);
        assert!(
            matches!(&sentences[0][1], TerminalType::ProperNounBody(body) if body.get_string() == "A do B")
        );
    }

    #[test]
    fn test_tokenize_tで始まる語は固有名詞になる() {
        for word in ["taro", "tetitanika", "tikanada", "tonasa"] {
            let sentences = tokenize(&format!("{word} do")).unwrap();
            assert!(
                matches!(&sentences[0][0], TerminalType::ProperNoun(pn) if pn.get_string() == word),
                "{word} should be a proper noun: {:?}",
                sentences[0][0]
            );
        }
    }

    #[test]
    fn test_tokenize_量化詞と数詞はtで始まる固有名詞にならない() {
        let sentences = tokenize("ta to tusadi do").unwrap();
        assert!(matches!(&sentences[0][0], TerminalType::Quantifier(_)));
        assert!(matches!(&sentences[0][1], TerminalType::Quantifier(_)));
        assert!(matches!(&sentences[0][2], TerminalType::Number(_)));
    }

    #[test]
    fn test_tokenize_tuで始まる数詞でない語はエラーになる() {
        assert!(tokenize("tuhana do").is_err());
    }

    #[test]
    fn test_tokenize_hohoが無い固有名詞はエラーになる() {
        let input = "parobe sa heba Kanye si ba do";
        let result = tokenize(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_1文のトークン分割が成功する() {
        let input = "parobe ma si ba do";
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
            TerminalType::Eos(Eos::new("do", 0, 16, 0, 18))
        );
    }

    #[test]
    fn test_eosなし1文のトークン分割が成功する() {
        let input = "hapa ta hitu hudidi parobe me si gi";
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
            TerminalType::Quantifier(Quantifier::new("ta", 0, 5, 0, 7))
        );
        assert_eq!(
            sentences[0][2],
            TerminalType::Tense(Tense::new("hitu", 0, 8, 0, 12))
        );
        assert_eq!(
            sentences[0][3],
            TerminalType::Aspect(Aspect::new("hudidi", 0, 13, 0, 19))
        );
        assert_eq!(
            sentences[0][4],
            TerminalType::BinaryPredicate(BinaryPredicate::new("parobe", 0, 20, 0, 26))
        );
        assert_eq!(
            sentences[0][5],
            TerminalType::Argument(Argument::new("me", 0, 27, 0, 29))
        );
        assert_eq!(
            sentences[0][6],
            TerminalType::CaseLabel(CaseLabel::new("si", 0, 30, 0, 32))
        );
        assert_eq!(
            sentences[0][7],
            TerminalType::Argument(Argument::new("gi", 0, 33, 0, 35))
        );
    }

    #[test]
    fn test_2文のトークン分割が成功する() {
        let input = "parobe me si ba do ri parobe bu si gu";
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
        assert_number_roundtrip("tuso", 0, false);
        assert_number_roundtrip("tuko", 4, false);
        assert_number_roundtrip("tuma", 9, false);
    }

    #[test]
    fn test_数詞_複数桁_ゼロを含む値が桁位置省略ルール込みで往復変換できる() {
        assert_number_roundtrip("tusada", 10, false);
        assert_number_roundtrip("tusade", 100, false);
        assert_number_roundtrip("tusadi", 1000, false);
        assert_number_roundtrip("tusadisedesidako", 1234, false);
    }

    #[test]
    fn test_数詞_16進数はruを付けて表す() {
        // Aに相当するme(値10)を含むので、ruを付けて明示的に16進として読む。
        assert_number_roundtrip("turusademedase", 0x1A2, true);
        assert_number_roundtrip("turupi", 15, true);
        // 0xFF = F(15)×16 + F(15)。上位桁には桁位置(16^1 = da)が必要で、末尾の1の位だけ桁位置なし。
        assert_number_roundtrip("turupidapi", 0xFF, true);
    }

    #[test]
    fn test_数詞_桁が全て0から9でもruがあれば16進として解釈される() {
        // ruなしなら10進の12だが、ru付きだと16進の12(=18)になる。
        let with_ru = Number::new("turusadase", 0, 0, 0, 0);
        assert_eq!(with_ru.get_value(), 0x12);
        assert!(with_ru.is_hex());
        assert_eq!(with_ru.get_string(), "turusadase");

        let without_ru = Number::new("tusadase", 0, 0, 0, 0);
        assert_eq!(without_ru.get_value(), 12);
        assert!(!without_ru.is_hex());
    }

    #[test]
    fn test_数詞_不正な文字列はis_matchで弾かれる() {
        assert!(!Number::is_match("tu")); // マーカーのみ
        assert!(!Number::is_match("turu")); // マーカー+フラグのみで数字が無い
        assert!(!Number::is_match("tuda")); // 数字が無く桁位置だけ
        assert!(!Number::is_match("tup")); // 末尾が半端(母音が無い)
        assert!(!Number::is_match("tuba")); // 数字テーブルに無い子音(b)
        assert!(!Number::is_match("tururu")); // ruが2回連続
        assert!(!Number::is_match("pa")); // マーカーが無い
        assert!(!Number::is_match("tupi")); // ruなしで10〜15の桁(pi=15)を使っている
        assert!(!Number::is_match("tusademedase")); // ruなしでme(=10)を使っている
    }

    #[test]
    fn test_数詞_アラビア数字でも代替表記できる() {
        // アラビア数字は数詞の代替入力表記になる。ただし正準形(get_string)は常に数詞の語(tu...)。
        let n = Number::new("12345", 0, 0, 0, 0);
        assert_eq!(n.get_value(), 12345);
        assert!(!n.is_hex());
        assert_eq!(n.get_string(), "tusagosedisidekodaka");

        let n = Number::new("0", 0, 0, 0, 0);
        assert_eq!(n.get_value(), 0);
        assert_eq!(n.get_string(), "tuso");
    }

    #[test]
    fn test_数詞_アラビア数字は4桁ごとのアンダースコア区切りを受け付ける() {
        assert_eq!(Number::new("1_0000", 0, 0, 0, 0).get_value(), 10_000);
        assert_eq!(
            Number::new("1_0000_0000", 0, 0, 0, 0).get_value(),
            100_000_000
        );
        assert_eq!(Number::new("1_2345", 0, 0, 0, 0).get_value(), 12345);
        // アンダースコアが無くても任意の桁数を受け付ける。
        assert_eq!(Number::new("1000000", 0, 0, 0, 0).get_value(), 1_000_000);
    }

    #[test]
    fn test_数詞_不正なアラビア数字表記はis_matchで弾かれる() {
        assert!(!Number::is_match("1_000")); // 4桁区切りになっていない
        assert!(!Number::is_match("1_00000")); // 4桁区切りになっていない
        assert!(!Number::is_match("_10000")); // 先頭がアンダースコア
        assert!(!Number::is_match("10000_")); // 末尾がアンダースコア
        assert!(!Number::is_match("1__0000")); // アンダースコアが連続
        assert!(!Number::is_match("12a45")); // 数字以外の文字を含む
    }
}
