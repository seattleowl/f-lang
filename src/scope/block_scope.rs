use std::{any::Any, borrow::Borrow, cell::RefCell, collections::HashMap, rc::Rc};

use crate::data::Data;

use super::{
	function::{CallScope, Function},
	Scope,
};

#[derive(Debug)]
pub enum IfState {
	Started,
	Captured,
	Finished,
}

#[derive(Debug)]
pub struct BlockScope {
	local_functions: HashMap<String, Function>,
	parent: Option<Rc<RefCell<dyn Scope>>>,
	pub return_value: Data,
	pub if_state: IfState,
	pub match_value: Option<Data>,
}

impl BlockScope {
	pub fn new(parent: Option<Rc<RefCell<dyn Scope>>>) -> Self {
		Self {
			local_functions: HashMap::new(),
			parent,
			return_value: Data::None,
			if_state: IfState::Finished,
			match_value: None,
		}
	}
}

impl Scope for BlockScope {
	fn has_function(&self, name: &str) -> bool {
		if self.local_functions.contains_key(name) {
			true
		} else if let Some(parent) = &self.parent {
			let borrow: &RefCell<dyn Scope> = parent.borrow();
			borrow.borrow().has_function(name)
		} else {
			false
		}
	}

	fn get_function(&self, name: &str) -> Option<Function> {
		let function = self.local_functions.get(name);
		if function.is_some() {
			function.map(|x| x.clone())
		} else if let Some(parent) = &self.parent {
			let borrow: &RefCell<dyn Scope> = parent.borrow();
			borrow.borrow().get_function(name).map(|x| x.clone())
		} else {
			None
		}
	}

	fn set_function(&mut self, name: &str, function: Function) {
		if self.local_functions.contains_key(name) {
			*self.local_functions.get_mut(name).unwrap() = function
		} else {
			self.local_functions.insert(String::from(name), function);
		}
	}

	fn delete_function(&mut self, name: &str) {
		self.local_functions.remove(name);
	}

	fn parent(&self) -> Option<Rc<RefCell<dyn Scope>>> {
		self.parent.as_ref().map(|x| Rc::clone(x))
	}
	fn get_call_scope(&self) -> Option<Rc<RefCell<CallScope>>> {
		if let Some(p) = &self.parent {
			RefCell::borrow(&p).get_call_scope()
		} else {
			None
		}
	}

	fn as_any(&self) -> &dyn Any {
		self
	}
}
