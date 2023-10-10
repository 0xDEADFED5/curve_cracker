use std::{fs::{self}, io::{BufReader, BufRead}, sync::atomic::{AtomicBool, Ordering, AtomicU64}, time, env};
use std::io::prelude::*;
use std::fs::OpenOptions;
#[macro_use(concat_string)]
extern crate concat_string;
use sha1::{Sha1, Digest, digest::{generic_array::GenericArray, typenum::U20}};
use arrayvec::ArrayString;
use std::thread;
// max string length, adjust as needed
const LEN: usize = 64;
// largest number to append to strings
const MAX_COUNT: i16 = 3333;
// adjust as needed, 3 minimum because i'm lazy
const MAX_THREADS: usize = 17;
const ABOOL: AtomicBool = AtomicBool::new(false);
const AZERO: AtomicU64 = AtomicU64::new(0);
static mut FINISHED: [AtomicBool; MAX_THREADS - 1] = [ABOOL; MAX_THREADS - 1];
static mut TOTAL: [AtomicU64; MAX_THREADS - 1] = [AZERO; MAX_THREADS - 1];

#[inline]
fn check_result(input: &GenericArray<u8,U20>, nist: &[[u8;20];NIST_LEN]) -> bool {
    let mut found = false;
    for h in nist {
        for x in 0..20 {
            if h[x] == input[x]
            {
                found = true;
            }
            else {
                found = false;
                break;
            }
        }
        if found {
            break;
        }
    }
    found
}
// this has to match max number of strings generated in mutate_string() below
const MUT_LEN: usize = 24;
#[inline]
fn mutate_string(input: &str, counter: i16, output: &mut [ArrayString<LEN>;MUT_LEN * STRING_VARIATIONS], index: usize) {
    // example: Jerry and Satan deserve a raise666.
    if counter != -1 {
        // ascii numbers
        let count = counter.to_string();
        let tmp = concat_string!(input, count);
        output[index * MUT_LEN + 0].push_str(&tmp);
        output[index * MUT_LEN + 1].push_str(&tmp);
        output[index * MUT_LEN + 1].make_ascii_lowercase();
        output[index * MUT_LEN + 2].push_str(&tmp);
        output[index * MUT_LEN + 2].make_ascii_uppercase();
        // i don't like this one
        let tmp = concat_string!(input, count, ".");
        output[index * MUT_LEN + 3].push_str(&tmp);
        output[index * MUT_LEN + 4].push_str(&tmp);
        output[index * MUT_LEN + 4].make_ascii_lowercase();
        output[index * MUT_LEN + 5].push_str(&tmp);
        output[index * MUT_LEN + 5].make_ascii_uppercase();
        let tmp = concat_string!(input, ".", count);
        output[index * MUT_LEN + 6].push_str(&tmp);
        output[index * MUT_LEN + 7].push_str(&tmp);
        output[index * MUT_LEN + 7].make_ascii_lowercase();
        output[index * MUT_LEN + 8].push_str(&tmp);
        output[index * MUT_LEN + 8].make_ascii_uppercase();
        let tmp = concat_string!(input, "!", count);
        output[index * MUT_LEN + 9].push_str(&tmp);
        output[index * MUT_LEN + 10].push_str(&tmp);
        output[index * MUT_LEN + 10].make_ascii_lowercase();
        output[index * MUT_LEN + 11].push_str(&tmp);
        output[index * MUT_LEN + 11].make_ascii_uppercase();
        // binary numbers
        let bin = counter.to_le_bytes();
        let sbin = unsafe { core::str::from_utf8_unchecked(&bin) };
        let tmp = concat_string!(input, sbin);
        output[index * MUT_LEN + 12].push_str(&tmp);
        output[index * MUT_LEN + 13].push_str(&tmp);
        output[index * MUT_LEN + 13].make_ascii_lowercase();
        output[index * MUT_LEN + 14].push_str(&tmp);
        output[index * MUT_LEN + 14].make_ascii_uppercase();
        // still don't like this one
        let tmp = concat_string!(input, sbin, ".");
        output[index * MUT_LEN + 15].push_str(&tmp);
        output[index * MUT_LEN + 16].push_str(&tmp);
        output[index * MUT_LEN + 16].make_ascii_lowercase();
        output[index * MUT_LEN + 17].push_str(&tmp);
        output[index * MUT_LEN + 17].make_ascii_uppercase();
        let tmp = concat_string!(input, "!", sbin);
        output[index * MUT_LEN + 18].push_str(&tmp);
        output[index * MUT_LEN + 19].push_str(&tmp);
        output[index * MUT_LEN + 19].make_ascii_lowercase();
        output[index * MUT_LEN + 20].push_str(&tmp);
        output[index * MUT_LEN + 20].make_ascii_uppercase();
        let tmp = concat_string!(input, ".", sbin);
        output[index * MUT_LEN + 21].push_str(&tmp);
        output[index * MUT_LEN + 22].push_str(&tmp);
        output[index * MUT_LEN + 22].make_ascii_lowercase();
        output[index * MUT_LEN + 23].push_str(&tmp);
        output[index * MUT_LEN + 23].make_ascii_uppercase();
    }
    // -1 means don't append a counter
    // example: Jerry and Satan deserve a raise.
    else {
        output[index * MUT_LEN + 0].push_str(&input);
        output[index * MUT_LEN + 1].push_str(&input);
        output[index * MUT_LEN + 1].make_ascii_lowercase();
        output[index * MUT_LEN + 2].push_str(&input);
        output[index * MUT_LEN + 2].make_ascii_uppercase();
        let tmp = concat_string!(input, ".");
        output[index * MUT_LEN + 3].push_str(&tmp);
        output[index * MUT_LEN + 4].push_str(&tmp);
        output[index * MUT_LEN + 4].make_ascii_lowercase();
        output[index * MUT_LEN + 5].push_str(&tmp);
        output[index * MUT_LEN + 5].make_ascii_uppercase();
        let tmp = concat_string!(input, "!");
        output[index * MUT_LEN + 6].push_str(&tmp);
        output[index * MUT_LEN + 7].push_str(&tmp);
        output[index * MUT_LEN + 7].make_ascii_lowercase();
        output[index * MUT_LEN + 8].push_str(&tmp);
        output[index * MUT_LEN + 8].make_ascii_uppercase();
    }
}

// has to match the number of string variations in get_strings() below
const STRING_VARIATIONS: usize = 16;
#[inline]
fn get_strings(counter: i16, name: &str, mut output: &mut [ArrayString<LEN>;MUT_LEN * STRING_VARIATIONS]) {
    let mut index = 0;
    let s = concat_string!("Jerry and ", name, " deserve a raise"); // 1
    mutate_string(&s, counter, &mut output, index);
    index += 1;
    let s = concat_string!(name, " and Jerry deserve a raise"); // 2
    mutate_string(&s, counter, &mut output, index);
    index += 1;
    let s = "Jerry deserves a raise"; // 3
    mutate_string(&s, counter, &mut output, index);
    index += 1;
    let s = "Jerry deserves a break"; // 4
    mutate_string(&s, counter, &mut output, index);
    index += 1;
    let s = "Jerry needs a coffee"; // 5
    mutate_string(&s, counter, &mut output, index);
    index += 1;
    let s = concat_string!("Jerry and ", name, " deserve raises"); // 6
    mutate_string(&s, counter, &mut output, index);
    index += 1;
    let s = concat_string!(name, " and Jerry deserve raises"); // 7
    mutate_string(&s, counter, &mut output, index);
    index += 1;
    let s = concat_string!("Jerry and ", name, " deserve promotions"); // 8
    mutate_string(&s, counter, &mut output, index);
    index += 1;
    let s = concat_string!(name, " and Jerry deserve promotions"); // 9
    mutate_string(&s, counter, &mut output, index);
    index += 1;
    let s = concat_string!("Jerry and ", name, " deserve a promotion"); // 10
    mutate_string(&s, counter, &mut output, index);
    index += 1;
    let s = concat_string!(name, " and Jerry deserve a promotion"); // 11
    mutate_string(&s, counter, &mut output, index);
    index += 1;
    let s = concat_string!("Give Jerry and ", name, " a raise"); // 12
    mutate_string(&s, counter, &mut output, index);
    index += 1;
    let s = concat_string!("Give ", name, " and Jerry a raise"); // 13
    mutate_string(&s, counter, &mut output, index);
    index += 1;
    let s = concat_string!("Give ", name, " and Jerry raises"); // 14
    mutate_string(&s, counter, &mut output, index);
    index += 1;
    let s = concat_string!("Give Jerry and ", name, " raises"); // 15
    mutate_string(&s, counter, &mut output, index);
    index += 1;
    let s = "Give Jerry a raise"; // 16
    mutate_string(&s, counter, &mut output, index);
}
#[inline]
fn log_found(msg: &[u8], path: &str) {
    let mut f = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .unwrap();
    // prints fucked up UTF-8 to console, saves actual bytes to log
    // might have to view log file with a hex editor
    println!("FOUND: '{}'", String::from_utf8_lossy(msg));
    f.write(msg).expect("error writing file...");
    f.write("\n".as_bytes()).expect("error writing file...");
}

// update this if length of nist array below is modified
const NIST_LEN: usize = 5;
#[inline]
fn worker(id: usize, names: &[String], start: i16, end: i16, log: &str) {
    let nist: [[u8;20];NIST_LEN] = [
    //[1,125,225,238,87,121,14,202,36,38,70,93,194,152,145,244,32,50,251,70], // Jerry and Satan deserve a raise'0x05''0x0D'.
    //[38,232,14,202,25,250,218,187,145,59,48,16,185,82,241,75,189,87,53,117], // Jerry and Satan deserve a raise.3333
    [48,69,174,111,200,66,47,100,237,87,149,40,211,129,32,234,225,33,150,213], // # NIST P-192, ANSI prime192v1
    [189,113,52,71,153,213,199,252,220,69,181,159,163,185,171,143,106,148,139,197], // # NIST P-224
    [196,157,54,8,134,231,4,147,106,102,120,225,19,157,38,183,129,159,126,144], // # NIST P-256, ANSI prime256v1
    [163,53,146,106,163,25,162,122,29,0,137,106,103,115,164,130,122,205,172,115], // # NIST P-384
    [208,158,136,0,41,28,184,83,150,204,103,23,57,50,132,170,160,218,100,186], // # NIST P-521
    ];
    let mut output: [ArrayString<LEN>;MUT_LEN * STRING_VARIATIONS] = [ArrayString::<LEN>::new();MUT_LEN * STRING_VARIATIONS];
    let mut digest;
    let mut total = 0;
    let mut update_counter = 0;
    for n in names {
        for x in start..=end {
            get_strings(x, n, &mut output);
            for y in 0..output.len() {
                digest = Sha1::digest(output[y].as_bytes());
                if check_result(&digest, &nist) {
                   log_found(output[y].as_bytes(), log);
                }
                output[y].clear();
            }
            // update infos every 100,000
            if update_counter > 100_000 {
                unsafe { TOTAL[id].store(total, Ordering::Relaxed) };
                update_counter = 0;
            }
            total += (MUT_LEN * STRING_VARIATIONS) as u64;
            update_counter += MUT_LEN * STRING_VARIATIONS;
        } 
    }
    unsafe { TOTAL[id].store(total, Ordering::Release) };
    unsafe { FINISHED[id].store(true, Ordering::Release) };
}

#[inline]
fn get_total() -> u64 {
    let mut result = 0;
    for x in 0..MAX_THREADS - 1 {
        unsafe { result += TOTAL[x].load(Ordering::Relaxed); }
    }
    result
}

#[inline]
fn threads_done() -> bool {
    let mut result = false;
    for x in 0..MAX_THREADS - 1 {
        unsafe { result = FINISHED[x].load(Ordering::Acquire); }
        if !result {
            break;
        }
    }
    result
}

fn watcher(total_hashes: u64) {
    let mut total;
    let mut last_total: u64 = 0;
    loop {
        thread::sleep(time::Duration::from_secs(1));
        if threads_done() {
            break;
        }
        total = get_total();
        println!("{} hashes per second, {:.0}% done", total - last_total, (total as f32 / total_hashes as f32) * 100.0);
        last_total = total;
    }
    total = get_total();
    println!("Finished.\n{} total hashes checked.", total);
}
fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 && args.len() != 3 {
        println!("Usage: {} wordlist logfile(optional)", args[0]);
        println!("Matches are printed to console and saved to output.log by default");
        println!("Example: {} names.txt", args[0]);
        return;
    }
    let mut log = "output.log";
    if args.len() == 3 {
        log = &args[2];
    }
    let file = fs::File::open(&args[1]).unwrap();
    let reader = BufReader::new(file);
    let mut names: Vec<String> = vec![];
    for line in reader.lines() {
        if let Ok(l) = line {
            names.push(l);
        } else if let Err(e) = line {
            println!("error reading wordlist: {}", e.to_string());
        }
    }
    println!("{} names loaded from wordlist", names.len());
    println!{"{} threads to be spun up...", MAX_THREADS};
    let mut start: i16 = -1;
    // assign each thread a number range
    // (thread id, start number, end number)
    let mut thread_info: Vec<(usize, i16, i16)> = vec![];
    for x in 0..MAX_THREADS {
        let mut c = MAX_COUNT / (MAX_THREADS - 1) as i16;
        if x == MAX_THREADS - 2 {
            c = MAX_COUNT - start;
        }
        thread_info.push((x, start, start + c));
        start += c + 1;
    }
    let total_hashes = MUT_LEN as u64 * STRING_VARIATIONS as u64 * (MAX_COUNT + 2) as u64 * names.len() as u64;
    thread::scope(|s| {
        for x in thread_info.iter() {
            if x.0 == MAX_THREADS - 1 {
                let builder = thread::Builder::new();
                let res = builder.spawn_scoped(s, || watcher(total_hashes) );
                if res.is_err() {
                    println!("error spinning up watcher: {}", res.err().unwrap());
                }
            }
            else {
                let builder = thread::Builder::new();
                let res = builder.spawn_scoped(s, || worker(x.0, &names, x.1, x.2, log) );
                if res.is_err() {
                    println!("error spinning up worker: {}", res.err().unwrap());
                }
            }
        }
    });
}
