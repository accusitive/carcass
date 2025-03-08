//! Token, tokenizer, node, noder, parser implementations.

#![feature(assert_matches, gen_blocks, if_let_guard, let_chains, trait_alias)]

mod color;
pub mod format;

use std::ops;

use enumset::{
    EnumSet,
    enum_set,
};

pub use self::{
    color::*,
    noder::*,
    tokenizer::*,
};

pub mod node;
mod noder;

pub mod token;
mod tokenizer;

#[allow(dead_code)]
mod red {
    use crate::*;

    pub type Node = cstree::syntax::ResolvedNode<Kind>;
    pub type ResolvedNode = cstree::syntax::SyntaxNode<Kind>;

    pub type Token = cstree::syntax::ResolvedToken<Kind>;
    pub type ResolvedToken = cstree::syntax::SyntaxToken<Kind>;

    pub type Element = cstree::syntax::ResolvedElement<Kind>;
    pub type ResolvedElement = cstree::syntax::SyntaxElement<Kind>;

    pub type ElementRef<'a> = cstree::util::NodeOrToken<&'a Node, &'a Token>;
    pub type ResolvedElementRef<'a> = cstree::util::NodeOrToken<&'a Node, &'a Token>;
}

pub trait Node = TryFrom<red::Node> + ops::Deref<Target = red::Node>;
pub trait NodeRef<'a> = TryFrom<&'a red::Node> + ops::Deref<Target: ops::Deref<Target = red::Node>>;

pub trait Token = TryFrom<red::Token> + ops::Deref<Target = red::Token>;
pub trait TokenRef<'a> = TryFrom<&'a red::Token> + ops::Deref<Target: ops::Deref<Target = red::Token>>;

#[allow(dead_code)]
mod green {
    use std::sync::Arc;

    use crate::*;

    pub type Interner = Arc<cstree::interning::MultiThreadedTokenInterner>;

    pub fn interner() -> Interner {
        Arc::new(cstree::interning::new_threaded_interner())
    }

    pub type Checkpoint = cstree::build::Checkpoint;

    pub type NodeBuilder = cstree::build::GreenNodeBuilder<'static, 'static, Kind, Interner>;
    pub type NodeCache = cstree::build::NodeCache<'static, Interner>;

    pub type Node = cstree::green::GreenNode;
    pub type Token = cstree::green::GreenToken;
}

/// [`derive_more`] causes [`unreachable`] to warn too many times
/// so we're just suppressing it like this. No, #[allow(unreachable_code)]
/// doesn't suppress the warns coming from the #[derive(derive_more::Display)].
const fn unreachable() -> &'static str {
    unreachable!()
}

/// The syntax kind.
#[derive(derive_more::Display, Debug, Clone, Copy, PartialEq, Eq, Hash, enumset::EnumSetType, cstree::Syntax)]
#[repr(u32)]
#[enumset(no_super_impls)]
#[allow(non_camel_case_types)]
#[non_exhaustive]
pub enum Kind {
    /// Represents any sequence of tokens that was not recognized.
    #[display("an unknown token sequence")]
    TOKEN_ERROR_UNKNOWN,

    /// Anything that matches [`char::is_whitespace`].
    #[display("whitespace")]
    TOKEN_WHITESPACE,

    /// Anything that starts with a `#`.
    ///
    /// When the comment starts with `#` and a nonzero number of `=`, it will be
    /// multiline. Multiline comments can be closed with the initial amount of
    /// `=` and then a `#`, but they don't have to be.
    #[display("a comment")]
    TOKEN_COMMENT,

    #[display("';'")]
    #[static_text(";")]
    TOKEN_SEMICOLON,
    #[display("'?'")]
    #[static_text("?")]
    TOKEN_QUESTIONMARK,

    #[display("'<|'")]
    #[static_text("<|")]
    TOKEN_LESS_PIPE,
    #[display("'|>'")]
    #[static_text("|>")]
    TOKEN_PIPE_MORE,

    #[display("'('")]
    #[static_text("(")]
    TOKEN_PARENTHESIS_LEFT,
    #[display("')'")]
    #[static_text(")")]
    TOKEN_PARENTHESIS_RIGHT,

    #[display(r"'\('")]
    TOKEN_INTERPOLATION_START,
    #[display("')'")]
    #[static_text(")")]
    TOKEN_INTERPOLATION_END,

    #[display("','")]
    #[static_text(",")]
    TOKEN_COMMA,
    #[display("'=>'")]
    #[static_text("=>")]
    TOKEN_EQUAL_GREATER,

    #[display("':'")]
    #[static_text(":")]
    TOKEN_COLON,
    #[display("'++'")]
    #[static_text("++")]
    TOKEN_PLUS_PLUS,
    #[display("'['")]
    #[static_text("[")]
    TOKEN_BRACKET_LEFT,
    #[display("']'")]
    #[static_text("]")]
    TOKEN_BRACKET_RIGHT,

    #[display("'//'")]
    #[static_text("//")]
    TOKEN_SLASH_SLASH,
    #[display("'.'")]
    #[static_text(".")]
    TOKEN_PERIOD,
    #[display("'{{'")]
    #[static_text("{")]
    TOKEN_CURLYBRACE_LEFT,
    #[display("'}}'")]
    #[static_text("}")]
    TOKEN_CURLYBRACE_RIGHT,

    #[display("'!='")]
    #[static_text("!=")]
    TOKEN_EXCLAMATION_EQUAL,
    #[display("'='")]
    #[static_text("=")]
    TOKEN_EQUAL,
    #[display("'<='")]
    #[static_text("<=")]
    TOKEN_LESS_EQUAL,
    #[display("'<'")]
    #[static_text("<")]
    TOKEN_LESS,
    #[display("'>='")]
    #[static_text(">=")]
    TOKEN_MORE_EQUAL,
    #[display("'>'")]
    #[static_text(">")]
    TOKEN_MORE,

    #[display("'&&'")]
    #[static_text("&&")]
    TOKEN_AMPERSAND_AMPERSAND,
    #[display("'||'")]
    #[static_text("||")]
    TOKEN_PIPE_PIPE,
    #[display("'!'")]
    #[static_text("!")]
    TOKEN_EXCLAMATIONMARK,
    #[display("'->'")]
    #[static_text("->")]
    TOKEN_MINUS_MORE,

    #[display("'&'")]
    #[static_text("&")]
    TOKEN_AMPERSAND,
    #[display("'|'")]
    #[static_text("|")]
    TOKEN_PIPE,

    #[display("'+'")]
    #[static_text("+")]
    TOKEN_PLUS,
    #[display("'-'")]
    #[static_text("-")]
    TOKEN_MINUS,
    #[display("'*'")]
    #[static_text("*")]
    TOKEN_ASTERISK,
    #[display("'^'")]
    #[static_text("^")]
    TOKEN_CARET,
    #[display("'/'")]
    #[static_text("/")]
    TOKEN_SLASH,

    #[display("a non-decimal number with no digits")]
    TOKEN_ERROR_NUMBER_NO_DIGIT,
    #[display("an integer")]
    TOKEN_INTEGER,
    #[display("a float")]
    TOKEN_FLOAT,
    #[display("a float with a missing exponent")]
    TOKEN_ERROR_FLOAT_NO_EXPONENT,

    #[display("the keyword 'if'")]
    #[static_text("if")]
    TOKEN_KEYWORD_IF,
    #[display("the keyword 'then'")]
    #[static_text("then")]
    TOKEN_KEYWORD_THEN,
    #[display("the keyword 'else'")]
    #[static_text("else")]
    TOKEN_KEYWORD_ELSE,

    /// See [`NODE_STRING`].
    #[display("content")]
    TOKEN_CONTENT,

    /// A zero width token for the start of a path. Has no content.
    #[display("a path")]
    #[static_text("")]
    TOKEN_PATH_START,
    /// A zero width token for the end of a path. Has no content.
    #[display("the closing delimiter of a path")]
    #[static_text("")]
    TOKEN_PATH_END,

    /// A normal non-quoted identifier. All characters must be either
    /// [`char::is_alphanumeric`], `_`, `-` or `'`. The initial character must
    /// not match [`char::is_ascii_digit`].
    #[display("an identifier")]
    TOKEN_IDENTIFIER,

    #[display("'@'")]
    #[static_text("@")]
    TOKEN_AT,
    #[display("an identifier")]
    TOKEN_IDENTIFIER_START,
    #[display("the closing delimiter of an identifier")]
    TOKEN_IDENTIFIER_END,

    #[display("a string")]
    TOKEN_STRING_START,
    #[display("the closing delimiter of a string")]
    TOKEN_STRING_END,

    #[display("a rune")]
    TOKEN_RUNE_START,
    #[display("the closing delimiter of a rune")]
    TOKEN_RUNE_END,

    #[display("an island")]
    #[static_text("<")]
    TOKEN_ISLAND_HEADER_START,
    #[display("the closing delimiter of an island")]
    TOKEN_ISLAND_HEADER_END,

    #[display("{}", unreachable())]
    NODE_ROOT,
    #[display("an erroneous expression")]
    NODE_ERROR,

    #[display("a prefix operation")]
    NODE_PREFIX_OPERATION,
    #[display("an infix operation")]
    NODE_INFIX_OPERATION,
    #[display("a suffix operation")]
    NODE_SUFFIX_OPERATION,

    #[display("a parenthesized expression")]
    NODE_PARENTHESIS,

    #[display("a list")]
    NODE_LIST,

    #[display("attributes")]
    NODE_ATTRIBUTES,

    /// A node which starts with a [`TOKEN_INTERPOLATION_START`], ends with a
    /// [`TOKEN_INTERPOLATION_END`] while having a node at the middle that can
    /// be cast to an [Expression](crate::node::Expression).
    #[display("{}", unreachable())]
    NODE_INTERPOLATION,

    // TODO: Document.
    #[display("{}", unreachable())]
    NODE_ISLAND_HEADER,
    #[display("an island")]
    NODE_ISLAND,

    /// A stringlike that is delimited by zero width [`TOKEN_PATH_START`] and
    /// [`TOKEN_PATH_END`] tokens. See [`NODE_STRING`] for the definition of
    /// stringlike.
    #[display("a path")]
    NODE_PATH,

    /// A node that starts with a [`TOKEN_AT`] and has a [`NODE_IDENTIFIER`] as
    /// a child, used for binding expressions to identifiers.
    #[display("a bind")]
    NODE_BIND,

    /// A stringlike that is delimited by a single backtick. See [`NODE_STRING`]
    /// for the definition of stringlike.
    #[display("an identifier")]
    NODE_IDENTIFIER,

    /// A stringlike that is delimited by a single `"` and any number of `=`:
    ///
    /// ```text
    /// "== foo =="
    /// ```
    ///
    /// A stringlike is a sequence of nodes and tokens, where all the immediate
    /// children tokens are start, end or [`TOKEN_CONTENT`]s, while all the
    /// immediate children nodes are all [`NODE_INTERPOLATION`]s.
    #[display("a string")]
    NODE_STRING,

    /// A stringlike that can only contain a single character delimited by `'`.
    /// See [`NODE_STRING`] for the definition of stringlike.
    #[display("a rune")]
    NODE_RUNE,

    #[display("an integer")]
    NODE_INTEGER,
    #[display("a float")]
    NODE_FLOAT,

    #[display("an if")]
    NODE_IF,
}

use Kind::*;

impl Kind {
    /// An enumset of all valid expression starter token kinds.
    pub const EXPRESSIONS: EnumSet<Kind> = enum_set!(
        TOKEN_PARENTHESIS_LEFT
            | TOKEN_BRACKET_LEFT
            | TOKEN_CURLYBRACE_LEFT
            | TOKEN_INTEGER
            | TOKEN_FLOAT
            | TOKEN_KEYWORD_IF
            | TOKEN_PATH_START
            | TOKEN_AT
            | TOKEN_IDENTIFIER
            | TOKEN_IDENTIFIER_START
            | TOKEN_STRING_START
            | TOKEN_RUNE_START
            | TOKEN_ISLAND_HEADER_START
    );
    /// An enumset of all identifier starter token kinds.
    pub const IDENTIFIERS: EnumSet<Kind> = enum_set!(TOKEN_IDENTIFIER | TOKEN_IDENTIFIER_START);

    /// Whether if this token can be used as a lambda argument.
    ///
    /// ```txt
    /// max 42 (38) + 61
    ///     t  t    f
    /// ```
    pub fn is_argument(self) -> bool {
        let mut arguments = Self::EXPRESSIONS;
        arguments.remove(TOKEN_KEYWORD_IF);

        arguments.contains(self) || self.is_error() // Error nodes are expressions.
    }

    /// Whether if the token should be ignored by the noder.
    pub fn is_trivia(self) -> bool {
        matches!(self, TOKEN_COMMENT | TOKEN_WHITESPACE)
    }

    /// Whether if this token is erroneous.
    pub fn is_error(self) -> bool {
        matches!(
            self,
            TOKEN_ERROR_UNKNOWN | TOKEN_ERROR_NUMBER_NO_DIGIT | TOKEN_ERROR_FLOAT_NO_EXPONENT
        )
    }

    /// Returns the node and closing kinds of this kind.
    pub fn as_node_and_closing(self) -> Option<(Kind, Kind)> {
        Some(match self {
            TOKEN_PATH_START => (NODE_PATH, TOKEN_PATH_END),
            TOKEN_IDENTIFIER_START => (NODE_IDENTIFIER, TOKEN_IDENTIFIER_END),
            TOKEN_STRING_START => (NODE_STRING, TOKEN_STRING_END),
            TOKEN_RUNE_START => (NODE_RUNE, TOKEN_RUNE_END),
            _ => return None,
        })
    }
}
