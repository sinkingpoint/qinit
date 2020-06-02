#[derive(Debug)]
/// An enum representing either a numerical ID or a name, e.g. a username or a uid
pub enum Identifier {
    Name(String),
    ID(u64)
}