use std::collections::HashMap;

use crate::error::PromptCacheError;

#[derive(Debug, Clone, Default)]
pub struct TemplateRenderer {
    strict_mode: bool,
}

impl TemplateRenderer {
    pub fn new() -> Self {
        Self { strict_mode: false }
    }

    pub fn strict() -> Self {
        Self { strict_mode: true }
    }

    pub fn with_strict_mode(mut self, strict_mode: bool) -> Self {
        self.strict_mode = strict_mode;
        self
    }

    pub fn render(
        &self,
        template: &str,
        context: &HashMap<String, String>,
    ) -> Result<String, PromptCacheError> {
        let placeholders = parse_placeholders(template)?;
        if placeholders.is_empty() {
            return Ok(template.to_string());
        }

        let mut rendered = String::with_capacity(template.len());
        let mut cursor = 0;

        for placeholder in placeholders {
            rendered.push_str(&template[cursor..placeholder.start]);
            if let Some(value) = context.get(&placeholder.name) {
                rendered.push_str(value);
            } else if self.strict_mode {
                return Err(PromptCacheError::TemplateRenderError(format!(
                    "missing template variable '{}'",
                    placeholder.name
                )));
            } else {
                rendered.push_str(&template[placeholder.start..placeholder.end]);
            }
            cursor = placeholder.end;
        }

        rendered.push_str(&template[cursor..]);
        Ok(rendered)
    }

    pub fn extract_variables(&self, template: &str) -> Result<Vec<String>, PromptCacheError> {
        extract_template_variables(template)
    }
}

pub fn is_valid_variable_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

pub fn extract_template_variables(template: &str) -> Result<Vec<String>, PromptCacheError> {
    let placeholders = parse_placeholders(template)?;
    let mut seen = std::collections::HashSet::new();
    let mut variables = Vec::new();

    for placeholder in placeholders {
        if seen.insert(placeholder.name.clone()) {
            variables.push(placeholder.name);
        }
    }

    Ok(variables)
}

#[derive(Debug, Clone)]
struct Placeholder {
    start: usize,
    end: usize,
    name: String,
}

fn parse_placeholders(template: &str) -> Result<Vec<Placeholder>, PromptCacheError> {
    let bytes = template.as_bytes();
    let mut placeholders = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'{' && index + 1 < bytes.len() && bytes[index + 1] == b'{' {
            let prev_is_open = index > 0 && bytes[index - 1] == b'{';
            let next_is_open = index + 2 < bytes.len() && bytes[index + 2] == b'{';

            if !prev_is_open && !next_is_open {
                match find_placeholder_close(bytes, index + 2) {
                    Some(close_index) => {
                        let next_is_close =
                            close_index + 2 < bytes.len() && bytes[close_index + 2] == b'}';
                        if next_is_close {
                            index += 1;
                            continue;
                        }

                        let name = &template[index + 2..close_index];
                        if !is_valid_variable_name(name) {
                            return Err(PromptCacheError::TemplateRenderError(format!(
                                "invalid template variable '{}'",
                                name
                            )));
                        }

                        placeholders.push(Placeholder {
                            start: index,
                            end: close_index + 2,
                            name: name.to_string(),
                        });
                        index = close_index + 2;
                        continue;
                    }
                    None => {
                        return Err(PromptCacheError::TemplateRenderError(
                            "unclosed template variable".to_string(),
                        ));
                    }
                }
            }
        }

        if let Some(ch) = template[index..].chars().next() {
            index += ch.len_utf8();
        } else {
            break;
        }
    }

    Ok(placeholders)
}

fn find_placeholder_close(bytes: &[u8], start: usize) -> Option<usize> {
    let mut index = start;
    while index + 1 < bytes.len() {
        if bytes[index] == b'}' && bytes[index + 1] == b'}' {
            return Some(index);
        }
        index += 1;
    }
    None
}
