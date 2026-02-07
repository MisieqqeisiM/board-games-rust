use std::io::Result;

pub trait StateBuilder {
    fn load_state(&mut self, version: u64, data: Vec<u8>) -> Result<()>;
    fn load_event(&mut self, version: u64, data: Vec<u8>) -> Result<()>;
}

pub trait Store {
    fn apply_event(&mut self, data: &[u8]) -> impl Future<Output = Result<()>>;
    fn snapshot(&mut self, data: &[u8]) -> impl Future<Output = Result<()>>;
}
