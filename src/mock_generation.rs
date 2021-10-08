use clang::*;

pub fn find_class_entity<'tu>(
    tu: &'tu TranslationUnit,
    class_name: &'tu str,
) -> Option<Entity<'tu>> {
    find_class_entity_impl(tu.get_entity().get_children(), class_name)
}

fn find_class_entity_impl<'tu>(
    entities: Vec<Entity<'tu>>,
    class_name: &'tu str,
) -> Option<Entity<'tu>> {
    if entities.is_empty() {
        None
    } else {
        entities
            .iter()
            .cloned()
            .find(|&e| e.get_name() == Some(class_name.to_string()))
            .or_else(|| {
                find_class_entity_impl(
                    entities
                        .into_iter()
                        .map(|e| e.get_children())
                        .flatten()
                        .collect(),
                    class_name,
                )
            })
    }
}

pub fn generate_mock_definition(interface_class: &Entity, mock_class_name: &str) -> String {
    let mock_methods: Vec<_> = interface_class
        .get_children()
        .iter()
        .filter(|e| e.is_pure_virtual_method())
        .map(format_mock_method_definition)
        .collect();

    format!(
        "struct {} : {} {{\n{}\n}};",
        mock_class_name,
        get_qualified_name(interface_class),
        mock_methods.join("\n")
    )
}

fn format_mock_method_definition(method: &Entity) -> String {
    let attributes = [
        get_method_const_qualifier(&method),
        get_method_value_category_qualifier(&method),
        get_method_exception_specification(&method),
        Some("override"),
    ];

    format!(
        "MOCK_METHOD({}, {}, ({}), ({}));",
        protect_commas(method.get_result_type().unwrap().get_display_name()),
        method.get_name().unwrap(),
        get_method_parameter_type_list(&method)
            .into_iter()
            .map(protect_commas)
            .collect::<Vec<String>>()
            .join(", "),
        itertools::Itertools::intersperse(attributes.iter().cloned().flatten(), ", ")
            .collect::<String>()
    )
}

fn get_method_const_qualifier(method: &Entity) -> Option<&'static str> {
    if method.is_const_method() {
        Some("const")
    } else {
        None
    }
}

fn get_method_value_category_qualifier(method: &Entity) -> Option<&'static str> {
    method
        .get_type()
        .unwrap()
        .get_ref_qualifier()
        .map(|q| match q {
            RefQualifier::LValue => "ref(&)",
            RefQualifier::RValue => "ref(&&)",
        })
}

fn get_method_exception_specification(method: &Entity) -> Option<&'static str> {
    method.get_exception_specification().and_then(|e| match e {
        ExceptionSpecification::BasicNoexcept => Some("noexcept"),
        _ => None,
    })
}

fn get_method_parameter_type_list(method: &Entity) -> Vec<String> {
    method
        .get_arguments()
        .unwrap()
        .iter()
        .map(|&e| e.get_type().unwrap().get_display_name())
        .collect::<Vec<_>>()
}

fn protect_commas(mut ty: String) -> String {
    if ty.contains(",") {
        ty.insert(0, '(');
        ty.push(')');
    }

    ty
}

struct EntitySemanticParentIter<'a> {
    next: Option<Entity<'a>>,
}

impl<'a> Iterator for EntitySemanticParentIter<'a> {
    type Item = Entity<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let parent = self.next.and_then(|e| e.get_semantic_parent());
        let current = std::mem::replace(&mut self.next, parent);
        if self.next.is_some() {
            current
        } else {
            None
        }
    }
}

fn get_qualified_name(entity: &Entity) -> String {
    itertools::Itertools::intersperse(
        EntitySemanticParentIter {
            next: Some(entity.clone()),
        }
        .map(|e| e.get_display_name().unwrap())
        .collect::<Vec<_>>()
        .into_iter()
        .rev(),
        "::".to_string(),
    )
    .collect()
}

#[cfg(test)]
mod test_helpers {
    use super::*;
    use itertools::Itertools;
    use std::num::Wrapping;
    use std::path::PathBuf;

    thread_local!(static CLANG: Clang = Clang::new().unwrap());

    pub fn get_temp_cpp_filename() -> PathBuf {
        std::env::temp_dir().with_file_name("interface.cpp")
    }

    pub fn test_tu_from_source<C: Fn(&TranslationUnit)>(code: &str, callback: C) {
        CLANG.with(|clang| {
            let index = Index::new(clang, false, true);
            let file = Unsaved::new(get_temp_cpp_filename(), code);

            let tu = index
                .parser(get_temp_cpp_filename())
                .unsaved(&[file])
                .arguments(&["--std=c++17"])
                .parse()
                .unwrap();

            callback(&tu);
        });
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
}

#[cfg(test)]
mod class_entity_location_tests {
    use super::test_helpers::*;
    use super::*;

    #[test]
    fn class_entity_is_found_in_file() {
        test_tu_from_source(
            r#"
                struct Foo {
                    int foo();
                };
            "#,
            |tu| {
                let class = find_class_entity(tu, "Foo").expect("Class entity was not found!");
                let loc = class.get_location().unwrap().get_file_location();
                assert_eq!(loc.file.unwrap().get_path(), get_temp_cpp_filename());
                assert_eq!(loc.line, 2);
                assert_eq!(loc.column, 24);
            },
        );
    }

    #[test]
    fn class_entity_is_found_in_namespace() {
        test_tu_from_source(
            r#"
                template <typename T, typename U> struct map;
                struct string;
                
                namespace Bar {
                    struct Foo {
                        int foo();
                        virtual auto getPhoneBook() const -> map<string, unsigned int> = 0; 
                    };
                }
            "#,
            |tu| {
                let class = find_class_entity(tu, "Foo").expect("Class entity was not found!");
                let loc = class.get_location().unwrap().get_file_location();
                assert_eq!(loc.file.unwrap().get_path(), get_temp_cpp_filename());
                assert_eq!(loc.line, 6);
                assert_eq!(loc.column, 28);
            },
        );
    }

    #[test]
    fn class_entity_is_found_for_nested_class() {
        test_tu_from_source(
            r#"
                struct Bar {
                    struct Foo {
                        int foo();
                    };
                };
            "#,
            |tu| {
                let class = find_class_entity(tu, "Foo").expect("Class entity was not found!");
                let loc = class.get_location().unwrap().get_file_location();
                assert_eq!(loc.file.unwrap().get_path(), get_temp_cpp_filename());
                assert_eq!(loc.line, 3);
                assert_eq!(loc.column, 28);
            },
        );
    }

    #[test]
    fn class_entity_is_not_found() {
        test_tu_from_source(
            r#"
                struct Foo {
                    int foo();
                };
            "#,
            |tu| {
                assert!(find_class_entity(tu, "Bar").is_none());
            },
        );
    }
}

#[cfg(test)]
mod mock_method_tests {
    use super::*;

    fn test_method_for_function<C: Fn(&Entity)>(func_decl: &str, callback: C) {
        super::test_helpers::test_tu_from_source(
            &format!(
                r#"
                    template <typename T, typename U> struct pair;
                    struct TestClass {{ {} }};
                "#,
                func_decl
            ),
            |tu| {
                callback(
                    find_class_entity(tu, "TestClass")
                        .unwrap()
                        .get_children()
                        .first()
                        .unwrap(),
                )
            },
        )
    }

    fn assert_mock_for_function(func_decl: &str, mock_decl: &str) {
        test_method_for_function(func_decl, |method| {
            assert_eq!(format_mock_method_definition(method), mock_decl)
        });
    }

    #[test]
    fn mock_definition_for_simple_function() {
        assert_mock_for_function(
            "virtual void foo() = 0;",
            "MOCK_METHOD(void, foo, (), (override));",
        );
    }

    #[test]
    fn mock_definition_for_function_with_return_type() {
        assert_mock_for_function(
            "virtual int foo() = 0;",
            "MOCK_METHOD(int, foo, (), (override));",
        );
    }

    #[test]
    fn mock_definition_for_function_with_trailing_return_type() {
        assert_mock_for_function(
            "virtual auto foo() -> int = 0;",
            "MOCK_METHOD(int, foo, (), (override));",
        );
    }

    #[test]
    fn mock_definition_for_function_with_unprotected_commas_in_return_type() {
        assert_mock_for_function(
            "virtual auto foo() -> pair<int, double> = 0;",
            "MOCK_METHOD((pair<int, double>), foo, (), (override));",
        );
    }

    #[test]
    fn mock_definition_for_function_with_single_parameter() {
        assert_mock_for_function(
            "virtual void foo(int x) = 0;",
            "MOCK_METHOD(void, foo, (int), (override));",
        );
    }

    #[test]
    fn mock_definition_for_function_with_multiple_parameters() {
        assert_mock_for_function(
            "virtual void foo(int const& x, double const*) = 0;",
            "MOCK_METHOD(void, foo, (const int &, const double *), (override));",
        );
    }

    #[test]
    fn mock_definition_for_function_with_unprotected_commas_in_parameters() {
        assert_mock_for_function(
            "virtual void foo(int, pair<int, double> y) = 0;",
            "MOCK_METHOD(void, foo, (int, (pair<int, double>)), (override));",
        );
    }

    #[test]
    fn mock_definition_for_noexcept_function() {
        assert_mock_for_function(
            "virtual void foo() noexcept = 0;",
            "MOCK_METHOD(void, foo, (), (noexcept, override));",
        );
    }

    #[test]
    fn mock_definition_for_const_qualified_function() {
        assert_mock_for_function(
            "virtual void foo() const = 0;",
            "MOCK_METHOD(void, foo, (), (const, override));",
        );
    }

    #[test]
    fn mock_definition_for_lvalue_ref_qualified_function() {
        assert_mock_for_function(
            "virtual void foo() & = 0;",
            "MOCK_METHOD(void, foo, (), (ref(&), override));",
        );
    }

    #[test]
    fn mock_definition_for_rvalue_ref_qualified_function() {
        assert_mock_for_function(
            "virtual void foo() && = 0;",
            "MOCK_METHOD(void, foo, (), (ref(&&), override));",
        );
    }

    #[test]
    fn mock_definition_for_lvalue_const_ref_qualified_function() {
        assert_mock_for_function(
            "virtual void foo() const& = 0;",
            "MOCK_METHOD(void, foo, (), (const, ref(&), override));",
        );
    }

    #[test]
    fn mock_definition_for_maximally_qualified_function() {
        assert_mock_for_function(
            "virtual void foo() const&& noexcept = 0;",
            "MOCK_METHOD(void, foo, (), (const, ref(&&), noexcept, override));",
        );
    }
}

#[cfg(test)]
mod generate_mock_class_from_interface {
    use super::test_helpers::*;
    use super::*;

    #[test]
    fn mock_class_inherits_from_class() {
        test_class_from_source("struct Foo;", "Foo", |class| {
            assert_eq_upto_whitespace(
                &generate_mock_definition(&class, "FooMock"),
                "struct FooMock : Foo {};",
            )
        });
    }

    #[test]
    fn mock_class_inherits_from_class_in_namespace() {
        test_class_from_source("namespace Bar { struct Foo; }", "Foo", |class| {
            assert_eq_upto_whitespace(
                &generate_mock_definition(&class, "FooMock"),
                "struct FooMock : Bar::Foo {};",
            )
        })
    }

    #[test]
    fn mock_class_inherits_from_nested_class() {
        test_class_from_source("struct Bar { struct Foo; };", "Foo", |class| {
            assert_eq_upto_whitespace(
                &generate_mock_definition(&class, "FooMock"),
                "struct FooMock : Bar::Foo {};",
            )
        })
    }

    #[test]
    fn only_pure_virtual_functions_are_mocked() {
        test_class_from_source(
            r#"
                struct Foo {
                    void foo();
                    virtual void bar();
                    virtual void baz() = 0;
                };
            "#,
            "Foo",
            |class| {
                assert_eq_upto_whitespace(
                    &generate_mock_definition(&class, "FooMock"),
                    r#"
                        struct FooMock : Foo {
                            MOCK_METHOD(void, baz, (), (override));
                        };
                    "#,
                )
            },
        )
    }

    #[test]
    fn multiple_pure_virtual_functions_are_mocked() {
        test_class_from_source(
            r#"
                struct Foo {
                    virtual void foo() = 0;
                    virtual void bar() const = 0;
                    virtual void baz() noexcept = 0;
                };
            "#,
            "Foo",
            |class| {
                assert_eq_upto_whitespace(
                    &generate_mock_definition(&class, "FooMock"),
                    r#"
                        struct FooMock : Foo {
                            MOCK_METHOD(void, foo, (), (override));
                            MOCK_METHOD(void, bar, (), (const, override));
                            MOCK_METHOD(void, baz, (), (noexcept, override));
                        };
                    "#,
                )
            },
        )
    }
}
