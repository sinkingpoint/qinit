use std::collections::HashMap;

/// do_string_replacement is responsible for doing argument substitution in strings
/// it's very simple at the moment - for every k=v in the hashmap, replace ${k} with v in the given
/// string, and return the result
pub fn do_string_replacement(args: &Option<&HashMap<String, String>>, s: &str) -> String {
    let mut build = s.clone().to_owned();
    if let Some(args) = args {
        for (k, v) in args.iter() {
            build = build.replace(&format!("${{{}}}", k), v);
        }    
    }

    return build;
}