use spin_sdk::key_value::Store;

pub(crate) trait MazeStateStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()>;
    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()>;
}

impl MazeStateStore for Store {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
        Store::get(self, key).map_err(|_| ())
    }

    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
        Store::set(self, key, value).map_err(|_| ())
    }
}
