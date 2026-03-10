use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compiler {
    pub modules: HashMap<String, String>,
}

impl Compiler {
    pub fn drop_module(&mut self, name: impl AsRef<str>) -> Option<()> {
        if self.modules.contains_key(name.as_ref()) {
            self.modules.remove(name.as_ref());
            return Some(());
        }
        None
    }

    pub fn load_module(&mut self, name: impl AsRef<str>, content: impl AsRef<str>) -> Option<()> {
        if self.modules.contains_key(name.as_ref()) {
            return None;
        }
        self.modules
            .insert(name.as_ref().into(), content.as_ref().into());
        Some(())
    }

    pub fn bind_module(&mut self, name: impl AsRef<str>, content: impl AsRef<str>) -> Option<()> {
        self.modules.get_mut(name.as_ref()).map(|entry| {
            *entry = content.as_ref().into();
        })
    }
}
