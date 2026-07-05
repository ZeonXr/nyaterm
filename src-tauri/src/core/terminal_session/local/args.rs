pub(crate) fn parse_shell_args(input: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut chars = input.trim().chars().peekable();
    let mut quote: Option<char> = None;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        if ch == '\\' {
            let escapes_next = chars.peek().is_some_and(|next| match quote {
                Some(active_quote) => *next == active_quote || *next == '\\',
                None => next.is_whitespace() || *next == '"' || *next == '\'' || *next == '\\',
            });
            if escapes_next {
                escaped = true;
            } else {
                current.push(ch);
            }
            continue;
        }

        if let Some(active_quote) = quote {
            if ch == active_quote {
                quote = None;
            } else {
                current.push(ch);
            }
            continue;
        }

        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            continue;
        }

        if ch.is_whitespace() {
            if !current.is_empty() {
                args.push(std::mem::take(&mut current));
            }
            while chars.peek().is_some_and(|next| next.is_whitespace()) {
                let _ = chars.next();
            }
            continue;
        }

        current.push(ch);
    }

    if escaped {
        current.push('\\');
    }
    if quote.is_some() {
        return Err("Unclosed quote in shell arguments".to_string());
    }
    if !current.is_empty() {
        args.push(current);
    }

    Ok(args)
}
