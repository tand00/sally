use std::{collections::HashMap, thread::ThreadId};

pub struct ExecutionCache<T : Send + Sync> {
    pub thread_storage : HashMap<ThreadId, T>
}

impl<T : Send + Sync> ExecutionCache<T> {

    fn thread_id() -> ThreadId {
        std::thread::current().id()
    }

    pub fn get(&self) -> &T {
        &self.thread_storage[&Self::thread_id()]
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.thread_storage.get_mut(&Self::thread_id()).unwrap()
    }

    pub fn get_or_else<F>(&mut self, f : F) -> T 
        where F : Fn() -> T
    {
        let index = Self::thread_id();
        self.thread_storage.remove(&index).unwrap_or_else(f)
    }

    pub fn set(&mut self, value : T) {
        self.thread_storage.insert(std::thread::current().id(), value);
    }

    pub fn clear_thread_cache(&mut self) -> Option<T> {
        self.thread_storage.remove(&std::thread::current().id())
    }

    pub fn clear(&mut self) {
        self.thread_storage.clear();
    }

}