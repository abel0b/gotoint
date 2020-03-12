enum TokenizerState {
    Space,
    SaveNumber,
    SaveWord,
    IgnoreToken,
}

static MAX_NUM_LEN: usize = 4;
static MAX_WORD_LEN: usize = 32;

pub fn process(content: String) -> String {
    let mut state = TokenizerState::Space;
    let mut tokens = String::new();
    let mut token = String::new();
    let mut first_add = true;

    for (idx, ch) in content.chars().enumerate() {
        match state {
            TokenizerState::Space => {
                if ch.is_digit(10) {
                    token.push(ch);
                    state = TokenizerState::SaveNumber;
                }
                else if ch.is_ascii_alphabetic() {
                    token.push(ch);
                    state = TokenizerState::SaveWord;
                }
                else if ch != ' ' {
                    state = TokenizerState::IgnoreToken;
                }
            },
            TokenizerState::SaveNumber => {
                if ch.is_digit(10) {
                    if token.len() == MAX_NUM_LEN {
                        token.clear();
                        state = TokenizerState::IgnoreToken;
                    }
                    else {
                        token.push(ch);
                    }
                }
                else if ch.is_ascii_alphabetic() {
                    if first_add {
                        first_add = false;
                    }
                    else {
                        tokens.push(' ');
                    }
                    // tokens.push_str(&token);
                    token.clear();
                    token.push(ch);
                    state = TokenizerState::SaveWord;
                }
                else if ch != ' ' {
                    token.clear();
                    state = TokenizerState::IgnoreToken;
                }

                if ch == ' ' || idx + 1 == content.len() {
                    // if first_add {
                    //     first_add = false;
                    // }
                    // else {
                    //     tokens.push(' ');
                    // }
                    // tokens.push_str(&token);
                    token.clear();
                    state = TokenizerState::Space;
                }
            },
            TokenizerState::SaveWord => {
                if ch.is_ascii_alphabetic() {
                    if token.len() == MAX_WORD_LEN {
                        token.clear();
                        state = TokenizerState::IgnoreToken;
                    }
                    else {
                        token.push(ch);
                    }
                }
                else if ch.is_digit(10) {
                    if first_add {
                        first_add = false;
                    }
                    else {
                        tokens.push(' ');
                    }
                    tokens.push_str(&token);
                    token.clear();
                    token.push(ch);
                    state = TokenizerState::SaveNumber;
                }
                
                if ch == ' ' || idx + 1 == content.len() {
                    if first_add {
                        first_add = false;
                    }
                    else {
                        tokens.push(' ');
                    }
                    tokens.push_str(&token);
                    token.clear();
                    state = TokenizerState::Space;
                }
            },
            TokenizerState::IgnoreToken => {
                if ch == ' ' {
                    state = TokenizerState::Space;
                }
            },
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer() {
        assert_eq!(
            process("\r\n\n\n Lorem".to_string()),
            "Lorem".to_string(),
        );
        assert_eq!(
            process("\r\n\n\n Lorem ( ~~ # * ip-sum    ".to_string()),
            "Lorem ipsum".to_string(),
        );
        assert_eq!(
            process("\r\n\n\n Lorem ips'um dolo^r sit amet, cons<ect>etur       adipisci\"ng elit,\n\n sed do ei;usmod tempor inci'didunt ut labore et dolore magna aliqua.\n\n   ".to_string()),
            "Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor incididunt ut labore et dolore magna aliqua".to_string(),
        );
    }
}