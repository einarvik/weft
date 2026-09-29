//! Explicit, isolated client-island generation.

use thiserror::Error;

use crate::{Block, Document, Island, SectionChild, SectionKind, TextExpr, TextTerm};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Counter {
    pub label: TextExpr,
    pub initial: i64,
    pub minimum: i64,
    pub maximum: i64,
    pub text: TextExpr,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IslandError {
    #[error("line {line}: unsupported island `{name}`")]
    Unsupported { line: usize, name: String },
    #[error("line {line}: counter islands require `{field}`")]
    Missing { line: usize, field: &'static str },
    #[error("line {line}: counter range must target state `{state}`")]
    RangeStateMismatch { line: usize, state: String },
    #[error("line {line}: counter state must be within its range")]
    InitialOutOfRange { line: usize },
    #[error("line {line}: counter expressions may reference only state `{state}`")]
    InvalidExpression { line: usize, state: String },
    #[error("line {line}: unknown island reference `{name}`")]
    UnknownReference { line: usize, name: String },
}

/// Return whether a document has explicit client islands.
#[must_use]
pub fn has_islands(document: &Document) -> bool {
    mounted_islands(document).is_ok_and(|islands| !islands.is_empty())
}

/// Validate each island declaration in a document.
///
/// # Errors
///
/// Returns a line-numbered error when an island is unsupported or malformed.
pub fn validate(document: &Document) -> Result<(), IslandError> {
    for island in mounted_islands(document)? {
        let _ = counter(island)?;
    }
    Ok(())
}

fn mounted_islands(document: &Document) -> Result<Vec<&Island>, IslandError> {
    let mut islands = Vec::new();
    for page in &document.pages {
        for block in &page.blocks {
            match block {
                Block::Island(island) => islands.push(island),
                Block::Section(section) => {
                    if let SectionKind::Content { children } = &section.kind {
                        for child in children {
                            match child {
                                SectionChild::Island(island) => islands.push(island),
                                SectionChild::Use { name, line } => {
                                    let declaration =
                                        page.named_islands.get(name).ok_or_else(|| {
                                            IslandError::UnknownReference {
                                                line: *line,
                                                name: name.clone(),
                                            }
                                        })?;
                                    islands.push(&declaration.island);
                                }
                                SectionChild::Eyebrow(_)
                                | SectionChild::Title(_)
                                | SectionChild::Text(_) => {}
                            }
                        }
                    }
                }
                Block::Hero(_) => {}
            }
        }
    }
    Ok(islands)
}

/// Generate the small browser module required by the document's islands.
///
/// # Errors
///
/// Returns a line-numbered error when an island is unsupported or malformed.
pub fn render_javascript(document: &Document) -> Result<Option<String>, IslandError> {
    validate(document)?;
    if !has_islands(document) {
        return Ok(None);
    }
    Ok(Some(String::from(
        "customElements.define(\"weft-counter\",class extends HTMLElement{connectedCallback(){const n=this.dataset;const label=document.createElement(\"label\");const input=document.createElement(\"input\");input.type=\"range\";input.min=n.min;input.max=n.max;input.value=n.initial;const output=document.createElement(\"output\");const text=e=>JSON.parse(e).map(([k,v])=>k===\"l\"?v:k===\"s\"?input.value:String(Number(input.value)*Number(v))).join(\"\");const update=()=>{label.textContent=text(n.labelExpression);output.textContent=text(n.textExpression)};input.addEventListener(\"input\",update);this.append(label,input,output);update()}});\n",
    )))
}

/// Convert the supported `counter` island declaration to its safe runtime data.
///
/// # Errors
///
/// Returns a line-numbered error when a required field or the restricted text
/// expression is invalid.
pub fn counter(island: &Island) -> Result<Counter, IslandError> {
    if island.name != "counter" {
        return Err(IslandError::Unsupported {
            line: island.line,
            name: island.name.clone(),
        });
    }
    let state = required(island.state.as_ref(), island.line, "state")?;
    let range = required(island.range.as_ref(), island.line, "range")?;
    let text = required(island.text.as_ref(), island.line, "text")?;
    if range.name != state.name {
        return Err(IslandError::RangeStateMismatch {
            line: range.line,
            state: state.name.clone(),
        });
    }
    if !(range.minimum..=range.maximum).contains(&state.initial) {
        return Err(IslandError::InitialOutOfRange { line: state.line });
    }
    validate_expression(text, &state.name)?;
    let label = island.label.clone().unwrap_or(TextExpr {
        terms: vec![TextTerm::Literal("Count".to_owned())],
        line: island.line,
    });
    validate_expression(&label, &state.name)?;
    Ok(Counter {
        label,
        initial: state.initial,
        minimum: range.minimum,
        maximum: range.maximum,
        text: text.clone(),
    })
}

fn required<'value, T>(
    value: Option<&'value T>,
    line: usize,
    field: &'static str,
) -> Result<&'value T, IslandError> {
    value.ok_or(IslandError::Missing { line, field })
}

fn validate_expression(expression: &TextExpr, state: &str) -> Result<(), IslandError> {
    for term in &expression.terms {
        let identifier = match term {
            TextTerm::Literal(_) => continue,
            TextTerm::Identifier(identifier) | TextTerm::Multiply { identifier, .. } => identifier,
        };
        if identifier != state {
            return Err(IslandError::InvalidExpression {
                line: expression.line,
                state: state.to_owned(),
            });
        }
    }
    Ok(())
}

/// Serialize a restricted text expression for the generated island module.
#[must_use]
pub fn expression_json(expression: &TextExpr) -> String {
    let mut json = String::from("[");
    for (index, term) in expression.terms.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        match term {
            TextTerm::Literal(value) => {
                json.push_str("[\"l\",\"");
                json.push_str(&json_escape(value));
                json.push_str("\"]");
            }
            TextTerm::Identifier(_) => json.push_str("[\"s\",0]"),
            TextTerm::Multiply { factor, .. } => {
                json.push_str("[\"m\",");
                json.push_str(&factor.to_string());
                json.push(']');
            }
        }
    }
    json.push(']');
    json
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
