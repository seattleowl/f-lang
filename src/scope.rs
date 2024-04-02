use std::{any::Any, cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use crate::{data::Data, modules::CustomModule};
use function::{CallScope, Function};

pub mod block_scope;
pub mod function;

pub type ScopeRef = Rc<RefCell<dyn Scope>>;

pub trait Scope: Debug {
	fn has_function(&self, name: &str) -> bool;
	fn get_function(&self, name: &str) -> Option<Function>;
	fn set_function(&mut self, name: &str, function: Function);
	fn delete_function(&mut self, name: &str);

	fn parent(&self) -> Option<ScopeRef> {
		None
	}

	fn get_call_scope(&self) -> Option<Rc<RefCell<CallScope>>> {
		self.parent().map_or(None, |p| p.borrow().get_call_scope())
	}
	fn get_file_module(&self) -> Option<ScopeRef> {
		dbg!(&self);
		self.parent().map_or(None, |p| {
			let parent = p.borrow();
			match parent.as_any().downcast_ref::<CustomModule>() {
				Some(_) => self.parent(),
				None => parent.get_file_module(),
			}
		})
	}
	fn set_return_value(&mut self, value: Data);
	fn get_function_list(&self) -> HashMap<String, Function>;

	fn as_any(&self) -> &dyn Any;
	fn as_mut(&mut self) -> &mut dyn Any;

	fn to_string(&self) -> String {
		String::from("[scope]")
	}
}
