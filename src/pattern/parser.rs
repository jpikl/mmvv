use crate::pattern::char::Char;
use crate::pattern::error::{ParseError, ParseErrorKind, ParseResult};
use crate::pattern::lexer::{Lexer, Parsed, Token};
use crate::pattern::reader::Reader;
use crate::pattern::transform::Transform;
use crate::pattern::variable::Variable;

#[derive(Debug, PartialEq)]
pub enum PatternItem {
    Constant(String),
    Expression {
        variable: Parsed<Variable>,
        transforms: Vec<Parsed<Transform>>,
    },
}

pub struct Parser {
    lexer: Lexer,
    token: Option<Parsed<Token>>,
}

impl From<&str> for Parser {
    fn from(string: &str) -> Self {
        Self::new(Lexer::from(string))
    }
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self { lexer, token: None }
    }

    pub fn parse_items(&mut self) -> ParseResult<Vec<Parsed<PatternItem>>> {
        let mut items = Vec::new();

        while let Some(item) = self.parse_item()? {
            items.push(item);
        }

        Ok(items)
    }

    fn parse_item(&mut self) -> ParseResult<Option<Parsed<PatternItem>>> {
        if let Some(token) = self.fetch_token()? {
            match &token.value {
                Token::Raw(raw) => Ok(Some(Parsed {
                    value: PatternItem::Constant(Char::join(raw)),
                    start: token.start,
                    end: token.end,
                })),
                Token::ExprStart => {
                    let start = token.start;
                    let end = token.end;
                    let expression = self.parse_expression()?;

                    if let Some(Token::ExprEnd) = self.token_value() {
                        Ok(expression)
                    } else {
                        Err(ParseError {
                            kind: ParseErrorKind::UnterminatedExprStart,
                            start,
                            end,
                        })
                    }
                }
                Token::ExprEnd => Err(ParseError {
                    kind: ParseErrorKind::UnmatchedExprEnd,
                    start: token.start,
                    end: token.end,
                }),
                Token::Pipe => Err(ParseError {
                    kind: ParseErrorKind::PipeOutsideExpr,
                    start: token.start,
                    end: token.end,
                }),
            }
        } else {
            Ok(None)
        }
    }

    fn parse_expression(&mut self) -> ParseResult<Option<Parsed<PatternItem>>> {
        let start = self.token_start();
        let variable = self.parse_variable()?;
        let transforms = self.parse_transforms()?;
        let end = self.token_end();

        Ok(Some(Parsed {
            value: PatternItem::Expression {
                variable,
                transforms,
            },
            start,
            end,
        }))
    }

    fn parse_variable(&mut self) -> ParseResult<Parsed<Variable>> {
        self.parse_expression_member(Variable::parse, ParseErrorKind::ExpectedVariable)
    }

    fn parse_transforms(&mut self) -> ParseResult<Vec<Parsed<Transform>>> {
        let mut transforms: Vec<Parsed<Transform>> = Vec::new();

        while let Some(token) = self.fetch_token()? {
            match token.value {
                Token::Pipe => {
                    transforms.push(self.parse_transform()?);
                }
                Token::ExprStart => {
                    return Err(ParseError {
                        kind: ParseErrorKind::ExprStartInsideExpr,
                        start: token.start,
                        end: token.end,
                    })
                }
                Token::ExprEnd => {
                    break;
                }
                _ => {
                    panic!("Unexpected token {:?}", token); // Raw or anything else should never appear here!
                }
            }
        }

        Ok(transforms)
    }

    fn parse_transform(&mut self) -> ParseResult<Parsed<Transform>> {
        self.parse_expression_member(Transform::parse, ParseErrorKind::ExpectedTransform)
    }

    fn parse_expression_member<T, F: FnOnce(&mut Reader) -> ParseResult<T>>(
        &mut self,
        parse: F,
        error_kind: ParseErrorKind,
    ) -> ParseResult<Parsed<T>> {
        let position = self.token_end();
        let token = self.fetch_token()?.ok_or_else(|| ParseError {
            kind: error_kind.clone(),
            start: position,
            end: position,
        })?;
        if let Token::Raw(raw) = &token.value {
            let mut reader = Reader::new(raw.clone());
            let value = parse(&mut reader).map_err(|mut error| {
                error.start += position;
                error.end += position;
                error
            })?;
            if let Some(char) = reader.peek() {
                // There should be no remaining characters
                Err(ParseError {
                    kind: ParseErrorKind::ExpectedPipeOrExprEnd,
                    start: position + reader.position(),
                    end: position + reader.position() + char.len(),
                })
            } else {
                Ok(Parsed {
                    value,
                    start: token.start,
                    end: token.end,
                })
            }
        } else {
            Err(ParseError {
                kind: error_kind,
                start: token.start,
                end: token.end,
            })
        }
    }

    fn fetch_token(&mut self) -> ParseResult<Option<&Parsed<Token>>> {
        self.token = self.lexer.read_token()?;
        Ok(self.token.as_ref())
    }

    fn token_value(&self) -> Option<&Token> {
        self.token.as_ref().map(|token| &token.value)
    }

    fn token_start(&self) -> usize {
        self.token.as_ref().map_or(0, |token| token.start)
    }

    fn token_end(&self) -> usize {
        self.token.as_ref().map_or(0, |token| token.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::range::Range;
    use crate::pattern::substitution::Substitution;

    #[test]
    fn empty() {
        assert_eq!(Parser::from("").parse_items(), Ok(Vec::new()));
    }

    #[test]
    fn constant() {
        assert_eq!(
            Parser::from("a").parse_items(),
            Ok(vec![Parsed {
                value: PatternItem::Constant("a".to_string()),
                start: 0,
                end: 1,
            }])
        );
    }

    #[test]
    fn expected_variable_but_end_error() {
        assert_eq!(
            Parser::from("{").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedVariable,
                start: 1,
                end: 1,
            })
        );
    }

    #[test]
    fn expected_variable_but_pipe_error() {
        assert_eq!(
            Parser::from("{|").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedVariable,
                start: 1,
                end: 2,
            })
        );
    }

    #[test]
    fn pipe_outside_expr_error() {
        assert_eq!(
            Parser::from("|").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::PipeOutsideExpr,
                start: 0,
                end: 1,
            })
        );
    }

    #[test]
    fn expected_variable_but_expr_end_error() {
        assert_eq!(
            Parser::from("{}").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedVariable,
                start: 1,
                end: 2,
            })
        );
    }

    #[test]
    fn unmatched_expr_end_error() {
        assert_eq!(
            Parser::from("}").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::UnmatchedExprEnd,
                start: 0,
                end: 1,
            })
        );
    }

    #[test]
    fn unterminated_expr_start_after_variable_error() {
        assert_eq!(
            Parser::from("{f").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::UnterminatedExprStart,
                start: 0,
                end: 1,
            })
        );
    }

    #[test]
    fn variable() {
        assert_eq!(
            Parser::from("{f}").parse_items(),
            Ok(vec![Parsed {
                value: PatternItem::Expression {
                    variable: Parsed {
                        value: Variable::Filename,
                        start: 1,
                        end: 2,
                    },
                    transforms: Vec::new(),
                },
                start: 0,
                end: 3,
            }])
        );
    }

    #[test]
    fn unknown_variable_error() {
        assert_eq!(
            Parser::from("{x}").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::UnknownVariable(Char::Raw('x')),
                start: 1,
                end: 2,
            })
        );
    }

    #[test]
    fn expr_start_inside_expr_error() {
        assert_eq!(
            Parser::from("{f{").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExprStartInsideExpr,
                start: 2,
                end: 3,
            })
        );
    }

    #[test]
    fn expected_pipe_or_expr_end_after_variable_error() {
        assert_eq!(
            Parser::from("{fg").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedPipeOrExprEnd,
                start: 2,
                end: 3,
            })
        );
    }

    #[test]
    fn expected_transform_but_end_error() {
        assert_eq!(
            Parser::from("{f|").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedTransform,
                start: 3,
                end: 3,
            })
        );
    }

    #[test]
    fn expected_transform_but_pipe_error() {
        assert_eq!(
            Parser::from("{f||").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedTransform,
                start: 3,
                end: 4,
            })
        );
    }

    #[test]
    fn expected_transform_but_expr_end_error() {
        assert_eq!(
            Parser::from("{f|}").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedTransform,
                start: 3,
                end: 4,
            })
        );
    }

    #[test]
    fn unternimeted_expr_start_after_transform_error() {
        assert_eq!(
            Parser::from("{f|l").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::UnterminatedExprStart,
                start: 0,
                end: 1,
            })
        );
    }

    #[test]
    fn expected_pipe_or_expr_end_after_transform_error() {
        assert_eq!(
            Parser::from("{f|ll").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedPipeOrExprEnd,
                start: 4,
                end: 5,
            })
        );
    }

    #[test]
    fn variable_single_transform() {
        assert_eq!(
            Parser::from("{b|l}").parse_items(),
            Ok(vec![Parsed {
                value: PatternItem::Expression {
                    variable: Parsed {
                        value: Variable::Basename,
                        start: 1,
                        end: 2,
                    },
                    transforms: vec![Parsed {
                        value: Transform::Lowercase,
                        start: 3,
                        end: 4,
                    }],
                },
                start: 0,
                end: 5,
            }])
        );
    }

    #[test]
    fn variable_multiple_transforms() {
        assert_eq!(
            Parser::from("{e|t|n1-3}").parse_items(),
            Ok(vec![Parsed {
                value: PatternItem::Expression {
                    variable: Parsed {
                        value: Variable::Extension,
                        start: 1,
                        end: 2,
                    },
                    transforms: vec![
                        Parsed {
                            value: Transform::Trim,
                            start: 3,
                            end: 4,
                        },
                        Parsed {
                            value: Transform::Substring(Range {
                                offset: 0,
                                length: 3,
                            }),
                            start: 5,
                            end: 9,
                        },
                    ],
                },
                start: 0,
                end: 10,
            }])
        );
    }

    #[test]
    fn invalid_transform_error() {
        assert_eq!(
            Parser::from("{f|n2-1}").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::RangeEndBeforeStart(1, 2),
                start: 4,
                end: 7,
            })
        );
    }

    #[test]
    fn complex_input() {
        assert_eq!(
            Parser::from("image_{c|<000}.{e|l|r'e}2").parse_items(),
            Ok(vec![
                Parsed {
                    value: PatternItem::Constant("image_".to_string()),
                    start: 0,
                    end: 6,
                },
                Parsed {
                    value: PatternItem::Expression {
                        variable: Parsed {
                            value: Variable::LocalCounter,
                            start: 7,
                            end: 8,
                        },
                        transforms: vec![Parsed {
                            value: Transform::LeftPad("000".to_string()),
                            start: 9,
                            end: 13,
                        }],
                    },
                    start: 6,
                    end: 14,
                },
                Parsed {
                    value: PatternItem::Constant(".".to_string()),
                    start: 14,
                    end: 15,
                },
                Parsed {
                    value: PatternItem::Expression {
                        variable: Parsed {
                            value: Variable::Extension,
                            start: 16,
                            end: 17,
                        },
                        transforms: vec![
                            Parsed {
                                value: Transform::Lowercase,
                                start: 18,
                                end: 19,
                            },
                            Parsed {
                                value: Transform::ReplaceFirst(Substitution {
                                    value: 'e'.to_string(),
                                    replacement: String::new(),
                                }),
                                start: 20,
                                end: 23,
                            },
                        ],
                    },
                    start: 15,
                    end: 24,
                },
                Parsed {
                    value: PatternItem::Constant("2".to_string()),
                    start: 24,
                    end: 25,
                },
            ])
        );
    }
}
