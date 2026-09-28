//! Explicit, isolated client-island generation.

use thiserror::Error;

use crate::{Block, Document, Island, Text};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Counter {
    pub label: String,
    pub initial: i64,
    pub minimum: i64,
    pub maximum: i64,
    pub prefix: String,
    pub multiplier: i64,
    pub suffix: String,
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
    #[error("line {line}: counter text must use `\"prefix\" + state * number + \"suffix\"`")]
    InvalidExpression { line: usize },
}

/// Return whether a document has explicit client islands.
#[must_use]
pub fn has_islands(document: &Document) -> bool {
    document.pages.iter().any(|page| {
        page.blocks
            .iter()
            .any(|block| matches!(block, Block::Island(_)))
    })
}

/// Validate each island declaration in a document.
///
/// # Errors
///
/// Returns a line-numbered error when an island is unsupported or malformed.
pub fn validate(document: &Document) -> Result<(), IslandError> {
    for island in document.pages.iter().flat_map(|page| &page.blocks) {
        if let Block::Island(island) = island {
            let _ = counter(island)?;
        }
    }
    Ok(())
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
        "customElements.define(\"weft-counter\",class extends HTMLElement{connectedCallback(){const n=this.dataset;const label=document.createElement(\"label\");label.textContent=n.label;const input=document.createElement(\"input\");input.type=\"range\";input.min=n.min;input.max=n.max;input.value=n.initial;const output=document.createElement(\"output\");const update=()=>{output.textContent=`${n.prefix}${Number(input.value)*Number(n.multiplier)}${n.suffix}`};input.addEventListener(\"input\",update);this.append(label,input,output);update()}});\n",
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
    let (prefix, variable, multiplier, suffix) = expression(text)?;
    if variable != state.name {
        return Err(IslandError::InvalidExpression { line: text.line });
    }
    Ok(Counter {
        label: island
            .label
            .as_ref()
            .map_or_else(|| "Count".to_owned(), |text| text.value.clone()),
        initial: state.initial,
        minimum: range.minimum,
        maximum: range.maximum,
        prefix,
        multiplier,
        suffix,
    })
}

fn required<'value, T>(
    value: Option<&'value T>,
    line: usize,
    field: &'static str,
) -> Result<&'value T, IslandError> {
    value.ok_or(IslandError::Missing { line, field })
}

fn expression(text: &Text) -> Result<(String, String, i64, String), IslandError> {
    let parts: Vec<_> = text.value.split(" + ").collect();
    let [prefix, product, suffix] = parts.as_slice() else {
        return Err(IslandError::InvalidExpression { line: text.line });
    };
    let (variable, multiplier) = product
        .split_once(" * ")
        .ok_or(IslandError::InvalidExpression { line: text.line })?;
    Ok((
        string_literal(prefix, text.line)?,
        variable.to_owned(),
        multiplier
            .parse()
            .map_err(|_| IslandError::InvalidExpression { line: text.line })?,
        string_literal(suffix, text.line)?,
    ))
}

fn string_literal(value: &str, line: usize) -> Result<String, IslandError> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(str::to_owned)
        .ok_or(IslandError::InvalidExpression { line })
}
