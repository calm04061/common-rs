use std::sync::{Arc, Mutex};
use log::info;

pub(crate) trait CommonConnectionHolder<T> {
    fn get_object_ref(&self) -> Option<Arc<Mutex<T>>>;
    fn set_object_ref(&mut self, obj_ref: Option<Arc<Mutex<T>>>);
    fn get(&mut self) -> Arc<Mutex<T>> {
        match &self.get_object_ref() {
            Some(obj) => { Arc::clone(&obj) }
            None => {
                info!("init connection pool");
                let postgres_pool = Self::init_connection();
                info!("init connection pool finish");

                let mutex = Arc::new(Mutex::new(postgres_pool));
                self.set_object_ref(Some(Arc::clone(&mutex)));
                Arc::clone(&mutex)
            }
        }
    }
    fn init_connection() -> T;
}
