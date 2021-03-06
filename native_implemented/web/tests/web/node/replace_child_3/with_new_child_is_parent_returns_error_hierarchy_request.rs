#[path = "with_new_child_is_parent_returns_error_hierarchy_request/label_1.rs"]
pub mod label_1;
#[path = "with_new_child_is_parent_returns_error_hierarchy_request/label_2.rs"]
pub mod label_2;
#[path = "with_new_child_is_parent_returns_error_hierarchy_request/label_3.rs"]
pub mod label_3;
#[path = "with_new_child_is_parent_returns_error_hierarchy_request/label_4.rs"]
pub mod label_4;

use liblumen_alloc::erts::process::{Frame, Native};
use liblumen_alloc::erts::term::prelude::*;
use liblumen_alloc::{Arity, ModuleFunctionArity};

const ARITY: Arity = 0;

fn frame_for_native(native: Native) -> Frame {
    Frame::new(module_function_arity(), native)
}

fn function() -> Atom {
    Atom::from_str("replace_child_3_with_new_child_is_parent_returns_error_hierarchy_request")
}

fn module() -> Atom {
    Atom::from_str("Lumen.Web.NodeTest")
}

fn module_function_arity() -> ModuleFunctionArity {
    ModuleFunctionArity {
        module: module(),
        function: function(),
        arity: ARITY,
    }
}
