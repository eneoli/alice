use self::prop::Prop;

pub mod check;
pub mod parse;
pub mod process;
pub mod proof;
pub mod proof_term;
pub mod prop;
pub mod prove;

#[derive(Clone, Debug)]
pub struct Sequent {
    goal: Prop,
    inv_ctx: Vec<Prop>,
    non_inv_ctx: Vec<Prop>,
}

impl Sequent {
    pub fn new(goal: &Prop) -> Self {
        Sequent {
            goal: goal.clone(),
            inv_ctx: Vec::new(),
            non_inv_ctx: Vec::new(),
        }
    }

    pub fn change_goal(&mut self, new_goal: &Prop) -> &mut Self {
        self.goal = new_goal.clone();

        self
    }

    pub fn with_new_goal(&self, new_goal: &Prop) -> Self {
        let mut seq = self.clone();
        seq.goal = new_goal.clone();

        seq
    }

    pub fn push_inv(&mut self, prop: &Prop) -> &mut Self {
        self.inv_ctx.push(prop.clone());

        self
    }

    pub fn pop_inv(&mut self) -> Option<Prop> {
        self.inv_ctx.pop()
    }

    pub fn add_non_inv(&mut self, prop: &Prop) -> &mut Self {
        self.non_inv_ctx.push(prop.clone());

        self
    }

    pub fn ctx_contains(&self, prop: &Prop) -> bool {
        self.inv_ctx.contains(prop) || self.non_inv_ctx.contains(prop)
    }
}