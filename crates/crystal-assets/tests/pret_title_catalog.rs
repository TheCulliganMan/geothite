#[path = "../src/bin/pack_core/source_syntax.rs"]
mod source_syntax;
mod radio_text_export {
    pub(crate) use super::source_syntax::code_part;
}
#[path = "../src/bin/pack_core/source_expressions.rs"]
mod source_expressions;
#[path = "../src/bin/pack_core/source_title_catalog.rs"]
mod source_title_catalog;
#[path = "../src/bin/pack_core/source_title_screen.rs"]
mod source_title_screen;
