use std::sync::Arc;

use move_compiler::parser::ast::*;

pub trait SingleSyntaxExtractor {
    fn new(fmt_buffer: String) -> Self;
    fn collect_seq_item(&mut self, s: &SequenceItem);
    fn collect_seq(&mut self, s: &Sequence);
    fn collect_spec(&mut self, spec_block: &SpecBlock);
    fn collect_expr(&mut self, e: &Exp);
    fn collect_const(&mut self, c: &Constant);
    fn collect_struct(&mut self, s: &StructDefinition);
    fn collect_function(&mut self, d: &Function);
    fn collect_module(&mut self, d: &ModuleDefinition);
    fn collect_script(&mut self, d: &Script);
    fn collect_definition(&mut self, d: &Definition);
}

pub trait Preprocessor {
    fn preprocess(&mut self, module_defs: &Arc<Vec<Definition>>);

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    fn as_any(&self) -> &dyn std::any::Any;
}
