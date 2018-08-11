#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;

#[cfg(test)]
mod test {
    use ::demo::*;
    #[test]
    fn typed_example_test() {
        test_typed();
    }
}

extern crate hyper;
extern crate tokio_core;
use self::tokio_core::reactor::Core;

mod fetch;
pub fn run() {
    let mut core = Core::new().unwrap();
    let fut = fetch::fetch(&mut core);
    if let Err(_err) = core.run(fut) {
        error!("Something wrong here!");
    }
}
