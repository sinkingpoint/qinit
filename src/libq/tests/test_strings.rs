#[cfg(test)]
mod test_strings {
    fn _is_same(a: Vec<&str>, b: Vec<String>) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let aiter = a.iter();
        let mut biter = b.iter();
        for a_item in aiter {
            let b_item = biter.next().unwrap();
            if a_item != b_item {
                println!("{} != {}", a_item, b_item);
                return false;
            }
        }

        return true;
    }

    #[test]
    fn test_tokenizer_splits_on_whitespace() {
        let tokenizer = libq::strings::Tokenizer::new("cats and dogs");
        assert!(_is_same(vec!["cats", "and", "dogs"], tokenizer.collect()));
    }

    #[test]
    fn test_tokenizer_handles_single_quotes() {
        let tokenizer = libq::strings::Tokenizer::new("cats 'and dogs'");
        assert!(_is_same(vec!["cats", "and dogs"], tokenizer.collect()));
    }

    #[test]
    fn test_tokenizer_handles_double_quotes() {
        let tokenizer = libq::strings::Tokenizer::new("cats \"and dogs\"");
        assert!(_is_same(vec!["cats", "and dogs"], tokenizer.collect()));
    }

    #[test]
    fn test_tokenizer_handles_nested_quotes() {
        let tokenizer = libq::strings::Tokenizer::new("cats \"'and dogs\"");
        assert!(_is_same(vec!["cats", "'and dogs"], tokenizer.collect()));
    }

    #[test]
    fn test_tokenizer_handles_escapes() {
        let tokenizer = libq::strings::Tokenizer::new("cats \"\\\"and dogs'\" \\n\\t\\\\");
        assert!(_is_same(vec!["cats", "\"and dogs\'", "\n\t\\"], tokenizer.collect()));
    }
}