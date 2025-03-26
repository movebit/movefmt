use move_compiler::parser::ast::*;

pub trait SyntaxExtractor {
    fn collect_seq_item(&mut self, s: &SequenceItem);
    fn collect_seq(&mut self, s: &Sequence);
    fn collect_spec(&mut self, spec_block: &SpecBlock);
    fn collect_expr(&mut self, e: &Exp);
    fn collect_const(&mut self, c: &Constant);
    fn collect_function(&mut self, d: &Function);
    fn collect_module(&mut self, d: &ModuleDefinition);
    fn collect_script(&mut self, d: &Script);
    fn collect_definition(&mut self, d: &Definition);
}
