pub mod model;
#[cfg(feature = "quartz")]
mod scheduler;
#[cfg(feature = "quartz")]
pub use scheduler::get_scheduler;
#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "sqlite")]
pub mod sqlite;
mod database;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
