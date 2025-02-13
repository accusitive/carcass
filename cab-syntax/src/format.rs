//! Formatting utilities for [`node::Expression`]s.
use std::{
    fmt,
    io,
};

use yansi::Paint as _;

use crate::{
    COLORS,
    node,
    token,
};

/// Formats the given node with parentheses to disambiguate.
/// The node must be a valid [`node::Expression`] or this will panic.
pub fn parenthesize(writer: &mut impl io::Write, expression: node::Expression) -> io::Result<()> {
    Formatter::new(writer).parenthesize(expression)
}

#[derive(Debug)]
struct Formatter<'a, W: io::Write> {
    inner: &'a mut W,

    bracket_count: usize,
}

impl<'a, W: io::Write> Formatter<'a, W> {
    fn new(inner: &'a mut W) -> Self {
        Self {
            inner,

            bracket_count: 0,
        }
    }

    fn paint_bracket<'b>(&self, bracket: &'b str) -> yansi::Painted<&'b str> {
        let style = COLORS[self.bracket_count % COLORS.len()];
        bracket.paint(style)
    }

    fn bracket_start(&mut self, bracket: &str) -> io::Result<()> {
        write!(self.inner, "{painted}", painted = self.paint_bracket(bracket))?;
        self.bracket_count += 1;

        Ok(())
    }

    fn bracket_end(&mut self, bracket: &str) -> io::Result<()> {
        self.bracket_count -= 1;
        write!(self.inner, "{painted}", painted = self.paint_bracket(bracket))
    }

    fn write(&mut self, painted: impl fmt::Display) -> io::Result<()> {
        write!(self.inner, "{painted}")
    }

    fn parenthesize_parted<T: token::Token>(
        &mut self,
        parts: impl Iterator<Item = node::InterpolatedPart<T>>,
    ) -> io::Result<()> {
        for part in parts {
            match part {
                node::InterpolatedPart::Delimiter(token) => {
                    self.write(token.text().green().bold())?;
                },

                node::InterpolatedPart::Content(token) => {
                    self.write(token.text().green())?;
                },

                node::InterpolatedPart::Interpolation(interpolation) => {
                    self.write(r"\(".yellow())?;
                    self.parenthesize(&interpolation.expression())?;
                    self.write(")".yellow())?;
                },
            }
        }

        Ok(())
    }

    fn parenthesize(&mut self, expression: node::Expression) -> io::Result<()> {
        node::r#match! { node =>
            node::Error as _error => {
                self.write("error".red().bold())
            },

            node::Parenthesis as parenthesis => {
                self.parenthesize(&parenthesis.expression())
            },

            node::List as list => {
                self.bracket_start("[")?;

                let mut items = list.items().peekable();
                if items.peek().is_some() {
                    self.write(" ")?;
                }

                while let Some(item) = items.next() {
                    self.parenthesize(&item)?;

                    if items.peek().is_some() {
                        self.write(",")?;
                    }

                    self.write(" ")?;
                }

                self.bracket_end("]")
            },

            node::AttributeList as attribute_list => {
                self.bracket_start("{")?;

                let mut entries = attribute_list.entries().peekable();
                if entries.peek().is_some() {
                    self.write(" ")?;
                }

                while let Some(entry) = entries.next() {
                    self.parenthesize(&entry)?;

                    if entries.peek().is_some() {
                        self.write(",")?;
                    }

                    self.write(" ")?;
                }

                self.bracket_end("}")
            },

            node::PrefixOperation as operation => {
                self.bracket_start("(")?;

                self.write(match operation.operator() {
                    node::PrefixOperator::Swwallation => "+",
                    node::PrefixOperator::Negation => "-",

                    node::PrefixOperator::Not => "!",

                    node::PrefixOperator::Try => "?",
                })?;
                self.write(" ")?;
                self.parenthesize(&operation.right())?;

                self.bracket_end(")")
            },

            node::InfixOperation as operation => {
                self.bracket_start("(")?;

                let operator = match operation.operator() {
                    node::InfixOperator::Select => Some("."),

                    node::InfixOperator::Same => Some(","),
                    node::InfixOperator::Sequence => Some(";"),

                    node::InfixOperator::ImplicitApply | node::InfixOperator::Apply => None,
                    node::InfixOperator::Pipe => {
                        self.parenthesize(&operation.right())?;
                        self.write(" ")?;
                        self.parenthesize(&operation.left())?;

                        return self.bracket_end(")");
                    },

                    node::InfixOperator::Concat => Some("++"),
                    node::InfixOperator::Construct => Some(":"),

                    node::InfixOperator::Update => Some("//"),

                    node::InfixOperator::LessOrEqual => Some("<="),
                    node::InfixOperator::Less => Some("<"),
                    node::InfixOperator::MoreOrEqual => Some(">="),
                    node::InfixOperator::More => Some(">"),

                    node::InfixOperator::Equal => Some("=="),
                    node::InfixOperator::NotEqual => Some("!="),

                    node::InfixOperator::Addition => Some("+"),
                    node::InfixOperator::Subtraction => Some("-"),
                    node::InfixOperator::Multiplication => Some("*"),
                    node::InfixOperator::Power => Some("^"),
                    node::InfixOperator::Division => Some("/"),

                    node::InfixOperator::And => Some("&&"),
                    node::InfixOperator::Or => Some("||"),
                    node::InfixOperator::Implication => Some("->"),

                    node::InfixOperator::All => Some("&"),
                    node::InfixOperator::Any => Some("|"),

                    node::InfixOperator::This => Some("@"),
                    node::InfixOperator::Lambda => Some("=>"),
                    node::InfixOperator::Bind => Some(":="),
                };

                self.parenthesize(&operation.left())?;
                self.write(" ")?;

                if let Some(operator) = operator {
                    self.write(operator)?;
                    self.write(" ")?;
                }

                self.parenthesize(&operation.right())?;

                self.bracket_end(")")
            },

            node::SuffixOperation as operation => {
                self.bracket_start("(")?;

                self.parenthesize(&operation.left())?;
                self.write(" ")?;
                self.write(match operation.operator() {
                    node::SuffixOperator::Same => ",",
                    node::SuffixOperator::Sequence => ";",
                })?;

                self.bracket_end(")")
            },

            node::Path as path => self.parenthesize_parted(path.parts()),

            node::Identifier as identifier => {
                match identifier.value() {
                    node::IdentifierValue::Plain(token) => self.write(match token.text() {
                        boolean @ ("true" | "false") => boolean.magenta().bold(),
                        inexistent @ ("null" | "undefined") => inexistent.cyan().bold(),
                        import @ "import" => import.yellow().bold(),
                        identifier => identifier.new(),
                    }),

                    node::IdentifierValue::Quoted(quoted) => self.parenthesize_parted(quoted.parts()),
                }
            },

            node::SString as string => self.parenthesize_parted(string.parts()),

            node::Rune as rune => self.parenthesize_parted(rune.parts()),

            node::Island as island => self.parenthesize_parted(island.parts()),

            node::Number as number => {
                match number.value() {
                    node::NumberValue::Integer(token) => self.write(token.value().blue().bold()),
                    node::NumberValue::Float(token) => self.write(token.value().blue().bold()),
                }
            },

            node::IfThen as if_then => {
                self.bracket_start("(")?;

                self.write("if ".red().bold())?;
                self.parenthesize(&if_then.condition())?;
                self.write(" then ".red().bold())?;
                self.parenthesize(&if_then.consequence())?;

                if let Some(alternative) = if_then.alternative() {
                    self.write(" else ".red().bold())?;
                    self.parenthesize(&alternative)?;
                }

                self.bracket_end(")")
            },

            node::IfIs as if_is => {
                self.bracket_start("(")?;

                self.write("if ".red().bold())?;
                self.parenthesize(&if_is.expression())?;
                self.write(" is ".red().bold())?;
                self.parenthesize(&if_is.patterns())?;

                self.bracket_end(")")
            },

            else => unreachable!(),
        }
    }
}
