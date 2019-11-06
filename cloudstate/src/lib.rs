#[macro_use]
extern crate log;
extern crate log4rs;

pub mod protocol;
pub mod serveless;
pub mod handlers;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}