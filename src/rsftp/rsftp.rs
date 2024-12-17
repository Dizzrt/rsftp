use notify::{DebouncedEvent, RecursiveMode, Watcher};
use std::{net::TcpStream, path::Path, sync::mpsc::channel};
use walkdir::WalkDir;

use super::config::SyncPath;

pub fn rsftp(sp: SyncPath) -> Result<(), Box<dyn std::error::Error>> {
    let local_path = Path::new(sp.local_path.as_str());
    let remote_path = Path::new(sp.remote_path.as_str());

    let ssh_addr = format!("{}:{}", sp.host, sp.port);
    let ssh_key_path = Path::new(sp.private_key_path.as_str());

    // connect to the ssh server
    let tcp = TcpStream::connect(ssh_addr)?;
    let mut sess = ssh2::Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    sess.userauth_pubkey_file(sp.username.as_str(), None, ssh_key_path, None)?;

    // create the sftp session
    let sftp = sess.sftp()?;

    if let Err(e) = self::initial_sync(local_path, remote_path, &sftp) {
        eprintln!("Error during initial sync: {}", e);
    }

    // watch for changes
    let (tx, rx) = channel::<DebouncedEvent>();
    let mut watcher = notify::watcher(tx, std::time::Duration::from_secs(1))?;
    watcher.watch(local_path, RecursiveMode::Recursive)?;

    // handle events
    println!("Starting to watch for changes in {}", local_path.display());
    loop {
        match rx.recv() {
            Ok(event) => match event {
                notify::DebouncedEvent::Create(path) | notify::DebouncedEvent::Write(path) => {
                    println!("File created or modified: {:?}", path);

                    let relative_path = path.strip_prefix(local_path)?;
                    let remote_file_path = remote_path.join(relative_path);
                    self::sync_file(&path, &remote_file_path, &sftp)?;
                }
                notify::DebouncedEvent::Remove(path) => {
                    println!("File removed: {:?}", path);

                    let relative_path = path.strip_prefix(local_path)?;
                    let remote_file_path = remote_path.join(relative_path);
                    self::sync_file(&path, &remote_file_path, &sftp)?;
                }
                _ => {}
            },
            Err(e) => {
                eprintln!("Error watching for changes: {}", e);
            }
        }
    }
}

fn initial_sync(
    local_path: &Path,
    remote_path: &Path,
    sftp: &ssh2::Sftp,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in WalkDir::new(local_path) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let relative_path = path.strip_prefix(local_path)?;
            let remote_file_path = remote_path.join(relative_path);
            self::sync_file(path, &remote_file_path, sftp)?;
        }
    }

    Ok(())
}

fn sync_file(
    local_file: &Path,
    remote_file: &Path,
    sftp: &ssh2::Sftp,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut remote_file = sftp.create(remote_file)?;
    let mut local_file = std::fs::File::open(local_file)?;
    std::io::copy(&mut local_file, &mut remote_file)?;

    Ok(())
}
