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
mod login;

pub fn run() {
    let mut core = Core::new().unwrap();
    let input = "https://jiradc.int.net.nokia.com/rest/api/2/filter/145359";
    let login = login::Login::new().to_basic();
    let fut = fetch::fetch(&mut core, input, login);

    //schedule and run
    if let Err(_err) = core.run(fut) {
        error!("Something wrong here!");
    } else {
        info!("Completed now!");
    }
}
