use std::thread;

mod rsftp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let c = rsftp::config::load_config().unwrap();

    let mut handles = vec![];
    for sp in c.paths {
        let handle = thread::spawn(move || {
            let name = sp.name.clone();
            if let Err(e) = rsftp::rsftp(sp) {
                eprintln!("Error in rsftp for {}: {}", name, e);
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}
