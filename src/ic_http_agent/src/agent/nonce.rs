use crate::Blob;
use rand::rngs::OsRng;
use rand::Rng;
use std::sync::Mutex;

pub struct NonceFactory {
    inner: Mutex<Box<dyn Iterator<Item = Blob> + Send>>,
}

impl NonceFactory {
    pub fn from_iterator(iter: Box<dyn Iterator<Item = Blob> + Send>) -> Self {
        Self {
            inner: Mutex::new(iter),
        }
    }

    pub fn random() -> NonceFactory {
        Self::from_iterator(Box::new(RandomBlobIter {}))
    }

    pub fn empty() -> NonceFactory {
        Self::from_iterator(Box::new(EmptyBlobIter {}))
    }

    pub fn generate(&self) -> Option<Blob> {
        self.inner.lock().unwrap().next()
    }
}

struct RandomBlobIter {}

impl Iterator for RandomBlobIter {
    type Item = Blob;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Blob(OsRng.gen::<[u8; 16]>().to_vec()))
    }
}

struct EmptyBlobIter {}

impl Iterator for EmptyBlobIter {
    type Item = Blob;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
