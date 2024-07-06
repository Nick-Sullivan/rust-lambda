pub trait INameDatabase {
    fn increment(&mut self, name: &str);
    fn get_count(&self, name: &str) -> i32;
    fn clear(&mut self, name: &str);
}
