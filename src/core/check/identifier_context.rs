use crate::core::proof_term::Type;

#[derive(Clone)]
pub struct IdentifierContext {
    ctx: Vec<(String, Type)>,
}

impl IdentifierContext {
    pub fn new() -> Self {
        Self { ctx: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.ctx.len()
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
            .map(|(idx, _)| self.ctx.len() - 1 - idx);

        if let Some(idx) = idx {
            Some(self.ctx.remove(idx).1)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::proof_term::Type;

    use super::IdentifierContext;

    #[test]
    fn test_simple_get() {
        let mut ctx = IdentifierContext::new();
        ctx.insert("a".to_string(), Type::Datatype("t".to_string()));

        assert_eq!(
            ctx.get(&"a".to_string()),
            Some(&Type::Datatype("t".to_string()))
        );

        assert_eq!(1, ctx.len())
    }

    #[test]
    fn test_filled_get() {
        let mut ctx = IdentifierContext::new();
        ctx.insert("a".to_string(), Type::Datatype("x".to_string()));
        ctx.insert("b".to_string(), Type::Datatype("y".to_string()));
        ctx.insert("c".to_string(), Type::Datatype("z".to_string()));

        assert_eq!(
            ctx.get(&"b".to_string()),
            Some(&Type::Datatype("y".to_string()))
        );

        assert_eq!(3, ctx.len())
    }

    #[test]
    fn test_simple_remove() {
        let mut ctx = IdentifierContext::new();
        ctx.insert("a".to_string(), Type::Datatype("x".to_string()));
        ctx.insert("b".to_string(), Type::Datatype("y".to_string()));

        assert_eq!(
            ctx.remove(&"a".to_string()),
            Some(Type::Datatype("x".to_string()))
        );

        assert_eq!(ctx.remove(&"a".to_string()), None);

        assert_eq!(
            ctx.get(&"b".to_string()),
            Some(&Type::Datatype("y".to_string()))
        );

        assert_eq!(1, ctx.len())
    }

    #[test]
    fn test_filled_remove() {
        let mut ctx = IdentifierContext::new();
        ctx.insert("a".to_string(), Type::Datatype("x".to_string()));
        ctx.insert("b".to_string(), Type::Datatype("y".to_string()));
        ctx.insert("c".to_string(), Type::Datatype("z".to_string()));

        assert_eq!(
            ctx.remove(&"b".to_string()),
            Some(Type::Datatype("y".to_string()))
        );

        assert_eq!(ctx.remove(&"b".to_string()), None);

        assert_eq!(
            ctx.get(&"a".to_string()),
            Some(&Type::Datatype("x".to_string()))
        );

        assert_eq!(
            ctx.get(&"c".to_string()),
            Some(&Type::Datatype("z".to_string()))
        );

        assert_eq!(2, ctx.len())
    }

    #[test]
    fn test_shadowing_1() {
        let mut ctx = IdentifierContext::new();
        ctx.insert("a".to_string(), Type::Datatype("w".to_string()));
        ctx.insert("b".to_string(), Type::Datatype("x".to_string()));
        ctx.insert("a".to_string(), Type::Datatype("y".to_string()));
        ctx.insert("a".to_string(), Type::Datatype("z".to_string()));

        assert_eq!(4, ctx.len());
        assert_eq!(
            ctx.remove(&"a".to_string()),
            Some(Type::Datatype("z".to_string()))
        );

        assert_eq!(3, ctx.len());
        assert_eq!(
            ctx.remove(&"a".to_string()),
            Some(Type::Datatype("y".to_string()))
        );

        assert_eq!(2, ctx.len());
        assert_eq!(
            ctx.remove(&"a".to_string()),
            Some(Type::Datatype("w".to_string()))
        );

        assert_eq!(1, ctx.len());
        assert_eq!(
            ctx.get(&"b".to_string()),
            Some(&Type::Datatype("x".to_string()))
        )
    }

    #[test]
    fn test_shadowing_2() {
        let mut ctx = IdentifierContext::new();
        ctx.insert("a".to_string(), Type::Datatype("v".to_string()));
        ctx.insert("b".to_string(), Type::Datatype("w".to_string()));
        ctx.insert("c".to_string(), Type::Datatype("x".to_string()));
        ctx.insert("a".to_string(), Type::Datatype("y".to_string()));
        ctx.insert("a".to_string(), Type::Datatype("z".to_string()));

        assert_eq!(
            ctx.get(&"a".to_string()),
            Some(&Type::Datatype("z".to_string()))
        );
        assert_eq!(5, ctx.len());

        assert_eq!(
            ctx.remove(&"b".to_string()),
            Some(Type::Datatype("w".to_string()))
        );
        assert_eq!(4, ctx.len());

        assert_eq!(
            ctx.get(&"a".to_string()),
            Some(&Type::Datatype("z".to_string()))
        );
        assert_eq!(4, ctx.len());

        assert_eq!(
            ctx.remove(&"c".to_string()),
            Some(Type::Datatype("x".to_string()))
        );
        assert_eq!(3, ctx.len());

        assert_eq!(
            ctx.remove(&"a".to_string()),
            Some(Type::Datatype("z".to_string()))
        );
        assert_eq!(2, ctx.len());

        assert_eq!(
            ctx.remove(&"a".to_string()),
            Some(Type::Datatype("y".to_string()))
        );
        assert_eq!(1, ctx.len());

        assert_eq!(
            ctx.remove(&"a".to_string()),
            Some(Type::Datatype("v".to_string()))
        );
        assert_eq!(0, ctx.len());
    }
}
