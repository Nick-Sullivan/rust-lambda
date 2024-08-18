use domain::errors::LogicError;

#[derive(Clone)]
pub struct NameCount {
    pub name: String,
    pub count: i32,
    pub version: i32,
}

impl NameCount {
    pub fn new(name: &str) -> Self {
        NameCount {
            name: name.to_string(),
            count: 0,
            version: 0,
        }
    }
}

#[trait_variant::make(HttpService: Send)]
pub trait INameDatabase {
    async fn get(&self, name: &str) -> Result<NameCount, LogicError>;
    async fn save(&mut self, item: &NameCount) -> Result<(), LogicError>;
    async fn clear(&mut self, name: &str) -> Result<(), LogicError>;
}
