use core::panic;
use std::{any::Any, collections::HashMap, fs, path::PathBuf, rc::Rc};

use crate::{
	data::Data,
	error::BeanError,
	evaluator, lexer, parser,
	scope::{function::Function, Scope},
	util::{make_ref, MutRc},
};

use super::{
	registry::{ModuleRegistry, RegistryEntry},
	CustomModule, Module,
};

#[derive(Debug)]
pub struct ModuleWrapper(MutRc<dyn Module>);

impl Scope for ModuleWrapper {
	fn has_function(&self, name: &str) -> bool {
		self.0.borrow().has_pub_function(name)
	}

	fn get_function(&self, name: &str) -> Option<Function> {
		self.0.borrow().get_pub_function(name)
	}

	fn set_function(&mut self, _name: &str, _function: Function) {
		panic!("Tried to set function inside external module.")
	}

	fn delete_function(&mut self, _name: &str) {
		panic!("Tried to delete function inside external module.")
	}

	fn set_return_value(&mut self, _value: Data) {}

	fn get_function_list(&self) -> HashMap<String, Function> {
		self.0.borrow().get_function_list()
	}

	fn as_any(&self) -> &dyn Any {
		panic!("INTERNAL! tried to cast ModuleWrapper")
	}
	fn as_mut(&mut self) -> &mut dyn Any {
		self
	}
}

pub fn get(
	registry: MutRc<ModuleRegistry>,
	path: String,
) -> Result<MutRc<ModuleWrapper>, BeanError> {
	if path.starts_with("./") {
		get_local(
			&registry.borrow().local,
			Rc::clone(&registry),
			PathBuf::from(path.clone() + ".bean"),
		)
		.map(|m| make_ref(ModuleWrapper(m)))
	} else {
		get_reg(&mut registry.borrow_mut().registered, path.clone()).map_or(
			Err(BeanError::new(
				&format!("Module {} does not exist.", path),
				None,
			)),
			|s| Ok(make_ref(ModuleWrapper(s))),
		)
	}
}

fn get_reg(
	registered: &mut HashMap<String, RegistryEntry>,
	name: String,
) -> Option<MutRc<dyn Module>> {
	if let Some(RegistryEntry::Uninitialized(_)) = registered.get(&name) {
		let v = registered.remove(&name).unwrap().get_or_init();
		registered.insert(name.clone(), RegistryEntry::Available(v));
	}

	registered.get(&name).map_or(None, |x| match x {
		RegistryEntry::Available(r) => Some(Rc::clone(r)),
		RegistryEntry::Uninitialized(_) => None,
	})
}

fn get_local(
	local: &HashMap<PathBuf, MutRc<CustomModule>>,
	registry: MutRc<ModuleRegistry>,
	path: PathBuf,
) -> Result<MutRc<CustomModule>, BeanError> {
	if registry.borrow().loading.contains(&path) {
		return Err(BeanError::new(
			"Trying to load from a file that is currently being loaded.",
			None,
		));
	}
	match local.get(&path) {
		None => {
			let file = fs::read_to_string(path.clone()).map_err(|e| {
				BeanError::new(
					&(String::from("Error reading file ")
						+ path.to_str().unwrap_or("")
						+ ":" + &e.to_string()),
					None,
				)
			})?;

			let tokens = lexer::tokenize(file);
			let tree = parser::parse(tokens);

			let module = CustomModule::new(Rc::clone(&registry), path.clone());
			let module_ref = make_ref(module);
			evaluator::evaluate(&tree, CustomModule::to_scope(Rc::clone(&module_ref)));
			registry.borrow_mut().local.insert(path.clone(), module_ref);
		}
		Some(_) => (),
	}

	Ok(local.get(&path).unwrap().clone())
}
