#[cfg(test)]
mod test_strings {
    fn _is_same(a: Vec<&str>, b: Vec<String>) -> bool {
        println!("a: {:?}\nb: {:?}", a, b);
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
        let tokenizer = libq::strings::Tokenizer::new("cats and dogs", vec!['\n', ';', '|', '&', '#']);
        assert!(_is_same(vec!["cats", "and", "dogs"], tokenizer.collect()));
    }

    #[test]
    fn test_tokenizer_handles_single_quotes() {
        let tokenizer = libq::strings::Tokenizer::new("cats 'and dogs'", vec!['\n', ';', '|', '&', '#']);
        assert!(_is_same(vec!["cats", "'and dogs'"], tokenizer.collect()));
    }

    #[test]
    fn test_tokenizer_handles_double_quotes() {
        let tokenizer = libq::strings::Tokenizer::new("cats \"and dogs\"", vec!['\n', ';', '|', '&', '#']);
        assert!(_is_same(vec!["cats", "\"and dogs\""], tokenizer.collect()));
    }

    #[test]
    fn test_tokenizer_handles_nested_quotes() {
        let tokenizer = libq::strings::Tokenizer::new("cats \"'and dogs\"", vec!['\n', ';', '|', '&', '#']);
        assert!(_is_same(vec!["cats", "\"'and dogs\""], tokenizer.collect()));
    }

    #[test]
    fn test_tokenizer_handles_comments() {
        let tokenizer = libq::strings::Tokenizer::new("cats #\"'and dogs\"", vec!['\n', ';', '|', '&', '#']);
        assert!(_is_same(vec!["cats", "#", "\"'and dogs\""], tokenizer.collect()));
        let tokenizer = libq::strings::Tokenizer::new("cats #\"'and dogs\"\nand pigs", vec!['\n', ';', '|', '&', '#']);
        assert!(_is_same(vec!["cats", "#", "\"'and dogs\"", "\n", "and", "pigs"], tokenizer.collect()));
    }

    #[test]
    fn test_tokenizer_handles_non_whitespace_hashes() {
        // tests tokenizer doesn't treat "cats#ca" as a comment
        let tokenizer = libq::strings::Tokenizer::new("cats#ca", vec!['\n', ';', '|', '&', '#']);
        assert!(_is_same(vec!["cats#ca"], tokenizer.collect()));
    }

    #[test]
    fn test_tokenizer_handles_double_semicolon() {
        let tokenizer = libq::strings::Tokenizer::new("cats;;", vec!['\n', ';', '|', '&', '#']);
        assert!(_is_same(vec!["cats", ";;"], tokenizer.collect()));

        let tokenizer = libq::strings::Tokenizer::new("cats ;;", vec!['\n', ';', '|', '&', '#']);
        assert!(_is_same(vec!["cats", ";;"], tokenizer.collect()));
    }

    #[test]
    fn test_tokenizer_handles_single_semicolon() {
        let tokenizer = libq::strings::Tokenizer::new("cats;dogs;", vec!['\n', ';', '|', '&', '#']);
        assert!(_is_same(vec!["cats", ";", "dogs", ";"], tokenizer.collect()));
    }

    #[test]
    fn test_tokenizer_handles_new_lines() {
        let tokenizer = libq::strings::Tokenizer::new("cats\n\ndogs\n\n\n", vec!['\n', ';', '|', '&', '#']);
        assert!(_is_same(vec!["cats", "\n", "\n", "dogs", "\n", "\n", "\n"], tokenizer.collect()));
    }

    #[test]
    fn test_tokenizer_splits_on_specials() {
        let tokenizer = libq::strings::Tokenizer::new("cats|dogs", vec!['\n', ';', '|', '&', '#']);
        assert!(_is_same(vec!["cats", "|", "dogs"], tokenizer.collect()));

        let tokenizer = libq::strings::Tokenizer::new("cats& dogs", vec!['\n', ';', '|', '&', '#']);
        assert!(_is_same(vec!["cats", "&", "dogs"], tokenizer.collect()));

        let tokenizer = libq::strings::Tokenizer::new("cats && dogs", vec!['\n', ';', '|', '&', '#']);
        assert!(_is_same(vec!["cats", "&&", "dogs"], tokenizer.collect()));
    }

    #[test]
    fn test_tokenizer_on_full_programs() {
        let program = "cat /proc/cmdline | read cmdline
local root=
local root_type=auto
local options=

echo \"Parsing Commandline Flags\"
for word in $cmdline; do
    case $word in
        root=*)
            echo \"Found root device line: $word\"
            root=${word#root=} 
        ;;
        ro) local options=\"$options,ro\" ;;
    esac
done

echo \"Mounting $root with options $options and type $root_type\"";
        let tokenizer = libq::strings::Tokenizer::new(program, vec!['\n', ';', '|', '&', '#']);
        assert!(_is_same(vec!["cat", "/proc/cmdline", "|", "read", "cmdline", "\n", "local", "root=", "\n", "local", "root_type=auto", "\n", "local", "options=",
                              "\n", "\n", "echo", "\"Parsing Commandline Flags\"", "\n", "for", "word", "in", "$cmdline", ";", "do", "\n",
                              "case", "$word", "in", "\n", "root=*)", "\n", "echo", "\"Found root device line: $word\"", "\n", "root=${word#root=}", "\n", ";;", "\n",
                              "ro)", "local", "options=\"$options,ro\"", ";;", "\n", "esac", "\n", "done", "\n", "\n", "echo", "\"Mounting $root with options $options and type $root_type\""],
                        tokenizer.collect()));
    }
}