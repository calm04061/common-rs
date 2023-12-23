pub mod model;
#[cfg(feature = "quartz")]
mod scheduler;

#[cfg(feature = "quartz")]
pub use scheduler::get_scheduler;

mod database;
pub mod dao;
pub mod web;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
