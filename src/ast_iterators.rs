use clang::*;

pub struct EntitySemanticParents<'a> {
    next: Option<Entity<'a>>,
}

impl<'a> Iterator for EntitySemanticParents<'a> {
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

pub trait IterableEntity {
    fn semantic_parents(&self) -> EntitySemanticParents;
}

impl<'a> IterableEntity for Entity<'a> {
    fn semantic_parents(&self) -> EntitySemanticParents {
        EntitySemanticParents {
            next: Some(self.clone()),
        }
    }
}

pub fn print_ast(e: &Entity) {
    dump_ast(&mut std::io::stdout(), e, 0).unwrap();
}

pub fn dump_ast_to_string(e: &Entity) -> Result<String, ()> {
    use std::io::Cursor;
    let mut cursor = Cursor::new(Vec::<u8>::new());
    dump_ast(&mut cursor, e, 0)?;

    String::from_utf8(cursor.into_inner()).map_err(|_| ())
}

fn dump_ast<W: std::io::Write>(
    out: &mut W,
    e: &Entity,
    indentation_level: usize,
) -> Result<(), ()> {
    writeln!(
        out,
        "{}{:?}: {:?} ({:?}, {:?}, {:?}, {:?})",
        "\t".repeat(indentation_level),
        e.get_name().unwrap_or("".to_string()),
        e.get_kind(),
        e.get_accessibility(),
        if e.is_const_method() {
            Some("const")
        } else {
            None
        },
        e.get_type().and_then(|t| t.get_ref_qualifier()),
        e.get_exception_specification()
    )
    .map_err(|_| ())?;

    for e in e.get_children().into_iter() {
        dump_ast(out, &e, indentation_level + 1)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::ast_iterators::dump_ast_to_string;
    use crate::test_utils::test_tu_from_source;
    use clang::TranslationUnit;

    #[test]
    fn dump_ast_for_simple_class() -> Result<(), ()> {
        test_tu_from_source(
            r#"
            namespace Bar {
                struct Foo {
                    struct Baz;

                    bool foo(double x) const noexcept {
                        return x >= 42.;
                    }

                private:
                    int i = 42;
                };
            }
        "#,
            |tu: &TranslationUnit| -> Result<(), ()> {
                assert_eq!(
                    dump_ast_to_string(&tu.get_entity().get_children().first().unwrap())?.trim(),
                    r#"
"Bar": Namespace (None, None, None, None)
	"Foo": StructDecl (None, None, None, None)
		"Baz": StructDecl (Some(Public), None, None, None)
		"foo": Method (Some(Public), Some("const"), None, Some(BasicNoexcept))
			"x": ParmDecl (None, None, None, None)
			"": CompoundStmt (None, None, None, None)
				"": ReturnStmt (None, None, None, None)
					"": BinaryOperator (None, None, None, None)
						"x": UnexposedExpr (None, None, None, None)
							"x": DeclRefExpr (None, None, None, None)
						"": FloatingLiteral (None, None, None, None)
		"": AccessSpecifier (Some(Private), None, None, None)
		"i": FieldDecl (Some(Private), None, None, None)
			"": IntegerLiteral (None, None, None, None)
						"#
                    .trim()
                );
                Ok(())
            },
        )
    }
}
