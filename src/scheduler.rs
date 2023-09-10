

use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
#[cfg(feature = "quartz")]
use quartz_sched::Scheduler;
#[cfg(feature = "quartz")]
lazy_static! {
    static ref SCHEDULER_HOLDER: Mutex<SchedulerHolder> = Mutex::new(SchedulerHolder::new());
}
#[cfg(feature = "quartz")]
struct SchedulerHolder {
    object_ref: Option<Arc<Mutex<Scheduler>>>,
}
#[cfg(feature = "quartz")]
impl SchedulerHolder {
    fn new() -> Self {
        SchedulerHolder { object_ref: None }
    }
    fn get(&mut self) -> Arc<Mutex<Scheduler>> {
        match &self.object_ref {
            Some(obj) => { Arc::clone(&obj) }
            None => {
                let sched: Scheduler = init_scheduler();
                let mutex = Arc::new(Mutex::new(sched));
                self.object_ref = Some(Arc::clone(&mutex));
                Arc::clone(&mutex)
            }
        }
    }
}
#[cfg(feature = "quartz")]
fn init_scheduler() -> Scheduler<8> {
    let mut sched: Scheduler = Scheduler::new();
    sched.start();
    return sched;
}
#[cfg(feature = "quartz")]
pub fn get_scheduler() -> Arc<Mutex<Scheduler>> {
    let mut guard = SCHEDULER_HOLDER.lock().unwrap();
    guard.get()
}
