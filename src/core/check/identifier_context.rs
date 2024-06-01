use crate::core::proof_term::Type;

#[derive(Clone)]
pub struct IdentifierContext {
    ctx: Vec<(String, Type)>,
}

impl IdentifierContext {
    pub fn new() -> Self {
        Self { ctx: Vec::new() }
    }

    pub fn insert(&mut self, ident: String, identifer_type: Type) {
        self.ctx.push((ident, identifer_type));
    }

    pub fn get(&self, ident: &String) -> Option<&Type> {
        self.ctx
            .iter()
            .rev()
            .find(|(ctx_ident, _)| ctx_ident == ident)
            .map(|pair| &pair.1)
    }

    pub fn remove(&mut self, ident: &String) -> Option<Type> {
        let idx = self
            .ctx
            .iter()
            .rev()
            .enumerate()
            .find(|(_, (ctx_ident, _))| ctx_ident == ident)
            .map(|(idx, _)| idx);

        if let Some(idx) = idx {
            Some(self.ctx.remove(idx).1)
        } else {
            None
        }
    }
}
