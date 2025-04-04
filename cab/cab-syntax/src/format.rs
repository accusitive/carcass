//! Formatting utilities for [`node::Expression`]s.
use std::{
    fmt,
    io,
};

use yansi::Paint as _;

use crate::{
    COLORS,
    node::{
        self,
        Parted as _,
    },
};

/// Formats the given node with parentheses to disambiguate.
pub fn parenthesize(writer: &mut impl io::Write, expression: node::ExpressionRef<'_>) -> io::Result<()> {
    Formatter::new(writer).parenthesize(expression)
}

#[derive(Debug)]
struct Formatter<'write, W: io::Write> {
    inner: &'write mut W,

    bracket_count: usize,
}

impl<'write, W: io::Write> Formatter<'write, W> {
    fn new(inner: &'write mut W) -> Self {
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

    fn parenthesize_parted<'part>(
        &mut self,
        parts: impl Iterator<Item = node::InterpolatedPartRef<'part>>,
    ) -> io::Result<()> {
        for part in parts {
            match part {
                node::InterpolatedPartRef::Delimiter(token) => {
                    self.write(token.text().green().bold())?;
                },

                node::InterpolatedPartRef::Content(token) => {
                    self.write(token.text().green())?;
                },

                node::InterpolatedPartRef::Interpolation(interpolation) => {
                    self.write(r"\(".yellow())?;
                    self.parenthesize(interpolation.expression())?;
                    self.write(")".yellow())?;
                },
            }
        }

        Ok(())
    }

    fn parenthesize(&mut self, expression: node::ExpressionRef<'_>) -> io::Result<()> {
        match expression {
            node::ExpressionRef::Error(_error) => self.write("error".red().bold()),

            node::ExpressionRef::Parenthesis(parenthesis) => {
                if let Some(expression) = parenthesis.expression() {
                    self.parenthesize(expression)?;
                }

                Ok(())
            },

            node::ExpressionRef::List(list) => {
                self.bracket_start("[")?;

                let mut items = list.items().peekable();
                if items.peek().is_some() {
                    self.write(" ")?;
                }

                while let Some(item) = items.next() {
                    self.parenthesize(item)?;

                    if items.peek().is_some() {
                        self.write(",")?;
                    }

                    self.write(" ")?;
                }

                self.bracket_end("]")
            },

            node::ExpressionRef::Attributes(attributes) => {
                self.bracket_start("{")?;

                if let Some(expression) = attributes.expression() {
                    self.write(" ")?;
                    self.parenthesize(expression)?;
                    self.write(" ")?;
                }

                self.bracket_end("}")
            },

            node::ExpressionRef::PrefixOperation(operation) => {
                self.bracket_start("(")?;

                self.write(match operation.operator() {
                    node::PrefixOperator::Swwallation => "+",
                    node::PrefixOperator::Negation => "-",

                    node::PrefixOperator::Not => "!",

                    node::PrefixOperator::Try => "?",
                })?;
                self.write(" ")?;
                self.parenthesize(operation.right())?;

                self.bracket_end(")")
            },

            node::ExpressionRef::InfixOperation(operation) => {
                self.bracket_start("(")?;

                let operator = match operation.operator() {
                    node::InfixOperator::Select => Some("."),

                    node::InfixOperator::Same => Some(","),
                    node::InfixOperator::Sequence => Some(";"),

                    node::InfixOperator::ImplicitApply | node::InfixOperator::Apply => None,
                    node::InfixOperator::Pipe => {
                        self.parenthesize(operation.right())?;
                        self.write(" ")?;
                        self.parenthesize(operation.left())?;

                        return self.bracket_end(")");
                    },

                    node::InfixOperator::Concat => Some("++"),
                    node::InfixOperator::Construct => Some(":"),

                    node::InfixOperator::Update => Some("//"),

                    node::InfixOperator::LessOrEqual => Some("<="),
                    node::InfixOperator::Less => Some("<"),
                    node::InfixOperator::MoreOrEqual => Some(">="),
                    node::InfixOperator::More => Some(">"),

                    node::InfixOperator::Equal => Some("="),
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

                    node::InfixOperator::Lambda => Some("=>"),
                };

                self.parenthesize(operation.left())?;
                self.write(" ")?;

                if let Some(operator) = operator {
                    self.write(operator)?;
                    self.write(" ")?;
                }

                self.parenthesize(operation.right())?;

                self.bracket_end(")")
            },

            node::ExpressionRef::SuffixOperation(operation) => {
                self.bracket_start("(")?;

                self.parenthesize(operation.left())?;
                self.write(" ")?;
                self.write(match operation.operator() {
                    node::SuffixOperator::Same => ",",
                    node::SuffixOperator::Sequence => ";",
                })?;

                self.bracket_end(")")
            },

            node::ExpressionRef::Path(path) => self.parenthesize_parted(path.parts()),

            node::ExpressionRef::Bind(bind) => {
                self.write("@")?;
                self.parenthesize(bind.identifier())
            },

            node::ExpressionRef::Identifier(identifier) => {
                match identifier.value() {
                    node::IdentifierValueRef::Plain(token) => {
                        self.write(match token.text() {
                            boolean @ ("true" | "false") => boolean.magenta().bold(),
                            inexistent @ ("null" | "undefined") => inexistent.cyan().bold(),
                            import @ "import" => import.yellow().bold(),
                            identifier => identifier.new(),
                        })
                    },

                    node::IdentifierValueRef::Quoted(quoted) => self.parenthesize_parted(quoted.parts()),
                }
            },

            node::ExpressionRef::SString(string) => self.parenthesize_parted(string.parts()),

            node::ExpressionRef::Rune(rune) => self.parenthesize_parted(rune.parts()),

            node::ExpressionRef::Island(_island) => {
                todo!();
                // self.parenthesize_parted(island.parts())
            },

            node::ExpressionRef::Integer(integer) => self.write(integer.value().blue().bold()),
            node::ExpressionRef::Float(float) => self.write(float.value().blue().bold()),

            node::ExpressionRef::If(if_) => {
                self.bracket_start("(")?;

                self.write("if ".red().bold())?;
                self.parenthesize(if_.condition())?;
                self.write(" then ".red().bold())?;
                self.parenthesize(if_.consequence())?;
                self.write(" else ".red().bold())?;
                self.parenthesize(if_.alternative())?;

                self.bracket_end(")")
            },
        }
    }
}
