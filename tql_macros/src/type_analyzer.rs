//! Expression type analyzer.

extern crate rustc_front;

use rustc::lint::{EarlyContext, EarlyLintPass, LateContext, LateLintPass, LintArray, LintContext, LintPass};
use rustc::middle::ty::{Ty, TypeAndMut, TyS, TypeVariants};
use self::rustc_front::hir::Expr;
use self::rustc_front::hir::Expr_::{self, ExprAddrOf, ExprMethodCall, ExprVec};
use syntax::ast::Attribute;
use syntax::ast::IntTy::{TyI32, TyI64};
use syntax::codemap::{NO_EXPANSION, BytePos, Span};

use analyzer::unknown_table_error;
use error::{Error, ErrorType, SqlResult, res};
use state::{SqlFields, SqlTables, Type, lint_singleton, singleton};
use string::find_near;

declare_lint!(SQL_LINT, Forbid, "Err about SQL type errors");
declare_lint!(SQL_ATTR_LINT, Forbid, "Err about SQL table errors");

pub struct SqlError;
pub struct SqlAttrError;

impl LintPass for SqlError {
    fn get_lints(&self) -> LintArray {
        lint_array!(SQL_LINT)
    }
}

impl LintPass for SqlAttrError {
    fn get_lints(&self) -> LintArray {
        lint_array!(SQL_ATTR_LINT)
    }
}

/// Analyze the types of the SQL table struct.
fn analyze_table_types(fields: &SqlFields, sql_tables: &SqlTables) -> SqlResult<()> {
    let mut errors = vec![];
    let mut primary_key_count = 0;
    for field in fields.values() {
        match field.node {
            Type::Custom(ref related_table_name) =>
                if let None = sql_tables.get(related_table_name) {
                    unknown_table_error(related_table_name, field.span, sql_tables, &mut errors);
                },
            Type::UnsupportedType(ref typ) => {
                errors.push(Error::new_with_code(format!("Use of unsupported type name `{}`", typ), field.span, "E0412"));
            },
            Type::Serial => primary_key_count += 1,
            _ => (),
        }
    }
    //match primary_key_count {
        //0 => errors.push(Error.new_warning("No primary key found", )), // TODO
    //}
    res((), errors)
}

/// Get the types of the elements in a `Vec`.
fn argument_types<'a>(cx: &'a LateContext, arguments: &'a Expr_) -> Vec<Ty<'a>> {
    let mut types = vec![];
    if let ExprAddrOf(_, ref argument) = *arguments {
        if let ExprVec(ref vector) = argument.node {
            for element in vector {
                if let ExprAddrOf(_, ref field) = element.node {
                    types.push(cx.tcx.node_id_to_type(field.id));
                }
            }
        }
    }
    types
}

impl EarlyLintPass for SqlAttrError {
    fn exit_lint_attrs(&mut self, cx: &EarlyContext, _: &[Attribute]) {
        static mut analyze_done: bool = false;
        let done = unsafe { analyze_done };
        if !done {
            let sql_tables = singleton();
            for fields in sql_tables.values() {
                if let Err(errors) = analyze_table_types(&fields, &sql_tables) {
                    span_errors(errors, cx);
                }
            }
        }
        unsafe {
            analyze_done = true;
        }
    }
}

impl LateLintPass for SqlError {
    /// Check the types of the `Vec` argument of the `postgres::stmt::Statement::query` method.
    fn check_expr(&mut self, cx: &LateContext, expr: &Expr) {
        let tables = singleton();
        if let ExprMethodCall(name, _, ref arguments) = expr.node {
            let method_name = name.node.to_string();
            if method_name == "query" || method_name == "execute" {
                let types = argument_types(cx, &arguments[1].node);
                let calls = lint_singleton();
                let BytePos(low) = expr.span.lo;
                match calls.get(&low) {
                    Some(fields) => {
                        if let Some(table) = tables.get(&fields.table_name) {
                            for (i, typ) in types.iter().enumerate() {
                                let field = &fields.arguments[i];
                                let position = Span {
                                    lo: BytePos(field.low),
                                    hi: BytePos(field.high),
                                    expn_id: NO_EXPANSION,
                                };
                                if field.name == "i64" {
                                    check_type(&Type::I64, typ, position, expr.span, cx);
                                }
                                else if let Some(field_type) = table.get(&field.name) {
                                    check_type(&field_type.node, typ, position, expr.span, cx);
                                }
                                else {
                                    cx.sess().span_err(position, &format!("attempted access of field `{}` on type `{}`, but no field with that name was found", field.name, fields.table_name));
                                    let field_names = fields.arguments.iter().map(|arg| {
                                        &arg.name
                                    });
                                    match find_near(&field.name, field_names) {
                                        Some(name) => {
                                            cx.sess().span_help(position, &format!("did you mean `{}`?", name));
                                        },
                                        None => (),
                                    }
                                }
                            }
                        }
                    },
                    None => (), // TODO
                }
            }
        }
    }
}

/// Check that the `field_type` is the same as the `expected_type`.
/// If not, show an error message.
fn check_type(field_type: &Type, expected_type: &TyS, position: Span, note_position: Span, cx: &LateContext) {
    if !same_type(field_type, expected_type) {
        cx.sess().span_err_with_code(position, &format!("mismatched types:\n expected `{}`,    found `{:?}`", field_type, expected_type), "E0308");
        cx.sess().fileline_note(note_position, "in this expansion of sql! (defined in tql)");
    }
}

/// Comapre the `field_type` with the `expected_type`.
fn same_type(field_type: &Type, expected_type: &TyS) -> bool {
    match expected_type.sty {
        TypeVariants::TyInt(TyI32) => {
            *field_type == Type::I32 || *field_type == Type::Serial
        },
        TypeVariants::TyInt(TyI64) => {
            *field_type == Type::I64
        },
        TypeVariants::TyRef(_, TypeAndMut { ty, .. }) => {
            // TODO: supporter les références de références.
            match ty.sty {
                TypeVariants::TyStr => {
                    *field_type == Type::String
                },
                _ => false,
            }
        },
        TypeVariants::TyStruct(_, _) => {
            // TODO: supporter la comparaison d’une clé étrangère.
            false
        },
        _ => false,
    }
}

/// Show the compilation errors.
fn span_errors(errors: Vec<Error>, cx: &EarlyContext) {
    for &Error {ref code, ref message, position, ref kind} in &errors {
        match *kind {
            ErrorType::Error => {
                match *code {
                    Some(ref code) => cx.sess().span_err_with_code(position, &message, code),
                    None => cx.sess().span_err(position, &message),
                }
            },
            ErrorType::Help => {
                cx.sess().fileline_help(position, &message);
            },
            ErrorType::Note => {
                cx.sess().fileline_note(position, &message);
            },
            ErrorType::Warning => {
                cx.sess().span_warn(position, &message);
            },
        }
    }
}
