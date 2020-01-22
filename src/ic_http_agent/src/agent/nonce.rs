use crate::Blob;
use rand::Rng;
use std::cell::RefCell;
use std::sync::Mutex;

pub struct NonceFactory {
    inner: Mutex<RefCell<Box<dyn Iterator<Item = Blob> + Send>>>,
}

impl NonceFactory {
    pub fn from_iterator(iter: Box<dyn Iterator<Item = Blob> + Send>) -> Self {
        Self {
            inner: Mutex::new(RefCell::new(iter)),
        }
    }

    pub fn random() -> NonceFactory {
        Self::from_iterator(Box::new(RandomBlobIter {}))
    }

    pub fn empty() -> NonceFactory {
        Self::from_iterator(Box::new(EmptyBlobIter {}))
    }

    pub fn generate(&self) -> Option<Blob> {
        self.inner.lock().unwrap().borrow_mut().next()
    }
}

struct RandomBlobIter {}

impl Iterator for RandomBlobIter {
    type Item = Blob;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Blob(rand::thread_rng().gen::<[u8; 16]>().to_vec()))
    }
}

struct EmptyBlobIter {}

impl Iterator for EmptyBlobIter {
    type Item = Blob;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
