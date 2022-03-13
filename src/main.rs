#[allow(unused)]
use std::io::BufRead;
use std::process::Command;
use tokio::sync::Mutex;

use chrono::Local;
use once_cell::sync::OnceCell;
use tokio_cron_scheduler::{Job, JobScheduler};

use school_library::book_store_info::BookStoreInfo;

static GLOBAL_BOOKSTORE_INFO: OnceCell<Mutex<BookStoreInfo>> = OnceCell::new();

#[tokio::main]
pub async fn main() {
    let mut sched = JobScheduler::new();
    #[cfg(feature = "signal")]
    sched.shutdown_on_ctrl_c();
    GLOBAL_BOOKSTORE_INFO.set(
        Mutex::new(
            BookStoreInfo::new("./src/config.toml").await.expect("Initialization failed!\n1. Please Check If the config.toml is in your current working directory\n2. Make Sure you have stable network connection(Inner Network).\n")
        )
    ).unwrap();

    println!("WelCome to SCL REPL(Rust) v0.5!\nPrint \"help\" for more information");
    sched.add(Job::new_async("0 5/30 7-22 * * *", |_uuid, _l| Box::pin( async {
        let mut bookstore_info = GLOBAL_BOOKSTORE_INFO.get().unwrap().lock().await;
        bookstore_info.refresh().await.unwrap();
    })).unwrap()).unwrap();
    sched.add(Job::new_async("2 0 0 * * *", |_uuid, _l| Box::pin( async {
        let mut bookstore_info = GLOBAL_BOOKSTORE_INFO.get().unwrap().lock().await;
        bookstore_info.refresh().await.unwrap();
        let res = bookstore_info.make_one_seat_every_appointment(None, Some(true)).await.unwrap();
        println!("{:?}", res);
    })).unwrap()).unwrap();

    sched.start();

    loop {
        let mut input = String::new();
        {
            let stdin = std::io::stdin();
            let mut handle = stdin.lock();
            handle.read_line(&mut input).unwrap();
        }
        input = input.strip_suffix('\n').unwrap().to_string();
        let commands = input.split(' ').collect::<Vec<_>>();
        if commands.is_empty() {
            continue;
        }
        match commands[0] {
            "help" => println!(r##"
    [{}]
    help: print this help message
    exit: exit the program
    la  : print school library full list
    ls  : print school library available list
    ap  : print appointments list
    jb  : print background jobs(debug)
    r   : force refresh info(debug)
    cs  : cancel schedule
    sg  : sign(force)
    st  : set current seat
    sn  : schedule now(force)
    s   : schedule next_day
    "##, Local::now().to_rfc3339()),
            "exit" => {
                print!("Bye!");
                break;
            }
            "la" => {
                let bookstore_info = GLOBAL_BOOKSTORE_INFO.get().unwrap().lock().await;
                BookStoreInfo::show_seat_info(&bookstore_info.full_data);
            }
            "ls" => {
                let bookstore_info = GLOBAL_BOOKSTORE_INFO.get().unwrap().lock().await;
                BookStoreInfo::show_seat_info(&bookstore_info.available_data);
            }
            "ap" => {
                let bookstore_info = GLOBAL_BOOKSTORE_INFO.get().unwrap().lock().await;
                println!("{:?}", &bookstore_info.appointment_to_be_signed);
            }
            "r" => {
                let mut bookstore_info = GLOBAL_BOOKSTORE_INFO.get().unwrap().lock().await;
                bookstore_info.refresh().await.unwrap();
            }
            "jb" => {

            }
            "cls" | "clear" => {
                Command::new(commands[0]).spawn().unwrap().wait().unwrap();
            }
            _ => println!("Unknown command\nPrint \"help\" for more information")
        }
    }
    sched.shutdown().unwrap();
}