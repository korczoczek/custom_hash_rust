use std::{io::{self, Write}, time::Duration};
use sha256::{digest};
use std::sync::{Arc, Mutex};
use std::thread;
use clap::{Parser, ValueEnum};
use ctrlc;
use std::process;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode{
    
    /// Check for the key at the start
    Start,

    /// Check for the key throughout the hash
    Scatter,

    /// Check for the biggest uninterrupted chunk of the key
    Chunk
}

#[derive(Parser)]
#[command(version, name = "custom_hash")]
#[command(about = "Derives custom sha256 hashes", long_about = None)]
struct Cli {
    ///Starting content of the message
    #[arg(default_value_t = String::from(""))]
    message: String,

    ///Key to be found in the resulting hash
    #[arg(default_value_t = String::from("0"))]
    key: String,

    ///Starting index of the search in base62
    #[arg(default_value_t = String::from("0"))]
    index: String,
    
    ///Checking mode
    #[arg(value_enum, short, long, default_value_t = Mode::Start)]
    mode: Mode,

    ///Find as many examples at the current depth
    #[arg(short, long)]
    all: bool,

    ///Starting count
    #[arg(short, long, default_value_t = 1)]
    count: usize,
}




fn int_to_base_x(num:usize,list:&String) -> String {
    
    let mut base_x: String = String::from("");
    
    let mut num: usize = num;

    if num == 0 {
        base_x = String::from("0");
    }else{
        let mut remainder:usize;
        let len: usize = list.chars().count();
        let chars: Vec<char> = list.chars().collect();
        while num > 0{
            remainder = num % len;
            base_x = format!("{}{}", chars[remainder], base_x);
            num -= remainder;
            num /= len;
        }
    }
    return base_x;
}

fn base_x_to_int(base_x:&String, list:&String) -> usize {
    let mut val:usize = 0;
    let mut mult:usize = 1;
    let list_len:usize = list.chars().count();

    for i in base_x.chars().rev(){
        let digit: Option<usize> = list.find(i);
        let digit: usize = digit.unwrap();

        val += digit*mult;
        mult *= list_len;
    }

    return val;
}

fn start_count(search: &String, find: &String) -> usize{
    if search.is_empty() || find.is_empty() {
        return 0;
    }
    
    let mut count:usize = 0;
    let chars:Vec<char> = find.chars().collect();
    let len: usize = find.chars().count();

    for (i, c) in search.chars().enumerate(){
        if c != chars[i % len]{
            break;
        }
        count += 1;
    }
    
    return count;
}

fn scatter_count(search: &String, find: &String) -> usize {
    if search.is_empty() || find.is_empty() {
        return 0;
    }
    
    let mut count:usize = 0;

    let search_iter: std::str::Chars<'_> = search.chars();
    let find_iter: std::str::Chars<'_> = find.chars();

    let search_chars:Vec<char> = search_iter.clone().collect();
    let find_chars:Vec<char> = find_iter.clone().collect();

    let search_len:usize = search_iter.count();
    let find_len:usize = find_iter.count();

    if search_len >= find_len{
        for i in 0..(search_len - (find_len-1)){
            if search_chars[i] == find_chars[0]{
                let check:String = search_chars[i..i+find_len].iter().cloned().collect::<String>();
                if start_count(&check, &find) == find_len{
                    count += 1;
                }
            }
        }
    }

    return count;
}

fn chunk_count(search: &String, find: &String) -> usize {
    if search.is_empty() || find.is_empty() {
        return 0;
    }
    let mut count:usize = 0;
    let mut max_count:usize = 0;
    let mut char_pos:usize = 0;

    let search_iter: std::str::Chars<'_> = search.chars();
    let find_iter: std::str::Chars<'_> = find.chars();

    let search_chars:Vec<char> = search_iter.clone().collect();
    let find_chars:Vec<char> = find_iter.clone().collect();

    //let search_len:usize = search_iter.count();
    let find_len:usize = find_iter.count();

    for letter in search_chars{
        if letter == find_chars[char_pos]{
            count += 1;
            char_pos = count % find_len;
        }else{
            if count > max_count{
                max_count = count;
            }
            count = 0;
            char_pos = 0;
        }
    }

    if count > max_count{
        return count;
    }

    return max_count;

}

fn is_hex(text:&String) -> bool{
    let hex: String = String::from("0123456789abcdef");

    for i in text.chars(){
        if !hex.contains(i){
            return false;
        }
    }

    return true;
}

fn main() {
    let cli = Cli::parse();
    
    let hex_list: String = String::from("0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz");

    let message = cli.message;
    let key: String = cli.key;
    let start_max_count = cli.count;

    if !is_hex(&key){
        println!("Key \"{}\" is not a valid hex number", &key);
        return;
    }

    let sep: String = if message == String::from(""){
        String::from("")
    } else {
        String::from(" ")
    };

    const THREADS:usize = 8;
    const THREAD_SIZE:usize = 2_usize.pow(17); //131 072

    let idx = Arc::new(Mutex::new(base_x_to_int(&cli.index, &hex_list))); 
    let max_count = Arc::new(Mutex::new(start_max_count.clone()));
    //let mut handles = vec![];
    let print_mutex = Arc::new(Mutex::new(false));



    let idx_clone = Arc::clone(&idx);
    let print_clone = Arc::clone(&print_mutex);
    let hex_list_clone = hex_list.clone();
    let message_clone = message.clone();
    let sep_clone = sep.clone();
    let _ = ctrlc::set_handler(move || {
        let idx = Arc::clone(&idx_clone);
        let print_mutex = Arc::clone(&print_clone);

        let hex_list = &hex_list_clone;
        let message = &message_clone;
        let sep = &sep_clone;

        let idx_lock = idx.lock().unwrap();
        let idx = *idx_lock;
        
        let _print_lock = print_mutex.lock().unwrap();
        print!("\r                                                       ");
        println!("\r{}{}{}",message, sep, int_to_base_x(idx, hex_list));

        process::exit(0);
    });



    for _ in 0..THREADS {
        let idx = Arc::clone(&idx);
        let max_count = Arc::clone(&max_count);
        let print_mutex = Arc::clone(&print_mutex);

        let hex_list = hex_list.clone();
        let message = message.clone();
        let sep = sep.clone();
        let key = key.clone();
        //let key_chars:Vec<char> = key.clone().chars().collect();
        let mode = cli.mode;

        thread::spawn(move ||{
            let mut count;
            let mut local_max_count = start_max_count.clone();
            loop{
                let mut idx_lock = idx.lock().unwrap();
                let size = *idx_lock;

                *idx_lock = size + THREAD_SIZE;

                std::mem::drop(idx_lock);

                for i in size..(size + THREAD_SIZE) {

                    let hex_init: String = format!("{}{}{}", &message, &sep, int_to_base_x(i, &hex_list));

                    let hex: String = digest(&hex_init);

                    count = match mode {
                        Mode::Start => start_count(&hex, &key),
                        Mode::Scatter => scatter_count(&hex, &key),
                        Mode::Chunk => chunk_count(&hex, &key)
                    };

                    if count >= local_max_count{
                        let mut max_count_lock = max_count.lock().unwrap();
                        local_max_count = *max_count_lock;
                        
                        if count >= local_max_count{
                            *max_count_lock = count;
                            let print_lock = print_mutex.lock().unwrap();
                            match mode {
                                Mode::Start => println!("\rFound sha256 hash starting with \"{}\" ({} characters):", &hex[0..count], count),
                                Mode::Scatter => println!("\rFound sha256 hash with {} instances of the \"{}\" key", count, &key),
                                Mode::Chunk => println!("\rFound sha256 hash with a {} character long chunk of \"{}\"", count, &key)
                            }
                            
                            println!("{}",&hex_init);
                            println!("{}",&hex);
                            std::mem::drop(print_lock);
                        }
                        std::mem::drop(max_count_lock);
                    }
                }

                // if i%100000 == 0 && now.elapsed() > Duration::from_secs(1){
                //     print!("\r{} ({}/s)", &hex_init, i - prev_i);
                //     now = Instant::now();
                //     prev_i = i;
                //     let _ = io::stdout().flush();
                // }
            }
        });
    }

    let handle = thread::spawn(move || {
        let idx = Arc::clone(&idx);
        let print_mutex = Arc::clone(&print_mutex);
        let hex_list = hex_list.clone();

        let message = message.clone();
        let sep = sep.clone();
        let mut prev_idx:usize = 0;
        loop{
            thread::sleep(Duration::from_secs(1));
            let idx_lock = idx.lock().unwrap();
            let print_lock = print_mutex.lock().unwrap();
            print!("\r{}{}{} ({}/s)     ", &message, &sep, int_to_base_x(*idx_lock, &hex_list), *idx_lock - prev_idx);
            let _ = io::stdout().flush();
            prev_idx = *idx_lock;
            std::mem::drop(idx_lock);
            std::mem::drop(print_lock);
        }
    });

    handle.join().unwrap();
}
