use anyhow::{anyhow, Result};

use crate::aal::builder::build_spec;
use crate::aal::diagnostics::AalDiagnostic;
use crate::aal::draft::Draft;
use crate::aal::formatter;
use crate::aal::lexer::tokenize;
use crate::aal::preamble;
use crate::aal::section::{parse_section, Section};
use crate::aal::semantics;
use crate::aal::statements;
use crate::aal::values::join;
use crate::aal::AalParseOutput;

pub fn parse_aal(source: &str) -> Result<AalParseOutput> {
    let mut parser = AalParser::default();
    for (index, raw) in source.lines().enumerate() {
        parser.line(index + 1, raw)?;
    }
    parser.finish()
}

pub fn format_aal(source: &str) -> Result<String> {
    Ok(parse_aal(source)?.normalized)
}

#[derive(Default)]
struct AalParser {
    draft: Draft,
    diagnostics: Vec<AalDiagnostic>,
    section: Section,
    open: bool,
    closed: bool,
}

impl AalParser {
    fn line(&mut self, line_number: usize, raw: &str) -> Result<()> {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            return Ok(());
        }
        if !self.open {
            return self.header(line_number, line);
        }
        if line == "}" {
            self.closed = true;
            self.open = false;
            return Ok(());
        }
        if let Some(section) = parse_section(line) {
            self.section = section;
            return Ok(());
        }
        let item = line.strip_prefix("- ").unwrap_or(line);
        let tokens = tokenize(item, line_number)?;
        if line.starts_with("- ") {
            return self.section_item(line_number, &tokens);
        }
        self.statement(line_number, &tokens)
    }

    fn header(&mut self, line_number: usize, line: &str) -> Result<()> {
        let tokens = tokenize(line, line_number)?;
        if preamble::handle(&mut self.draft, line_number, &tokens)? {
            return Ok(());
        }
        if tokens.len() == 3 && matches!(tokens[0].as_str(), "change" | "task") && tokens[2] == "{"
        {
            self.draft.name = Some(tokens[1].clone());
            self.open = true;
            return Ok(());
        }
        Err(anyhow!(
            "{}",
            AalDiagnostic::error(line_number, "expected `change Name {`").render()
        ))
    }

    fn statement(&mut self, line_number: usize, tokens: &[String]) -> Result<()> {
        if self.section == Section::Transaction {
            if !statements::transaction(&mut self.draft, line_number, tokens)? {
                return self.unknown(line_number, tokens.first().map_or("", String::as_str));
            }
            return Ok(());
        }
        match tokens.first().map(String::as_str) {
            Some("workspace") if tokens.len() == 2 => {
                self.draft.workspace = Some(tokens[1].clone())
            }
            Some("goal") if tokens.len() >= 2 => self.draft.goal = Some(tokens[1..].join(" ")),
            Some("topology") if tokens.len() == 2 => self.draft.topology = Some(tokens[1].clone()),
            Some("use")
                if tokens.get(1).map(String::as_str) == Some("skill") && tokens.len() == 3 =>
            {
                self.draft.skills.push(tokens[2].clone());
                self.draft.skill_lines.push(line_number);
            }
            Some(other) => return self.unknown(line_number, other),
            None => {}
        }
        Ok(())
    }

    fn section_item(&mut self, line_number: usize, tokens: &[String]) -> Result<()> {
        match self.section {
            Section::Allow => {
                self.draft.allow.push(join(tokens));
                self.draft.allow_lines.push(line_number);
            }
            Section::Deny => {
                self.draft.deny.push(join(tokens));
                self.draft.deny_lines.push(line_number);
            }
            Section::Rules => self.draft.rules.push(join(tokens)),
            Section::Execute => self.draft.execution_commands.push(join(tokens)),
            Section::Verify => {
                if !statements::verify(&mut self.draft, line_number, tokens)? {
                    return self.unknown(line_number, tokens.first().map_or("", String::as_str));
                }
            }
            Section::Transaction => {
                if !statements::transaction(&mut self.draft, line_number, tokens)? {
                    return self.unknown(line_number, tokens.first().map_or("", String::as_str));
                }
            }
            Section::Body => {
                return self.unknown(line_number, tokens.first().map_or("-", String::as_str))
            }
        }
        Ok(())
    }

    fn finish(mut self) -> Result<AalParseOutput> {
        if !self.closed {
            return Err(anyhow!(
                "{}",
                AalDiagnostic::error(0, "missing closing `}`").render()
            ));
        }
        self.diagnostics.extend(semantics::validate(&self.draft));
        let normalized = formatter::format(&self.draft);
        let spec = build_spec(&self.draft);
        spec.validate()?;
        Ok(AalParseOutput {
            spec,
            diagnostics: self.diagnostics,
            normalized,
        })
    }

    fn unknown<T>(&self, line_number: usize, item: &str) -> Result<T> {
        Err(anyhow!(
            "{}",
            AalDiagnostic::error(line_number, format!("unsupported AAL statement `{item}`"))
                .render()
        ))
    }
}
