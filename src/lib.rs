pub mod model;
#[cfg(feature = "quartz")]
mod scheduler;

#[cfg(feature = "quartz")]
pub use scheduler::get_scheduler;

#[cfg(feature = "sqlite")]
mod database;
pub mod dao;
#[cfg(feature = "web")]
pub mod web;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
