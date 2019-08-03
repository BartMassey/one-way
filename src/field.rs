pub type Loc = char;

pub struct Field(Vec<Loc>);

impl Default for Field {
    fn default() -> Self {
        let field = Vec::new();
        Field(field)
    }
}
