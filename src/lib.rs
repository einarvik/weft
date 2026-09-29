//! The Weft source parser.
//!
//! The parser deliberately accepts a small, line-oriented grammar. It captures
//! indentation and reports the source line for every syntax error so compiler
//! stages never need to guess at malformed intent.

use std::collections::BTreeMap;

use thiserror::Error;

pub mod island;
pub mod render;
pub mod style;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub site: Option<Site>,
    pub theme: BTreeMap<String, String>,
    pub pages: Vec<Page>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Site {
    pub name: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub path: String,
    pub line: usize,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Hero(Hero),
    Section(Section),
    Island(Island),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hero {
    pub line: usize,
    pub eyebrow: Option<TextExpr>,
    pub title: Option<TextExpr>,
    pub text: Option<TextExpr>,
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub line: usize,
    pub name: String,
    pub cards: usize,
    pub style: Option<String>,
    pub items: Vec<Card>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Island {
    pub line: usize,
    pub name: String,
    pub label: Option<TextExpr>,
    pub state: Option<State>,
    pub range: Option<Range>,
    pub text: Option<TextExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextExpr {
    pub terms: Vec<TextTerm>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextTerm {
    Literal(String),
    Identifier(String),
    Multiply { identifier: String, factor: i64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub label: TextExpr,
    pub destination: String,
    pub variant: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub title: TextExpr,
    pub description: TextExpr,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub name: String,
    pub initial: i64,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    pub name: String,
    pub minimum: i64,
    pub maximum: i64,
    pub line: usize,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("line {line}: {message}")]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
struct Line<'source> {
    number: usize,
    level: usize,
    content: &'source str,
}

/// Parse a complete `.wft` document.
///
/// # Errors
///
/// Returns a line-numbered [`ParseError`] when the source violates the
/// indentation or statement grammar.
pub fn parse(source: &str) -> Result<Document, ParseError> {
    let lines = source_lines(source)?;
    let mut parser = Parser { lines, cursor: 0 };
    parser.document()
}

fn source_lines(source: &str) -> Result<Vec<Line<'_>>, ParseError> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, raw)| {
            let number = index + 1;
            if raw.trim().is_empty() {
                return None;
            }
            Some((number, raw))
        })
        .map(|(number, raw)| {
            if raw.contains('\t') {
                return Err(error(
                    number,
                    "tabs are not allowed; indent with two spaces",
                ));
            }
            let spaces = raw.len() - raw.trim_start_matches(' ').len();
            if spaces % 2 != 0 {
                return Err(error(number, "indentation must use two-space levels"));
            }
            Ok(Line {
                number,
                level: spaces / 2,
                content: raw.trim(),
            })
        })
        .collect()
}

struct Parser<'source> {
    lines: Vec<Line<'source>>,
    cursor: usize,
}

impl<'source> Parser<'source> {
    fn document(&mut self) -> Result<Document, ParseError> {
        let mut document = Document {
            site: None,
            theme: BTreeMap::new(),
            pages: Vec::new(),
        };

        while let Some(line) = self.peek() {
            Self::require_level(line, 0)?;
            let line = self.next().expect("a peeked line must exist");
            if let Some(name) = line.content.strip_prefix("site ") {
                document.site = Some(Site {
                    name: non_empty(name, line.number, "site name")?.to_owned(),
                    line: line.number,
                });
            } else if let Some(tokens) = line.content.strip_prefix("theme:") {
                document.theme.extend(parse_theme(tokens, line.number)?);
            } else if let Some(path) = line
                .content
                .strip_prefix("page ")
                .and_then(|value| value.strip_suffix(':'))
            {
                document.pages.push(self.page(
                    non_empty(path, line.number, "page path")?.to_owned(),
                    line.number,
                )?);
            } else {
                return Err(error(
                    line.number,
                    "expected `site`, `theme:`, or `page /:`",
                ));
            }
        }

        Ok(document)
    }

    fn page(&mut self, path: String, line: usize) -> Result<Page, ParseError> {
        let mut blocks = Vec::new();
        while self.at_child_level(1) {
            let line = self.next().expect("child line must exist");
            if line.content == "hero:" {
                blocks.push(Block::Hero(self.hero(line.number)?));
            } else if let Some(header) = line.content.strip_prefix("section ") {
                blocks.push(Block::Section(self.section(header, line.number)?));
            } else if let Some(name) = line
                .content
                .strip_prefix("island ")
                .and_then(|value| value.strip_suffix(':'))
            {
                blocks.push(Block::Island(self.island(
                    non_empty(name, line.number, "island name")?.to_owned(),
                    line.number,
                )?));
            } else {
                return Err(error(
                    line.number,
                    "expected `hero:`, `section …:`, or `island …:`",
                ));
            }
        }
        self.reject_deeper_than(0)?;
        Ok(Page { path, line, blocks })
    }

    fn hero(&mut self, line: usize) -> Result<Hero, ParseError> {
        let mut hero = Hero {
            line,
            eyebrow: None,
            title: None,
            text: None,
            actions: Vec::new(),
        };
        while self.at_child_level(2) {
            let child = self.next().expect("child line must exist");
            match child.content {
                content if content.starts_with("eyebrow ") => {
                    hero.eyebrow = Some(text_value(content, "eyebrow", child.number)?);
                }
                content if content.starts_with("title ") => {
                    hero.title = Some(text_value(content, "title", child.number)?);
                }
                content if content.starts_with("text ") => {
                    hero.text = Some(text_value(content, "text", child.number)?);
                }
                "actions:" => hero.actions = self.actions()?,
                _ => return Err(error(child.number, "expected hero content or `actions:`")),
            }
        }
        self.reject_deeper_than(1)?;
        Ok(hero)
    }

    fn actions(&mut self) -> Result<Vec<Action>, ParseError> {
        let mut actions = Vec::new();
        while self.at_child_level(3) {
            let line = self.next().expect("child line must exist");
            actions.push(parse_action(line.content, line.number)?);
        }
        self.reject_deeper_than(2)?;
        Ok(actions)
    }

    fn section(&mut self, header: &str, line: usize) -> Result<Section, ParseError> {
        let header = header
            .strip_suffix(':')
            .ok_or_else(|| error(line, "section declarations must end with `:`"))?;
        let (header, style) = split_style(header, line)?;
        let mut words = header.split_whitespace();
        let name = words
            .next()
            .ok_or_else(|| error(line, "section name is required"))?;
        if words.next() != Some("cards") {
            return Err(error(line, "section syntax is `section name cards N:`"));
        }
        let cards = words
            .next()
            .ok_or_else(|| error(line, "card count is required"))?
            .parse()
            .map_err(|_| error(line, "card count must be a positive integer"))?;
        if cards == 0 || words.next().is_some() {
            return Err(error(line, "section syntax is `section name cards N:`"));
        }

        let mut items = Vec::new();
        while self.at_child_level(2) {
            let child = self.next().expect("child line must exist");
            items.push(parse_card(child.content, child.number)?);
        }
        self.reject_deeper_than(1)?;
        Ok(Section {
            line,
            name: name.to_owned(),
            cards,
            style,
            items,
        })
    }

    fn island(&mut self, name: String, line: usize) -> Result<Island, ParseError> {
        let mut island = Island {
            line,
            name,
            label: None,
            state: None,
            range: None,
            text: None,
        };
        while self.at_child_level(2) {
            let child = self.next().expect("child line must exist");
            match child.content {
                content if content.starts_with("label ") => {
                    island.label = Some(text_value(content, "label", child.number)?);
                }
                content if content.starts_with("state ") => {
                    island.state = Some(parse_state(content, child.number)?);
                }
                content if content.starts_with("range ") => {
                    island.range = Some(parse_range(content, child.number)?);
                }
                content if content.starts_with("text ") => {
                    island.text = Some(text_value(content, "text", child.number)?);
                }
                _ => return Err(error(child.number, "expected island content")),
            }
        }
        self.reject_deeper_than(1)?;
        Ok(island)
    }

    fn at_child_level(&self, level: usize) -> bool {
        self.peek().is_some_and(|line| line.level == level)
    }

    fn reject_deeper_than(&self, level: usize) -> Result<(), ParseError> {
        if let Some(line) = self.peek().filter(|line| line.level > level) {
            return Err(error(line.number, "unexpected indentation"));
        }
        Ok(())
    }

    fn require_level(line: &Line<'_>, level: usize) -> Result<(), ParseError> {
        if line.level != level {
            return Err(error(line.number, "unexpected indentation"));
        }
        Ok(())
    }

    fn peek(&self) -> Option<&Line<'source>> {
        self.lines.get(self.cursor)
    }

    fn next(&mut self) -> Option<Line<'source>> {
        let line = self.lines.get(self.cursor).copied();
        self.cursor += usize::from(line.is_some());
        line
    }
}

fn parse_theme(tokens: &str, line: usize) -> Result<BTreeMap<String, String>, ParseError> {
    tokens
        .split(';')
        .filter(|token| !token.trim().is_empty())
        .map(|token| {
            let mut words = token.split_whitespace();
            let name = words
                .next()
                .ok_or_else(|| error(line, "theme token name is required"))?;
            let value = words
                .next()
                .ok_or_else(|| error(line, "theme token value is required"))?;
            if words.next().is_some() {
                return Err(error(line, "theme tokens use `name value` pairs"));
            }
            Ok((name.to_owned(), value.to_owned()))
        })
        .collect()
}

fn text_value(content: &str, keyword: &str, line: usize) -> Result<TextExpr, ParseError> {
    let value = content
        .strip_prefix(keyword)
        .expect("called only after checking the keyword prefix")
        .trim();
    let (expression, rest) = parse_text_expression(value, line)?;
    if !rest.trim().is_empty() {
        return Err(error(line, "unexpected text after expression"));
    }
    Ok(expression)
}

fn parse_action(content: &str, line: usize) -> Result<Action, ParseError> {
    let (label, rest) = parse_text_expression(content, line)?;
    let rest = rest
        .strip_prefix(" -> ")
        .ok_or_else(|| error(line, "action syntax is `TextExpr -> /path variant`"))?;
    let mut words = rest.split_whitespace();
    let destination = words
        .next()
        .ok_or_else(|| error(line, "action destination is required"))?;
    let variant = words
        .next()
        .ok_or_else(|| error(line, "action variant is required"))?;
    if words.next().is_some() {
        return Err(error(line, "action syntax is `TextExpr -> /path variant`"));
    }
    Ok(Action {
        label,
        destination: destination.to_owned(),
        variant: variant.to_owned(),
        line,
    })
}

fn parse_card(content: &str, line: usize) -> Result<Card, ParseError> {
    let rest = content
        .strip_prefix("card ")
        .ok_or_else(|| error(line, "expected `card TextExpr TextExpr`"))?;
    let (title, rest) = parse_text_expression(rest, line)?;
    let (description, rest) = parse_text_expression(rest.trim_start(), line)?;
    if !rest.trim().is_empty() {
        return Err(error(line, "card syntax is `card TextExpr TextExpr`"));
    }
    Ok(Card {
        title,
        description,
        line,
    })
}

fn parse_state(content: &str, line: usize) -> Result<State, ParseError> {
    let assignment = content
        .strip_prefix("state ")
        .expect("called only after checking the state prefix");
    let (name, initial) = assignment
        .split_once('=')
        .ok_or_else(|| error(line, "state syntax is `state name=number`"))?;
    Ok(State {
        name: non_empty(name, line, "state name")?.to_owned(),
        initial: initial
            .parse()
            .map_err(|_| error(line, "state value must be an integer"))?,
        line,
    })
}

fn parse_range(content: &str, line: usize) -> Result<Range, ParseError> {
    let value = content
        .strip_prefix("range ")
        .expect("called only after checking the range prefix");
    let (name, bounds) = value
        .split_once(' ')
        .ok_or_else(|| error(line, "range syntax is `range name minimum..maximum`"))?;
    let (minimum, maximum) = bounds
        .split_once("..")
        .ok_or_else(|| error(line, "range syntax is `range name minimum..maximum`"))?;
    let minimum = minimum
        .parse()
        .map_err(|_| error(line, "range minimum must be an integer"))?;
    let maximum = maximum
        .parse()
        .map_err(|_| error(line, "range maximum must be an integer"))?;
    if minimum > maximum {
        return Err(error(line, "range minimum cannot exceed maximum"));
    }
    Ok(Range {
        name: non_empty(name, line, "range state name")?.to_owned(),
        minimum,
        maximum,
        line,
    })
}

fn split_style(header: &str, line: usize) -> Result<(&str, Option<String>), ParseError> {
    match header.split_once(" @=") {
        None => Ok((header, None)),
        Some((main, style)) => Ok((main, Some(quoted(style, line)?.to_owned()))),
    }
}

fn quoted(value: &str, line: usize) -> Result<&str, ParseError> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .ok_or_else(|| error(line, "expected a double-quoted string"))
}

fn take_quoted(value: &str, line: usize) -> Result<(&str, &str), ParseError> {
    let value = value
        .strip_prefix('"')
        .ok_or_else(|| error(line, "expected a double-quoted string"))?;
    let (quoted, rest) = value
        .split_once('"')
        .ok_or_else(|| error(line, "unterminated double-quoted string"))?;
    Ok((quoted, rest))
}

fn parse_text_expression(value: &str, line: usize) -> Result<(TextExpr, &str), ParseError> {
    let mut rest = value.trim_start();
    let mut terms = vec![parse_text_term(&mut rest, line)?];
    while let Some(next) = rest.strip_prefix(" + ") {
        rest = next;
        terms.push(parse_text_term(&mut rest, line)?);
    }
    Ok((TextExpr { terms, line }, rest))
}

fn parse_text_term(rest: &mut &str, line: usize) -> Result<TextTerm, ParseError> {
    if rest.starts_with('"') {
        let (literal, remaining) = take_quoted(rest, line)?;
        *rest = remaining;
        return Ok(TextTerm::Literal(literal.to_owned()));
    }

    let end = rest.find(" + ").unwrap_or(rest.len());
    let term = rest[..end].trim_end();
    *rest = &rest[end..];
    if term.is_empty() {
        return Err(error(line, "expected a text expression term"));
    }
    if let Some((identifier, factor)) = term.split_once(" * ") {
        return Ok(TextTerm::Multiply {
            identifier: identifier_name(identifier, line)?.to_owned(),
            factor: factor
                .parse()
                .map_err(|_| error(line, "text expression multiplier must be an integer"))?,
        });
    }
    Ok(TextTerm::Identifier(
        identifier_name(term, line)?.to_owned(),
    ))
}

fn identifier_name(value: &str, line: usize) -> Result<&str, ParseError> {
    let valid = value.chars().enumerate().all(|(index, character)| {
        character == '_'
            || character.is_ascii_alphanumeric() && (index > 0 || !character.is_ascii_digit())
    });
    if valid {
        Ok(value)
    } else {
        Err(error(
            line,
            "text expression identifiers use letters, digits, and underscores",
        ))
    }
}

fn non_empty<'value>(
    value: &'value str,
    line: usize,
    label: &str,
) -> Result<&'value str, ParseError> {
    let value = value.trim();
    if value.is_empty() {
        Err(error(line, format!("{label} is required")))
    } else {
        Ok(value)
    }
}

fn error(line: usize, message: impl Into<String>) -> ParseError {
    ParseError {
        line,
        message: message.into(),
    }
}
