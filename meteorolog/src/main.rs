mod rtl;

use std::sync::mpsc;

fn main() {
    let (tx, rx) = mpsc::channel();

    let join_handle = rtl::start(tx);

    for message in rx {
        // Process the received RTL-433 messages
        println!("Received message: {:?}", message);
    }

    join_handle.join().expect("RTL-433 thread panicked");
}
