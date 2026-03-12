use std::collections::HashMap;

use nom_supreme::{error::ErrorTree, final_parser::final_parser};

use crate::parser::Parser;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Compiler {
    pub modules: HashMap<String, String>,
}

impl Compiler {
    /// Drop an existing `module` from compiler memory.
    pub fn drop_module(&mut self, name: impl AsRef<str>) -> Option<()> {
        if self.modules.contains_key(name.as_ref()) {
            self.modules.remove(name.as_ref());
            return Some(());
        }
        None
    }

    /// Initialise a new `module` in compiler memory.
    pub fn load_module(&mut self, name: impl AsRef<str>, content: impl AsRef<str>) -> Option<()> {
        if self.modules.contains_key(name.as_ref()) {
            return None;
        }
        self.modules
            .insert(name.as_ref().into(), content.as_ref().into());
        Some(())
    }

    /// Re-bind an existing `module` in compiler memory.
    pub fn bind_module(&mut self, name: impl AsRef<str>, content: impl AsRef<str>) -> Option<()> {
        self.modules.get_mut(name.as_ref()).map(|entry| {
            *entry = content.as_ref().into();
        })
    }

    /// Build all of the modules and get their results.
    pub fn try_build(&self) -> HashMap<String, Result<String, String>> {
        let mut parser = final_parser::<_, _, _, ErrorTree<&str>>(Parser::item);
        self.modules
            .iter()
            .map(|(module, content)| {
                (
                    module.clone(),
                    parser(content)
                        .map(|item| format!("{:?}", item))
                        .map_err(|err| format!("{}", err)),
                )
            })
            .collect()
    }
}
