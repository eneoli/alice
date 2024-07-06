use crate::kernel::proof_term::Type;

use super::identifier::Identifier;

#[derive(Clone)]
pub struct IdentifierContext {
    ctx: Vec<(Identifier, Type)>,
}

impl IdentifierContext {
    pub fn new() -> Self {
        Self { ctx: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.ctx.len()
    }

    pub fn insert(&mut self, ident: Identifier, identifer_type: Type) {
        self.ctx.push((ident, identifer_type));
    }

    pub fn get_by_name(&self, ident: &String) -> Option<(&Identifier, &Type)> {
        self.ctx
            .iter()
            .rev()
            .find(|(ctx_ident, _)| ctx_ident.name() == ident)
            .map(|(a, b)| (a, b))
    }

    pub fn get(&self, ident: &Identifier) -> Option<&Type> {
        self.ctx
            .iter()
            .rev()
            .find(|(ctx_ident, _)| ctx_ident == ident)
            .map(|ident| &ident.1)
    }

    pub fn remove(&mut self, ident: &Identifier) -> Option<Type> {
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

    pub fn remove_by_name(&mut self, ident: &String) -> Option<(Identifier, Type)> {
        let idx = self
            .ctx
            .iter()
            .rev()
            .enumerate()
            .find(|(_, (ctx_ident, _))| ctx_ident.name() == ident)
            .map(|(idx, _)| self.ctx.len() - 1 - idx);

        if let Some(idx) = idx {
            Some(self.ctx.remove(idx))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::kernel::{checker::identifier::Identifier, proof_term::Type};

    use super::IdentifierContext;

    #[test]
    fn test_simple_get() {
        let mut ctx = IdentifierContext::new();
        let identifier = Identifier::new("a".to_string(), 42);

        ctx.insert(identifier.clone(), Type::Datatype("t".to_string()));

        assert_eq!(ctx.get(&identifier), Some(&Type::Datatype("t".to_string())));

        assert_eq!(1, ctx.len())
    }

    #[test]
    fn test_simple_get_by_name() {
        let mut ctx = IdentifierContext::new();
        let identifier = Identifier::new("a".to_string(), 42);

        ctx.insert(identifier.clone(), Type::Datatype("t".to_string()));

        assert_eq!(
            ctx.get_by_name(&"a".to_string()),
            Some((&identifier, &Type::Datatype("t".to_string())))
        );

        assert_eq!(1, ctx.len())
    }

    #[test]
    fn test_filled_get() {
        let mut ctx = IdentifierContext::new();
        let b_identifier = Identifier::new("b".to_string(), 2);
        ctx.insert(
            Identifier::new("a".to_string(), 1),
            Type::Datatype("x".to_string()),
        );
        ctx.insert(b_identifier.clone(), Type::Datatype("y".to_string()));
        ctx.insert(
            Identifier::new("c".to_string(), 3),
            Type::Datatype("z".to_string()),
        );

        assert_eq!(
            ctx.get(&b_identifier),
            Some(&Type::Datatype("y".to_string()))
        );

        assert_eq!(3, ctx.len())
    }

    #[test]
    fn test_filled_get_by_name() {
        let mut ctx = IdentifierContext::new();
        let b_identifier = Identifier::new("b".to_string(), 2);
        ctx.insert(
            Identifier::new("a".to_string(), 1),
            Type::Datatype("x".to_string()),
        );
        ctx.insert(b_identifier.clone(), Type::Datatype("y".to_string()));
        ctx.insert(
            Identifier::new("c".to_string(), 3),
            Type::Datatype("z".to_string()),
        );

        assert_eq!(
            ctx.get_by_name(&"b".to_string()),
            Some((&b_identifier, &Type::Datatype("y".to_string())))
        );

        assert_eq!(3, ctx.len())
    }

    #[test]
    fn test_simple_remove() {
        let mut ctx = IdentifierContext::new();
        let a_identifier = Identifier::new("a".to_string(), 1);
        let b_identifier = Identifier::new("b".to_string(), 2);

        ctx.insert(a_identifier.clone(), Type::Datatype("x".to_string()));
        ctx.insert(b_identifier.clone(), Type::Datatype("y".to_string()));

        assert_eq!(
            ctx.remove(&a_identifier),
            Some(Type::Datatype("x".to_string()))
        );

        assert_eq!(ctx.remove(&a_identifier), None);

        assert_eq!(
            ctx.get(&b_identifier),
            Some(&Type::Datatype("y".to_string()))
        );

        assert_eq!(1, ctx.len())
    }

    #[test]
    fn test_simple_remove_by_name() {
        let mut ctx = IdentifierContext::new();
        let a_identifier = Identifier::new("a".to_string(), 1);
        let b_identifier = Identifier::new("b".to_string(), 2);

        ctx.insert(a_identifier.clone(), Type::Datatype("x".to_string()));
        ctx.insert(b_identifier.clone(), Type::Datatype("y".to_string()));

        assert_eq!(
            ctx.remove_by_name(&"a".to_string()),
            Some((a_identifier, Type::Datatype("x".to_string())))
        );

        assert_eq!(ctx.remove_by_name(&"a".to_string()), None);

        assert_eq!(
            ctx.get_by_name(&"b".to_string()),
            Some((&b_identifier, &Type::Datatype("y".to_string())))
        );

        assert_eq!(1, ctx.len())
    }

    #[test]
    fn test_filled_remove() {
        let mut ctx = IdentifierContext::new();

        let a_identifier = Identifier::new("c".to_string(), 1);
        let b_identifier = Identifier::new("b".to_string(), 1);
        let c_identifier = Identifier::new("a".to_string(), 1);

        ctx.insert(a_identifier.clone(), Type::Datatype("x".to_string()));
        ctx.insert(b_identifier.clone(), Type::Datatype("y".to_string()));
        ctx.insert(c_identifier.clone(), Type::Datatype("z".to_string()));

        assert_eq!(
            ctx.remove(&b_identifier),
            Some(Type::Datatype("y".to_string()))
        );

        assert_eq!(ctx.remove(&b_identifier), None);

        assert_eq!(
            ctx.get(&a_identifier),
            Some(&Type::Datatype("x".to_string()))
        );

        assert_eq!(
            ctx.get(&c_identifier),
            Some(&Type::Datatype("z".to_string()))
        );

        assert_eq!(2, ctx.len())
    }

    #[test]
    fn test_filled_remove_by_name() {
        let mut ctx = IdentifierContext::new();

        let c_identifier = Identifier::new("c".to_string(), 1);
        let b_identifier = Identifier::new("b".to_string(), 1);
        let a_identifier = Identifier::new("a".to_string(), 1);

        ctx.insert(a_identifier.clone(), Type::Datatype("x".to_string()));
        ctx.insert(b_identifier.clone(), Type::Datatype("y".to_string()));
        ctx.insert(c_identifier.clone(), Type::Datatype("z".to_string()));

        assert_eq!(
            ctx.remove_by_name(&"b".to_string()),
            Some((b_identifier, Type::Datatype("y".to_string())))
        );

        assert_eq!(ctx.remove_by_name(&"b".to_string()), None);

        assert_eq!(
            ctx.get_by_name(&"a".to_string()),
            Some((&a_identifier, &Type::Datatype("x".to_string())))
        );

        assert_eq!(
            ctx.get_by_name(&"c".to_string()),
            Some((&c_identifier, &Type::Datatype("z".to_string())))
        );

        assert_eq!(2, ctx.len())
    }

    #[test]
    fn test_shadowing_1() {
        let mut ctx = IdentifierContext::new();

        let a_1_identifier = Identifier::new("a".to_string(), 1);
        let a_2_identifier = Identifier::new("a".to_string(), 2);
        let b_identifier = Identifier::new("b".to_string(), 1);
        let a_3_identifier = Identifier::new("a".to_string(), 3);

        ctx.insert(a_1_identifier.clone(), Type::Datatype("w".to_string()));
        ctx.insert(b_identifier.clone(), Type::Datatype("x".to_string()));
        ctx.insert(a_2_identifier.clone(), Type::Datatype("y".to_string()));
        ctx.insert(a_3_identifier.clone(), Type::Datatype("z".to_string()));

        assert_eq!(4, ctx.len());
        assert_eq!(
            ctx.remove_by_name(&"a".to_string()),
            Some((a_3_identifier, Type::Datatype("z".to_string())))
        );

        assert_eq!(3, ctx.len());
        assert_eq!(
            ctx.remove_by_name(&"a".to_string()),
            Some((a_2_identifier, Type::Datatype("y".to_string())))
        );

        assert_eq!(2, ctx.len());
        assert_eq!(
            ctx.remove_by_name(&"a".to_string()),
            Some((a_1_identifier, Type::Datatype("w".to_string())))
        );

        assert_eq!(1, ctx.len());
        assert_eq!(
            ctx.get_by_name(&"b".to_string()),
            Some((&b_identifier, &Type::Datatype("x".to_string())))
        )
    }

    #[test]
    fn test_shadowing_2() {
        let mut ctx = IdentifierContext::new();

        let a_1_identifier = Identifier::new("a".to_string(), 1);
        let a_2_identifier = Identifier::new("a".to_string(), 2);
        let c_identifier = Identifier::new("c".to_string(), 1);
        let b_identifier = Identifier::new("b".to_string(), 1);
        let a_3_identifier = Identifier::new("a".to_string(), 3);

        ctx.insert(a_1_identifier.clone(), Type::Datatype("v".to_string()));
        ctx.insert(a_2_identifier.clone(), Type::Datatype("w".to_string()));
        ctx.insert(c_identifier.clone(), Type::Datatype("x".to_string()));
        ctx.insert(b_identifier.clone(), Type::Datatype("y".to_string()));
        ctx.insert(a_3_identifier.clone(), Type::Datatype("z".to_string()));

        assert_eq!(
            ctx.get_by_name(&"a".to_string()),
            Some((&a_3_identifier, &Type::Datatype("z".to_string())))
        );
        assert_eq!(5, ctx.len());

        assert_eq!(
            ctx.remove_by_name(&"b".to_string()),
            Some((b_identifier, Type::Datatype("y".to_string())))
        );
        assert_eq!(4, ctx.len());

        assert_eq!(
            ctx.get_by_name(&"a".to_string()),
            Some((&a_3_identifier, &Type::Datatype("z".to_string())))
        );
        assert_eq!(4, ctx.len());

        assert_eq!(
            ctx.remove_by_name(&"c".to_string()),
            Some((c_identifier, Type::Datatype("x".to_string())))
        );
        assert_eq!(3, ctx.len());

        assert_eq!(
            ctx.remove_by_name(&"a".to_string()),
            Some((a_3_identifier, Type::Datatype("z".to_string())))
        );
        assert_eq!(2, ctx.len());

        assert_eq!(
            ctx.remove_by_name(&"a".to_string()),
            Some((a_2_identifier, Type::Datatype("w".to_string())))
        );
        assert_eq!(1, ctx.len());

        assert_eq!(
            ctx.remove_by_name(&"a".to_string()),
            Some((a_1_identifier, Type::Datatype("v".to_string())))
        );
        assert_eq!(0, ctx.len());
    }
}
