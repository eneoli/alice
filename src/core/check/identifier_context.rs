#[derive(Clone)]
pub enum IdentifierType {
    Variable,
    Datatype,
}

#[derive(Clone)]
pub struct IdentifierContext {
    ctx: Vec<(String, IdentifierType)>,
}

impl IdentifierContext {
    pub fn insert(&mut self, ident: String, identifer_type: IdentifierType) {
        self.ctx.push((ident, identifer_type));
    }

    pub fn get(&self, ident: &String) -> Option<&IdentifierType> {
        self.ctx
            .iter()
            .rev()
            .find(|(ctx_ident, _)| ctx_ident == ident)
            .map(|pair| &pair.1)
    }

    pub fn remove(&mut self, ident: &String) -> Option<IdentifierType> {
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
