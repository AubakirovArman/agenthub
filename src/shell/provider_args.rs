use anyhow::{anyhow, Result};

pub(super) struct OpenaiHttpAdd<'a> {
    pub provider: &'a str,
    pub name: &'a str,
    pub url: &'a str,
    pub model: Option<&'a str>,
    pub api_key_env: Option<&'a str>,
}

pub(super) fn parse_add_openai_http<'a>(args: &'a [&'a str]) -> Result<OpenaiHttpAdd<'a>> {
    let provider = required(args, 1, "provider")?;
    if args.iter().any(|value| value.starts_with("--")) {
        return Ok(OpenaiHttpAdd {
            provider,
            name: flag_value(args, "--name")?,
            url: flag_value(args, "--url")?,
            model: optional_flag_value(args, "--model"),
            api_key_env: optional_flag_value(args, "--api-key-env"),
        });
    }
    Ok(OpenaiHttpAdd {
        provider,
        name: required(args, 2, "name")?,
        url: required(args, 3, "url")?,
        model: args.get(4).copied(),
        api_key_env: args.get(5).copied(),
    })
}

fn required<'a>(args: &'a [&str], index: usize, name: &str) -> Result<&'a str> {
    args.get(index)
        .copied()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("missing {name}"))
}

fn flag_value<'a>(args: &'a [&str], flag: &str) -> Result<&'a str> {
    optional_flag_value(args, flag).ok_or_else(|| anyhow!("missing {flag}"))
}

fn optional_flag_value<'a>(args: &'a [&str], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|pair| pair[0] == flag && !pair[1].starts_with("--"))
        .map(|pair| pair[1])
}

#[cfg(test)]
mod tests {
    use super::parse_add_openai_http;

    #[test]
    fn parses_openai_http_add_flags() {
        let args = [
            "add",
            "openai-http",
            "--name",
            "local-vllm",
            "--url",
            "http://127.0.0.1:8000",
            "--model",
            "qwen3",
        ];

        let parsed = parse_add_openai_http(&args).unwrap();

        assert_eq!(parsed.name, "local-vllm");
        assert_eq!(parsed.url, "http://127.0.0.1:8000");
        assert_eq!(parsed.model, Some("qwen3"));
    }

    #[test]
    fn parses_openai_http_add_shorthand() {
        let args = ["add", "openai-http", "kimi-api", "https://x.test/v1"];

        let parsed = parse_add_openai_http(&args).unwrap();

        assert_eq!(parsed.name, "kimi-api");
        assert_eq!(parsed.url, "https://x.test/v1");
        assert_eq!(parsed.model, None);
    }
}
