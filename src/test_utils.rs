use super::*;
use itertools::Itertools;
use std::num::Wrapping;
use std::path::PathBuf;

thread_local!(static CLANG: Clang = Clang::new().unwrap());

pub fn get_temp_cpp_filename() -> PathBuf {
    std::env::temp_dir().with_file_name("interface.cpp")
}

pub fn test_tu_from_source<R, C: Fn(&TranslationUnit) -> R>(code: &str, callback: C) -> R {
    CLANG.with(|clang| {
        let index = Index::new(clang, false, true);
        let file = Unsaved::new(get_temp_cpp_filename(), code);

        let tu = index
            .parser(get_temp_cpp_filename())
            .unsaved(&[file])
            .arguments(&["--std=c++17"])
            .parse()
            .unwrap();

        callback(&tu)
    })
}

pub fn test_class_from_source<C: Fn(Entity)>(code: &str, class_name: &str, callback: C) {
    test_tu_from_source(code, |tu| {
        callback(find_class_entity(tu, class_name).unwrap());
    })
}

fn split_around<P: FnMut(char) -> bool>(input: &str, pat: P) -> Vec<&str> {
    use std::iter::once;
    once(Wrapping(usize::MAX))
        .chain(input.match_indices(pat).map(|t| Wrapping(t.0)))
        .chain(once(Wrapping(input.len())))
        .tuple_windows()
        .map(|(a, b)| [((a + Wrapping(1)).0, b.0), (b.0, b.0 + 1)])
        .flatten()
        .filter(|(a, b)| b - a > 0)
        .filter(|(a, _)| *a < input.len())
        .map(|(a, b)| &input[a..b])
        .collect()
}

pub fn assert_eq_upto_whitespace(left: &str, right: &str) {
    fn tokens(input: &str) -> Vec<&str> {
        input
            .split_ascii_whitespace()
            .map(|token| split_around(token, |c| "{[()]},.;:+-*/&|^!%#@=".contains(c)))
            .flatten()
            .collect()
    }

    // println!("left:  {:?}\nright: {:?}", tokens(left), tokens(right));

    if tokens(left) != tokens(right) {
        assert_eq!(left, right);
    }
}
