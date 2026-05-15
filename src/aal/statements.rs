use anyhow::Result;

use crate::aal::draft::Draft;
use crate::aal::values::{join, parse_bool, parse_u16, parse_u32, parse_u64};
use crate::spec::RouteCheckSpec;

pub(crate) fn verify(draft: &mut Draft, line_number: usize, tokens: &[String]) -> Result<bool> {
    match tokens.first().map(String::as_str) {
        Some("profile") if tokens.len() == 2 => draft.verify_profile = Some(tokens[1].clone()),
        Some("command") if tokens.len() >= 2 => draft.verify_commands.push(join(&tokens[1..])),
        Some("runtime_start") if tokens.len() >= 2 => {
            draft.runtime.start_command = Some(join(&tokens[1..]))
        }
        Some("runtime_base_url") if tokens.len() == 2 => {
            draft.runtime.base_url = Some(tokens[1].clone())
        }
        Some("runtime_timeout_secs") if tokens.len() == 2 => {
            draft.runtime.timeout_secs = Some(parse_u64(line_number, &tokens[1])?)
        }
        Some("runtime_smoke") => return runtime_route(draft, line_number, tokens),
        Some(_) => draft.verify_commands.push(join(tokens)),
        None => {}
    }
    Ok(true)
}

pub(crate) fn transaction(
    draft: &mut Draft,
    line_number: usize,
    tokens: &[String],
) -> Result<bool> {
    match tokens.first().map(String::as_str) {
        Some("isolation") if tokens.len() == 2 => {}
        Some("max_repair_attempts") if tokens.len() == 2 => {
            draft.transaction.max_repair_attempts = parse_u32(line_number, &tokens[1])?
        }
        Some("approval_required") if tokens.len() == 2 => {
            draft.transaction.approval_required = parse_bool(line_number, &tokens[1])?
        }
        Some("on_failure") if tokens.len() == 2 => {
            draft.transaction.rollback_on_failure = tokens[1] == "rollback"
        }
        Some("on_success") => on_success(draft, &tokens[1..]),
        Some(_) => return Ok(false),
        None => {}
    }
    Ok(true)
}

fn runtime_route(draft: &mut Draft, line_number: usize, tokens: &[String]) -> Result<bool> {
    if tokens.len() == 5 && tokens[1] == "route" && tokens[3] == "expect" {
        draft.routes.push(RouteCheckSpec {
            path: tokens[2].clone(),
            expect: parse_u16(line_number, &tokens[4])?,
        });
        return Ok(true);
    }
    Ok(false)
}

fn on_success(draft: &mut Draft, tokens: &[String]) {
    draft.transaction.commit_on_success = tokens.iter().any(|token| token == "commit_code");
    draft.transaction.memory_promotion = if tokens.iter().any(|token| token == "promote_memory") {
        "on_success".to_string()
    } else {
        "never".to_string()
    };
}
