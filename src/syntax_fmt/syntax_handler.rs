use std::sync::Arc;

use super::bin_op_fmt::BinOpHandler;
use super::branch_fmt::BranchHandler;
use super::call_fmt::CallHandler;
use super::fun_fmt::FunHandler;
use super::let_fmt::LetHandler;
use super::quant_fmt::QuantHandler;
use super::skip_fmt::SkipHandler;
use super::syntax_trait::*;
use move_compiler::parser::ast::Definition;

pub struct SyntaxHandlerV2 {
    pub(crate) handlers: Vec<Box<dyn Preprocessor>>,
}

macro_rules! create_handlers {
    ($content:expr, $($handler:ty),*) => {
        vec![
            $(
                Box::new(<$handler>::new($content.to_string())) as Box<dyn Preprocessor>,
            )*
        ]
    };
}

// macro_rules! get_handler_fn {
//     ($name:ident, $idx:expr) => {
//         pub fn $name<HandlerType>(&mut self) -> &mut HandlerType
//         where
//             HandlerType: Preprocessor + 'static,
//         {
//             self.handlers[$idx]
//                 .as_any_mut()
//                 .downcast_mut::<HandlerType>()
//                 .expect("handler type mismatch")
//         }
//     };
// }

// macro_rules! get_immut_handler_fn {
//     ($name:ident, $idx:expr) => {
//         pub fn $name<HandlerType>(& self) -> & HandlerType
//         where
//             HandlerType: Preprocessor + 'static,
//         {
//             self.handlers[$idx]
//                 .as_any()
//                 .downcast_ref::<HandlerType>()
//                 .expect("handler type mismatch")
//         }
//     };
// }

impl SyntaxHandlerV2 {
    pub fn new(content: &str) -> Self {
        let handlers = create_handlers!(
            content,
            BranchHandler,
            FunHandler,
            CallHandler,
            LetHandler,
            BinOpHandler,
            QuantHandler,
            SkipHandler
        );
        Self { handlers }
    }

    pub fn preprocess(&mut self, module_defs: &Arc<Vec<Definition>>) {
        for p in &mut self.handlers {
            p.preprocess(&module_defs);
        }
    }

    // for i in self.handlers.len() {
    //     get_handler_fn!(i);
    //     get_immut_handler_fn!(i);
    // }

    // get_handler_fn!(0);
    // get_handler_fn!(1);
    // get_handler_fn!(2);
    // get_handler_fn!(3);
    // get_handler_fn!(4);
    // get_handler_fn!(5);
    // get_handler_fn!(6);

    // get_immut_handler_fn!(0);
    // get_immut_handler_fn!(1);
    // get_immut_handler_fn!(2);
    // get_immut_handler_fn!(3);
    // get_immut_handler_fn!(4);
    // get_immut_handler_fn!(5);
    // get_immut_handler_fn!(6);

    pub fn handler<HandlerType>(&mut self) -> &mut HandlerType
    where
        HandlerType: Preprocessor + 'static,
    {
        self.handlers
            .iter_mut()
            .find(|h| h.as_any().type_id() == std::any::TypeId::of::<HandlerType>())
            .and_then(|h| h.as_any_mut().downcast_mut::<HandlerType>())
            .expect("handler type mismatch")
    }

    pub fn handler_immut<HandlerType>(&self) -> &HandlerType
    where
        HandlerType: Preprocessor + 'static,
    {
        self.handlers
            .iter()
            .find(|h| h.as_any().type_id() == std::any::TypeId::of::<HandlerType>())
            .and_then(|h| h.as_any().downcast_ref::<HandlerType>())
            .expect("handler type mismatch")
    }
}
