#[derive(Debug, PartialEq)]
pub struct Deck<'a> {
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Deck<'a> {
    pub fn new() -> Self {
        Self {
            phantom: std::marker::PhantomData,
        }
    }
}
