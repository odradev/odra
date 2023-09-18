use casper_types::bytesrepr::ToBytes;

#[derive(Clone)]
struct Path(Vec<u8>);

impl Path {
    pub fn as_vec(&self) -> &Vec<u8> {
        &self.0
    }
}

impl<T: ToBytes> From<T> for Path {
    fn from(key: T) -> Self {
        Path(key.to_bytes().unwrap())
    }
}


#[derive(Clone)]
pub struct PathStack {
    keys: Vec<Path>,
}

impl Default for PathStack {
    fn default() -> Self {
        Self::new()
    }
}

impl PathStack {
    pub fn new() -> PathStack {
        PathStack { keys: Vec::new() }
    }

    pub fn push<T: ToBytes>(&mut self, key: T) {
        self.keys.push(key.into());
    }

    pub fn pop(&mut self) {
        self.keys.pop().unwrap();
    }

    pub fn get_key<T: ToBytes>(&self, suffix: Option<T>) -> Vec<u8> {
        let mut result = Vec::new();
        for key in &self.keys {
            result.extend_from_slice(key.as_vec());
            result.push(b'#');
        }
        if let Some(appendix) = suffix {
            let key: Path = appendix.into();
            result.extend_from_slice(key.as_vec());
        }
        result
    }
}
