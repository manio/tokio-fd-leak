use std::thread;
use std::time::Duration;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::task;
use tokio::time::timeout;

async fn worker(devicepath: &str) {
    loop {
        print!("{}: Opening device\n", devicepath);
        let mut options = OpenOptions::new();
        let future = options.read(true).write(true).open(devicepath);
        match timeout(Duration::from_secs(5), future).await {
            Ok(res) => match res {
                Ok(mut file) => {
                    print!("{}: device opened successfully...\n", devicepath);
                    loop {
                        print!("{}: before read...\n", devicepath);
                        let mut buffer = vec![0u8; 1];
                        let retval = file.read_exact(&mut buffer);
                        match timeout(Duration::from_secs_f32(2.5), retval).await {
                            Ok(res) => match res {
                                Ok(n) => {
                                    print!("{}: read {} byte(s)\n", devicepath, n);
                                    thread::sleep(Duration::from_secs(10));
                                }
                                Err(e) => {
                                    print!("{}: file read error: {}\n", devicepath, e);
                                }
                            },
                            Err(e) => {
                                print!("{}: response timeout: {}\n", devicepath, e);
                                break;
                            }
                        }
                        thread::sleep(Duration::from_millis(30));
                    }
                }
                Err(e) => {
                    print!("{}: error opening device: {:?}\n", devicepath, e);
                    thread::sleep(Duration::from_secs(3));
                    continue;
                }
            },
            Err(e) => {
                print!("{}: file open timeout: {}\n", devicepath, e);
            }
        }
        thread::sleep(Duration::from_millis(30));
    }
}

#[tokio::main(worker_threads = 4)]
async fn main() {
    //this task is causing fd leak:
    let _ = task::spawn(async move { worker("/dev/mychar0").await });

    //this task is reading ok until the above task fills the fd limit, then it stops working
    let _ = task::spawn(async move { worker("/dev/zero").await });

    print!("Entering main loop...\n");
    loop {
        thread::sleep(Duration::from_millis(50));
    }
}
