use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::*;
use std::io;
use std::path::*;
use std::sync::{mpsc, Arc};
use std::thread;

fn main() {
    let mut input = String::with_capacity(100);
    println!("This Searcher simply uses frequncy of words and their synonyms to find the file. It is not perfect");
    println!("Enter the directory you want to search : ");
    io::stdin().read_line(&mut input).unwrap();
    let path = input.clone();
    let path = path.trim();
    let path = Path::new(path);
    input.clear();
    println!("Enter the keywords you want to seach with : ");
    io::stdin().read_line(&mut input).unwrap();
    let keywords = input
        .trim()
        .split_ascii_whitespace()
        .map(|x| x.trim().to_ascii_lowercase())
        .collect::<HashSet<String>>();
    let files = read_dir(path).unwrap();
    let txt_extension = OsStr::new("txt");

    //this will be shared between threads
    let ignore_word_set = fill_set();
    let ignore_word_set = Arc::new(ignore_word_set);

    let mut title_keyword_map = HashMap::with_capacity(100);

    let (tx, rx) = mpsc::channel();

    for file in files {
        let dir_entry_path = file.unwrap().path();
        let file_extension = dir_entry_path.extension();

        if file_extension == Some(txt_extension) {
            let tx1 = tx.clone();
            let ignore_word_set_clone = ignore_word_set.clone();
            thread::spawn(move || {
                let file_name = dir_entry_path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();
                let content = read_to_string(dir_entry_path).unwrap();
                let keyword_set = top_keywords(content, &ignore_word_set_clone);
                tx1.send((file_name, keyword_set)).unwrap();
            });
        }
    }

    //dropping the extra tx
    drop(tx);

    for rec in rx {
        title_keyword_map.insert(rec.0, rec.1);
    }

    let found_files = search_keyword(title_keyword_map, keywords);

    if found_files.is_empty() {
        println!("Sorry no files found matching the keywords...");
    } else {
        println!("{found_files:?}");
    }

    println!("press any key to exit : ");
    io::stdin().read_line(&mut input).unwrap();
}

fn search_keyword(
    file_keyword: HashMap<String, HashSet<String>>,
    mut keywords: HashSet<String>,
) -> HashSet<String> {
    let mut set = HashSet::with_capacity(100);
    add_related_words(&mut keywords);
    for (key, val) in &file_keyword {
        if keywords.contains(key) {
            set.insert(key.to_owned());
            continue;
        }
        for i in &keywords {
            if val.contains(i) {
                set.insert(key.to_string());
                break;
            }
        }
    }
    set
}

fn add_related_words(keywords: &mut HashSet<String>) {
    let mut related_words = HashSet::with_capacity(100);
    for i in keywords.iter() {
        let v = thesaurus::synonyms(i);
        v.into_iter().for_each(|x| {
            related_words.insert(x);
        });
    }
    related_words.into_iter().for_each(|x| {
        keywords.insert(x);
    });
}

// Returns a set of top 50 most frequent words in text file.
//It ignore articles like {this, a, is ...}
//Time Complexity Could be improved
fn top_keywords(content: String, ignore_word_set: &HashSet<String>) -> HashSet<String> {
    let mut set = HashSet::with_capacity(500);
    let mut word_frequency_map = HashMap::with_capacity(100);
    for i in content.trim().split_ascii_whitespace() {
        if is_numeric(i) || ignore_word_set.contains(i) {
            continue;
        }
        if !i.chars().last().unwrap().is_alphabetic() {
            let key = i.chars().take(i.len() - 1).collect::<String>();
            let key = key.to_ascii_lowercase();
            word_frequency_map
                .entry(key)
                .and_modify(|e| {
                    *e += 1;
                })
                .or_insert(1);
        } else {
            word_frequency_map
                .entry(i.to_ascii_lowercase().to_owned())
                .and_modify(|e| {
                    *e += 1;
                })
                .or_insert(1);
        }
    }
    let mut pq = BinaryHeap::with_capacity(101);
    for (key, val) in word_frequency_map {
        pq.push((val, key.to_owned()));
        if pq.len() > 100 {
            pq.pop();
        }
    }
    for i in pq {
        set.insert(i.1);
    }
    set
}

fn is_numeric(s: &str) -> bool {
    for i in s.chars() {
        if i.is_ascii_digit() {
            return true;
        }
    }

    false
}

fn fill_set() -> HashSet<String> {
    let mut set = HashSet::with_capacity(100);
    let ignore_words = [
        "the", "a", "is", "that", "at", "he", "she", "what", "when", "they", "are", "how", "let",
        "yes", "no", "i", "me",
    ];
    ignore_words.into_iter().for_each(|x| {
        set.insert(x.to_owned());
    });
    set
}
