//! Language features computed from the text of one document.
//!
//! Everything here is a pure function of the source text, so it can be
//! tested without running a server.

use ekips_compiler::ast::{ArrowBody, DeclKind, Expr, ExprKind, Program, Stmt, StmtKind};
use ekips_compiler::diagnostic::Severity;
use ekips_i18n::{Lang, t};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionTextEdit, Diagnostic, DiagnosticSeverity,
    Documentation, Hover, HoverContents, InsertTextFormat, MarkupContent, MarkupKind, Range,
    TextEdit,
};

use crate::knowledge::{self, Entry, Kind};
use crate::position::LineIndex;

/// Name shown as the source of diagnostics in the editor.
pub const SOURCE: &str = "ekips";

/// Turns the compiler's problems into LSP diagnostics.
pub fn diagnostics(text: &str, index: &LineIndex, lang: Lang) -> Vec<Diagnostic> {
    ekips_compiler::parse(text)
        .diagnostics
        .iter()
        .map(|d| {
            let mut message = d.message(lang);
            if let Some(hint) = d.hint(lang) {
                message.push('\n');
                message.push_str(t(lang, "help: ", "Hilfe: "));
                message.push_str(&hint);
            }
            Diagnostic {
                range: index.range(text, d.span.start, d.span.end),
                severity: Some(match d.severity() {
                    Severity::Error => DiagnosticSeverity::ERROR,
                    Severity::Warning => DiagnosticSeverity::WARNING,
                }),
                source: Some(SOURCE.to_string()),
                message,
                ..Diagnostic::default()
            }
        })
        .collect()
}

/// Where the cursor is, roughly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Context {
    Code,
    Comment,
    Text,
}

/// Finds out whether `offset` is inside a comment or a text (string).
fn context_at(text: &str, offset: usize) -> Context {
    let mut chars = text.char_indices().peekable();
    let mut state = Context::Code;
    let mut quote = '"';
    let mut block_comment = false;
    while let Some((i, c)) = chars.next() {
        if i >= offset {
            break;
        }
        match state {
            Context::Code => match c {
                '"' | '\'' => {
                    state = Context::Text;
                    quote = c;
                }
                '/' => match chars.peek().map(|&(_, n)| n) {
                    Some('/') => {
                        chars.next();
                        state = Context::Comment;
                        block_comment = false;
                    }
                    Some('*') => {
                        chars.next();
                        state = Context::Comment;
                        block_comment = true;
                    }
                    _ => {}
                },
                _ => {}
            },
            Context::Comment => {
                if block_comment {
                    if c == '*' && chars.peek().map(|&(_, n)| n) == Some('/') {
                        let (end, _) = chars.next().unwrap_or((i, '/'));
                        // The cursor right after `*/` is code again.
                        if end < offset {
                            state = Context::Code;
                        }
                    }
                } else if c == '\n' {
                    state = Context::Code;
                }
            }
            Context::Text => match c {
                '\\' => {
                    chars.next();
                }
                '\n' => state = Context::Code,
                c if c == quote => state = Context::Code,
                _ => {}
            },
        }
    }
    state
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

/// Start of the word that ends at `offset`.
fn word_start(text: &str, offset: usize) -> usize {
    text[..offset]
        .char_indices()
        .rev()
        .take_while(|&(_, c)| is_word_char(c))
        .last()
        .map_or(offset, |(i, _)| i)
}

/// End of the word that starts at or before `offset`.
fn word_end(text: &str, offset: usize) -> usize {
    text[offset..]
        .char_indices()
        .find(|&(_, c)| !is_word_char(c))
        .map_or(text.len(), |(i, _)| offset + i)
}

/// The object name in front of a `.` that directly precedes `start`.
fn owner_before(text: &str, start: usize) -> Option<&str> {
    let before = text[..start].strip_suffix('.')?;
    let owner_start = word_start(before, before.len());
    let owner = &before[owner_start..];
    (!owner.is_empty() && !owner.starts_with(|c: char| c.is_ascii_digit())).then_some(owner)
}

/// Something the student defined in the program.
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    /// The name
    pub name: String,
    /// What it is
    pub kind: SymbolKind,
    /// Byte offset of the definition
    pub offset: usize,
}

/// The kinds of [`Symbol`].
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    /// A function declaration
    Function {
        /// Parameter names
        params: Vec<String>,
        /// Declared with `async`
        is_async: bool,
    },
    /// A `const` or `let` variable, a parameter or a loop variable
    Variable {
        /// `true` for `const`
        constant: bool,
        /// Class name if the value is created with `new Class(…)`
        class: Option<String>,
    },
}

/// Collects all names defined anywhere in the program.
pub fn symbols(program: &Program) -> Vec<Symbol> {
    let mut out = Vec::new();
    for stmt in &program.body {
        collect_stmt(stmt, &mut out);
    }
    out
}

fn variable(out: &mut Vec<Symbol>, name: &ekips_compiler::ast::Ident, constant: bool) {
    out.push(Symbol {
        name: name.name.clone(),
        kind: SymbolKind::Variable {
            constant,
            class: None,
        },
        offset: name.span.start,
    });
}

fn collect_stmt(stmt: &Stmt, out: &mut Vec<Symbol>) {
    match &stmt.kind {
        StmtKind::VarDecl { kind, name, init } => {
            let class = init.as_ref().and_then(|init| match &init.kind {
                ExprKind::New { callee, .. } => match &callee.kind {
                    ExprKind::Ident(id) => Some(id.name.clone()),
                    _ => None,
                },
                _ => None,
            });
            out.push(Symbol {
                name: name.name.clone(),
                kind: SymbolKind::Variable {
                    constant: *kind == DeclKind::Const,
                    class,
                },
                offset: name.span.start,
            });
            if let Some(init) = init {
                collect_expr(init, out);
            }
        }
        StmtKind::Function(f) => {
            out.push(Symbol {
                name: f.name.name.clone(),
                kind: SymbolKind::Function {
                    params: f.params.iter().map(|p| p.name.clone()).collect(),
                    is_async: f.is_async,
                },
                offset: f.name.span.start,
            });
            for p in &f.params {
                variable(out, p, false);
            }
            f.body.iter().for_each(|s| collect_stmt(s, out));
        }
        StmtKind::Expr(e) | StmtKind::Return(Some(e)) => collect_expr(e, out),
        StmtKind::If {
            cond,
            then,
            otherwise,
        } => {
            collect_expr(cond, out);
            collect_stmt(then, out);
            if let Some(o) = otherwise {
                collect_stmt(o, out);
            }
        }
        StmtKind::While { cond, body } => {
            collect_expr(cond, out);
            collect_stmt(body, out);
        }
        StmtKind::For {
            init,
            cond,
            update,
            body,
        } => {
            if let Some(i) = init {
                collect_stmt(i, out);
            }
            cond.iter().chain(update).for_each(|e| collect_expr(e, out));
            collect_stmt(body, out);
        }
        StmtKind::ForOf {
            kind,
            name,
            iterable,
            body,
        } => {
            variable(out, name, *kind == DeclKind::Const);
            collect_expr(iterable, out);
            collect_stmt(body, out);
        }
        StmtKind::Block(stmts) => stmts.iter().for_each(|s| collect_stmt(s, out)),
        StmtKind::Return(None) | StmtKind::Break | StmtKind::Continue | StmtKind::Empty => {}
    }
}

/// Only arrow functions can define names inside expressions.
fn collect_expr(expr: &Expr, out: &mut Vec<Symbol>) {
    match &expr.kind {
        ExprKind::Arrow { params, body, .. } => {
            for p in params {
                variable(out, p, false);
            }
            match body {
                ArrowBody::Expr(e) => collect_expr(e, out),
                ArrowBody::Block(stmts) => stmts.iter().for_each(|s| collect_stmt(s, out)),
            }
        }
        ExprKind::Array(items) => items.iter().for_each(|e| collect_expr(e, out)),
        ExprKind::Member { object, .. } => collect_expr(object, out),
        ExprKind::Index { object, index } => {
            collect_expr(object, out);
            collect_expr(index, out);
        }
        ExprKind::Call { callee, args } | ExprKind::New { callee, args } => {
            collect_expr(callee, out);
            args.iter().for_each(|e| collect_expr(e, out));
        }
        ExprKind::Unary { operand: e, .. }
        | ExprKind::Update { target: e, .. }
        | ExprKind::Await(e) => collect_expr(e, out),
        ExprKind::Binary { lhs, rhs, .. } => {
            collect_expr(lhs, out);
            collect_expr(rhs, out);
        }
        ExprKind::Conditional {
            cond,
            then,
            otherwise,
        } => {
            collect_expr(cond, out);
            collect_expr(then, out);
            collect_expr(otherwise, out);
        }
        ExprKind::Assign { target, value, .. } => {
            collect_expr(target, out);
            collect_expr(value, out);
        }
        ExprKind::Number { .. }
        | ExprKind::Str(_)
        | ExprKind::Bool(_)
        | ExprKind::Null
        | ExprKind::Undefined
        | ExprKind::Ident(_)
        | ExprKind::Error => {}
    }
}

/// Class or built-in object behind a name: `Port` stays `Port`, a variable
/// created with `new Motor(…)` becomes `Motor`.
fn resolve_owner(owner: &str, symbols: &[Symbol]) -> Option<String> {
    // Definitions in the program hide built-in names.
    if let Some(sym) = symbols.iter().find(|s| s.name == owner) {
        return match &sym.kind {
            SymbolKind::Variable { class, .. } => class.clone(),
            SymbolKind::Function { .. } => None,
        };
    }
    knowledge::members(owner).map(|_| owner.to_string())
}

fn markdown(value: String) -> MarkupContent {
    MarkupContent {
        kind: MarkupKind::Markdown,
        value,
    }
}

fn symbol_markdown(sym: &Symbol, text: &str, index: &LineIndex, lang: Lang) -> String {
    let code = match &sym.kind {
        SymbolKind::Function { params, is_async } => format!(
            "{}function {}({})",
            if *is_async { "async " } else { "" },
            sym.name,
            params.join(", ")
        ),
        SymbolKind::Variable { constant, class } => {
            let decl = if *constant { "const" } else { "let" };
            match class {
                Some(class) => format!("{decl} {} = new {class}(…)", sym.name),
                None => format!("{decl} {}", sym.name),
            }
        }
    };
    let line = index.position(text, sym.offset).line + 1;
    let defined = match lang {
        Lang::En => format!("Defined in line {line}."),
        Lang::De => format!("Definiert in Zeile {line}."),
    };
    let mut out = format!("```ts\n{code}\n```\n\n{defined}");
    if let SymbolKind::Variable {
        class: Some(class), ..
    } = &sym.kind
    {
        if let Some(entry) = knowledge::global(class) {
            out.push_str("\n\n");
            out.push_str(entry.doc(lang));
        }
    }
    out
}

/// Hover information for the position `offset`.
pub fn hover(text: &str, offset: usize, index: &LineIndex, lang: Lang) -> Option<Hover> {
    let offset = offset.min(text.len());
    if context_at(text, offset) != Context::Code {
        return None;
    }
    let start = word_start(text, offset);
    let mut end = word_end(text, offset);
    // `50%` and `90°`: the unit character is not a word character.
    if let Some(unit) = text[end..]
        .chars()
        .next()
        .filter(|c| matches!(c, '%' | '°'))
    {
        if end >= offset {
            end += unit.len_utf8();
        }
    }
    let word = &text[start..end];
    if word.is_empty() {
        return None;
    }
    let range = Some(index.range(text, start, end));
    let reply = |value: String| {
        Some(Hover {
            contents: HoverContents::Markup(markdown(value)),
            range,
        })
    };

    // A number with a unit, e.g. `20cm`.
    if word.starts_with(|c: char| c.is_ascii_digit()) {
        let suffix = word.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.');
        let entry = knowledge::unit(suffix)?;
        return reply(entry.hover_markdown(None, lang));
    }
    // A `%` that does not follow a number is the remainder operator.
    if word.ends_with(['%', '°']) {
        return None;
    }

    let program = ekips_compiler::parse(text).program;
    let symbols = symbols(&program);

    if let Some(owner) = owner_before(text, start) {
        let class = resolve_owner(owner, &symbols)?;
        let entry = knowledge::member(&class, word)?;
        return reply(entry.hover_markdown(Some(&class), lang));
    }
    if let Some(sym) = symbols.iter().find(|s| s.name == word) {
        return reply(symbol_markdown(sym, text, index, lang));
    }
    let entry = knowledge::global(word)?;
    reply(entry.hover_markdown(None, lang))
}

fn item_kind(kind: Kind) -> CompletionItemKind {
    match kind {
        Kind::Keyword => CompletionItemKind::KEYWORD,
        Kind::Unit => CompletionItemKind::UNIT,
        Kind::Class => CompletionItemKind::CLASS,
        Kind::Object => CompletionItemKind::MODULE,
        Kind::Function => CompletionItemKind::FUNCTION,
        Kind::Method => CompletionItemKind::METHOD,
        Kind::Property => CompletionItemKind::PROPERTY,
        Kind::Constant => CompletionItemKind::ENUM_MEMBER,
    }
}

fn entry_item(entry: &Entry, owner: Option<&str>, lang: Lang, snippets: bool) -> CompletionItem {
    let callable = matches!(entry.kind, Kind::Method | Kind::Function);
    let (insert_text, insert_text_format) = if callable && snippets {
        (
            Some(format!("{}($0)", entry.name)),
            Some(InsertTextFormat::SNIPPET),
        )
    } else {
        (None, None)
    };
    CompletionItem {
        label: entry.name.to_string(),
        kind: Some(item_kind(entry.kind)),
        detail: Some(entry.signature.lines().next().unwrap_or("").to_string()),
        documentation: Some(Documentation::MarkupContent(markdown(
            entry.hover_markdown(owner, lang),
        ))),
        insert_text,
        insert_text_format,
        ..CompletionItem::default()
    }
}

/// Completion options that depend on the editor.
#[derive(Debug, Clone, Copy)]
pub struct CompletionOptions {
    /// The editor understands snippet syntax (`$1`, `${1:name}`).
    pub snippets: bool,
}

/// Suggestions for the position `offset`.
pub fn completion(
    text: &str,
    offset: usize,
    index: &LineIndex,
    lang: Lang,
    opts: CompletionOptions,
) -> Vec<CompletionItem> {
    let offset = offset.min(text.len());
    if context_at(text, offset) != Context::Code {
        return Vec::new();
    }
    let start = word_start(text, offset);
    let word = &text[start..offset];

    // Units after a number: `20c|` → `cm`.
    if word.starts_with(|c: char| c.is_ascii_digit()) {
        let digits = word
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(word.len());
        let range: Range = index.range(text, start + digits, offset);
        return knowledge::UNITS
            .iter()
            .filter(|u| u.name != "%")
            .map(|u| CompletionItem {
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range,
                    new_text: u.name.to_string(),
                })),
                ..entry_item(u, None, lang, false)
            })
            .collect();
    }

    let program = ekips_compiler::parse(text).program;
    let symbols = symbols(&program);

    // Members after a dot: `robot.` → `drive`, `turn`, …
    if let Some(owner) = owner_before(text, start) {
        let Some(class) = resolve_owner(owner, &symbols) else {
            return Vec::new();
        };
        return knowledge::members(&class)
            .unwrap_or_default()
            .iter()
            .map(|m| entry_item(m, Some(&class), lang, opts.snippets))
            .collect();
    }

    let mut items: Vec<CompletionItem> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for sym in &symbols {
        if !seen.insert(sym.name.as_str()) {
            continue;
        }
        let (kind, detail) = match &sym.kind {
            SymbolKind::Function { params, .. } => (
                CompletionItemKind::FUNCTION,
                format!("{}({})", sym.name, params.join(", ")),
            ),
            SymbolKind::Variable { class, constant } => (
                if *constant {
                    CompletionItemKind::CONSTANT
                } else {
                    CompletionItemKind::VARIABLE
                },
                class.clone().unwrap_or_default(),
            ),
        };
        items.push(CompletionItem {
            label: sym.name.clone(),
            kind: Some(kind),
            detail: Some(detail),
            documentation: Some(Documentation::MarkupContent(markdown(symbol_markdown(
                sym, text, index, lang,
            )))),
            // Names from the program first.
            sort_text: Some(format!("0{}", sym.name)),
            ..CompletionItem::default()
        });
    }
    for entry in knowledge::GLOBALS.iter().chain(knowledge::KEYWORDS) {
        if seen.insert(entry.name) {
            items.push(CompletionItem {
                sort_text: Some(format!("1{}", entry.name)),
                ..entry_item(entry, None, lang, opts.snippets)
            });
        }
    }
    if opts.snippets {
        for snippet in knowledge::SNIPPETS {
            items.push(CompletionItem {
                label: snippet.label.to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some(snippet.detail(lang).to_string()),
                documentation: Some(Documentation::MarkupContent(markdown(format!(
                    "```ts\n{}\n```",
                    snippet.plain_text().replace('\t', "  ")
                )))),
                insert_text: Some(snippet.body.to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some(format!("2{}", snippet.label)),
                ..CompletionItem::default()
            });
        }
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Encoding;

    const PROGRAM: &str = "\
// drive a square
const robot = new DriveBase(new Motor(Port.A), new Motor(Port.B), 56mm, 112mm);
const eye = new ColorSensor(Port.C);

async function square(side) {
  await robot.drive(side);
  hub.show(\"robot.\");
}

async function main() {
  await square(20cm);
}
";

    fn idx(text: &str) -> LineIndex {
        LineIndex::new(text, Encoding::Utf16)
    }

    fn hover_text(text: &str, needle: &str, lang: Lang) -> Option<String> {
        let offset = text.find(needle).expect("needle not found") + 1;
        hover(text, offset, &idx(text), lang).map(|h| match h.contents {
            HoverContents::Markup(m) => m.value,
            other => panic!("unexpected hover {other:?}"),
        })
    }

    fn labels(text: &str, offset: usize) -> Vec<String> {
        completion(
            text,
            offset,
            &idx(text),
            Lang::En,
            CompletionOptions { snippets: true },
        )
        .into_iter()
        .map(|i| i.label)
        .collect()
    }

    #[test]
    fn diagnostics_have_ranges_and_hints() {
        let text = "cosnt x = 1;\nasync function main() {}\n";
        let diags = diagnostics(text, &idx(text), Lang::De);
        assert_eq!(diags.len(), 1);
        let d = &diags[0];
        assert_eq!(d.range.start, lsp_types::Position::new(0, 0));
        assert_eq!(d.range.end, lsp_types::Position::new(0, 5));
        assert!(d.message.contains("Meintest du `const`?"), "{}", d.message);
        assert_eq!(d.severity, Some(DiagnosticSeverity::ERROR));
    }

    #[test]
    fn valid_program_has_no_diagnostics() {
        assert!(diagnostics(PROGRAM, &idx(PROGRAM), Lang::En).is_empty());
    }

    #[test]
    fn hover_on_keyword() {
        let en = hover_text(PROGRAM, "await robot", Lang::En).unwrap();
        assert!(en.contains("Waits until"), "{en}");
        let de = hover_text(PROGRAM, "await robot", Lang::De).unwrap();
        assert!(de.contains("Wartet, bis"), "{de}");
    }

    #[test]
    fn hover_on_method_of_variable() {
        let text = hover_text(PROGRAM, "drive(side)", Lang::En).unwrap();
        assert!(text.contains("drive(distance, speed?)"), "{text}");
    }

    #[test]
    fn hover_on_builtin_member() {
        let text = hover_text(PROGRAM, "show(", Lang::De).unwrap();
        assert!(text.contains("hub.show(value)"), "{text}");
        assert!(text.contains("Lichtmatrix"), "{text}");
        let port = hover_text(PROGRAM, "C);", Lang::En).unwrap();
        assert!(port.contains("Port C"), "{port}");
    }

    #[test]
    fn hover_on_unit() {
        let text = hover_text(PROGRAM, "20cm", Lang::En).unwrap();
        assert!(text.contains("Centimetres"), "{text}");
        let percent = "const s = 50%;\n";
        let offset = percent.find('%').unwrap();
        let h = hover(percent, offset, &idx(percent), Lang::En).unwrap();
        assert!(matches!(h.contents, HoverContents::Markup(m) if m.value.contains("Percent")));
    }

    #[test]
    fn hover_on_user_function() {
        let text = hover_text(PROGRAM, "square(20cm)", Lang::En).unwrap();
        assert!(text.contains("async function square(side)"), "{text}");
        assert!(text.contains("Defined in line 5."), "{text}");
    }

    #[test]
    fn hover_on_variable_shows_class() {
        let text = hover_text(PROGRAM, "eye =", Lang::En).unwrap();
        assert!(text.contains("const eye = new ColorSensor(…)"), "{text}");
    }

    #[test]
    fn no_hover_in_comments_and_texts() {
        assert!(hover_text(PROGRAM, "drive a square", Lang::En).is_none());
        assert!(hover_text(PROGRAM, "robot.\"", Lang::En).is_none());
    }

    #[test]
    fn completion_after_dot_uses_class_of_variable() {
        let text = "const robot = new DriveBase(a, b, 56mm, 112mm);\nrobot.";
        let items = labels(text, text.len());
        assert!(items.contains(&"drive".to_string()), "{items:?}");
        assert!(items.contains(&"turn".to_string()), "{items:?}");
        assert!(!items.contains(&"while".to_string()), "{items:?}");
    }

    #[test]
    fn completion_after_builtin_object() {
        let text = "const m = new Motor(Port.";
        let items = labels(text, text.len());
        assert_eq!(items, ["A", "B", "C", "D", "E", "F"]);
        let text = "hub.sh";
        assert!(labels(text, text.len()).contains(&"show".to_string()));
    }

    #[test]
    fn completion_offers_keywords_snippets_and_own_names() {
        let text = "const speed = 50%;\nasync function main() {\n  \n}\n";
        let offset = text.find("  \n").unwrap() + 2;
        let items = labels(text, offset);
        for expected in ["while", "Motor", "main", "speed", "waitUntil", "program"] {
            assert!(
                items.contains(&expected.to_string()),
                "{expected}: {items:?}"
            );
        }
    }

    #[test]
    fn completion_without_snippet_support() {
        let text = "";
        let items = completion(
            text,
            0,
            &idx(text),
            Lang::En,
            CompletionOptions { snippets: false },
        );
        assert!(
            items
                .iter()
                .all(|i| i.kind != Some(CompletionItemKind::SNIPPET))
        );
        assert!(items.iter().all(|i| i.insert_text_format.is_none()));
    }

    #[test]
    fn completion_of_units() {
        let text = "await robot.drive(20c";
        let items = completion(
            text,
            text.len(),
            &idx(text),
            Lang::En,
            CompletionOptions { snippets: true },
        );
        let cm = items.iter().find(|i| i.label == "cm").expect("cm offered");
        let Some(CompletionTextEdit::Edit(edit)) = &cm.text_edit else {
            panic!("no text edit");
        };
        // Only the `c` after `20` is replaced.
        assert_eq!(edit.range.start.character, 20);
        assert_eq!(edit.range.end.character, 21);
    }

    #[test]
    fn no_completion_in_comments() {
        let text = "// robot.";
        assert!(labels(text, text.len()).is_empty());
        let text = "/* a */ ";
        assert!(!labels(text, text.len()).is_empty());
    }

    #[test]
    fn context_detection() {
        let text = "a 'b' /* c */ d // e\nf";
        assert_eq!(context_at(text, text.find('a').unwrap()), Context::Code);
        assert_eq!(context_at(text, text.find('b').unwrap()), Context::Text);
        assert_eq!(context_at(text, text.find('c').unwrap()), Context::Comment);
        assert_eq!(context_at(text, text.find('d').unwrap()), Context::Code);
        assert_eq!(context_at(text, text.find('e').unwrap()), Context::Comment);
        assert_eq!(context_at(text, text.find('f').unwrap()), Context::Code);
    }
}
