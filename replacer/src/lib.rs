use std::borrow::Cow;
use std::fmt::Error;
use std::fs::File;
use std::io::{self, Read, Write};

extern crate models;

use models::{ChangeString, ChangeSubject, ChangeStatus};

use std::path::PathBuf;

#[cfg(target_os = "linux")]
use std::os::unix::fs::MetadataExt;

#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

#[cfg(target_os = "macos")]
use std::os::darwin::fs::MetadataExt;

// there is two replacement strategies. The most logical thing to do is if from string is too small just use
// replace string and if it's large use streaming.

pub fn replace_string(path: &PathBuf, from: &String, to: &String, threshold: u64) -> Result<(), std::io::Error> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) => return Err(error)
    };

    #[cfg(target_os = "linux")]
    {
        match file.metadata() {
            Ok(meta) => {
                if meta.size() > threshold {
                    match replace_string_streaming(&mut file, from, to) {
                        Ok(_) => Ok(()),
                        Err(error) => {
                            println!("cannot replaced string for that reason: {}", error);

                            return Err(error)

                        }
                    }
                } else {
                    match replace_string_directly(path, from, to) {
                        Ok(_) => Ok(()),
                        Err(error) => {
                            println!("cannot replaced string for that reason: {}", error);

                            return Err(error)
                        }
                    }
                }
            },
            Err(error) => {
                println!("Cannot reach file's metadata for that reason: {}", error);

                return Err(error)
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        match file.metadata() {
            Ok(meta) => {
                if meta.file_size() > threshold {
                    match replace_string_streaming(&mut file, from, to) {
                        Ok(_) => Ok(()),
                        Err(error) => {
                            println!("cannot replaced string for that reason: {}", error);

                            return Err(error)

                        }
                    }
                } else {
                    match replace_string_directly(path, from, to) {
                        Ok(_) => (),
                        Err(error) => {
                            println!("cannot replaced string for that reason: {}", error);

                            return Err(error)
                        }
                    }
                }
            },
            Err(error) => {
                println!("Cannot reach file's metadata for that reason: {}", error);

                return Err(error)
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        match file.metadata() {
            Ok(meta) => {
                if meta.st_size() > threshold {
                    match replace_string_streaming(&mut file, from, to) {
                        Ok(_) => Ok(()),
                        Err(error) => {
                            println!("cannot replaced string for that reason: {}", error);

                            return Err(error)

                        }
                    }
                } else {
                    match replace_string_directly(path, from, to) {
                        Ok(_) => (),
                        Err(error) => {
                            println!("cannot replaced string for that reason: {}", error);

                            return Err(error)
                        }
                    }
                }
            },
            Err(error) => {
                println!("Cannot reach file's metadata for that reason: {}", error);

                return Err(error)
            }
        }
    }
}

pub fn replace_string_directly(path: &PathBuf, from: &String, to: &String) -> Result<(), std::io::Error> {
    let file_content;
    {
        let mut bytes = vec![];
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(error) => return Err(error)
        };


        match file.read_to_end(&mut bytes) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };

        file_content = String::from_utf8_lossy(&bytes).to_string()
    }

    let updated_content = file_content.replace(from, to);

    {
        let mut file = match File::create(path) {
            Ok(file) => file,
            Err(error) => return Err(error)
        };

        match file.write_all(updated_content.as_bytes()) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };
    }

    Ok(())
}

pub fn replace_string_streaming(file: &mut File, from: &String, to: &String) -> Result<(), std::io::Error> {
    match stream(file, from, to) {
        Ok(_) => Ok(()),
        Err(error) => Err(error)
    }
}


// the from and buffer's length will be same. because of that it could be only 1 whole match
// and we can make evaluation accurately.

// steps:

// 1 - read the files bytes.
// 2 - check a from's last chars are matches with chunks first chars.
// 3 - if it is, check the first chars of next from is matches with current remaining chars of from. 
pub fn stream(file: &mut File, from: &String, to: &String) -> Result<(), std::io::Error> {
    let byte_size = from.chars().count();
    
    let mut last = None;
    let mut first = None;

    let mut prev_buffer = vec![];

    loop {
        let mut buffer = vec![0; byte_size];

        match file.read(&mut buffer) {
            Ok(0) => break, // Okuma bitti
            Ok(n) => n,
            Err(e) => {
                println!("that error occured when we try to read bytes: {}", e);
                
                continue;
            }
        };

        let mut get_string = String::from_utf8_lossy(&buffer);
        let mut get_from: String = from.clone();

        let mut should_write = ChangeStatus::None;
    
        match get_string.contains(&get_from) {
            true => should_write = ChangeStatus::Whole,
            false => {
                let clone_buffer = get_string.clone();
                
                match last {
                    None => {
                        //let chars = from.chars().count();

                        let mut clone_from = get_from.clone();

                        while clone_from.pop().is_some() {
                            match clone_buffer.ends_with(&*clone_from) {
                                true => {
                                    prev_buffer = buffer.clone();

                                    last = Some(clone_from.clone());

                                    should_write = ChangeStatus::LastCollected;

                                    break;
                                },
                                false => ()
                            }
                        }
                    },
                    Some(_) => {
                        // burada da first varsa yapılacaklar yazılacak.
                        let mut from_length = get_from.chars().count();

                        match from_length {
                            0 => (),
                            1 => {
                                match clone_buffer.starts_with(&*get_from) {
                                    true => {
                                        first = Some(get_from);

                                        should_write = ChangeStatus::LastToStart;
                                    },
                                    false => ()
                                }

                                prev_buffer = vec![];
                            },
                            _ => {
                                loop {
                                    let mut get_from = get_from.clone();
                                    from_length = get_from.chars().count();

                                    get_from.remove(0);
            
                                    match clone_buffer.starts_with(&*get_from) {
                                        true => {
                                            first = Some(get_from);

                                            should_write = ChangeStatus::LastToStart;
                                        },
                                        false => ()
                                    }

                                    from_length = from_length - 1;
            
                                    if from_length == 1 {
                                        break;
                                    }
                                }

                                match first {
                                    None => prev_buffer = vec![],
                                    Some(_) => ()
                                }
                            }
                        }
                    }
                }
            }
        }

        match should_write {
            ChangeStatus::None => {
                match file.write_all(&buffer) {
                    Ok(_) => (),
                    Err(error) => return Err(error)
                }
            },
            ChangeStatus::Whole => {
                match file.write_all(&to.as_bytes()) {
                    Ok(_) => (),
                    Err(error) => return Err(error)
                }

                last = None;
                first = None;
            },
            ChangeStatus::LastCollected => (),
            ChangeStatus::LastToStart => {
                match prev_buffer.len() {
                    0 => panic!("prev_buffer variable shouldn't be the length of 0 at this moment, panicking."),
                    _ => ()
                }

                match last {
                    None => panic!("last variable shouldn't be the length of 0 at this moment, panicking."),
                    Some(mut last) => {
                        prev_buffer.reverse();

                        let mut new_last = None;

                        unsafe {
                            let mut newest_last: &mut [u8] = last.as_mut_vec();
                            newest_last.reverse();
                            new_last = Some(newest_last)
                        }

                        if let Some(last) = new_last {
                            for (index, s) in last.iter().enumerate() {
                                prev_buffer[index] = *s;
                            }

                            prev_buffer.reverse();

                            match file.write_all(&prev_buffer) {
                                Ok(_) => (),
                                Err(error) => return Err(error)
                            }
                        }
                    }
                }

                match first {
                    None => panic!("first variable shouldn't be the length of 0 at this moment, panicking."),
                    Some(first) => {
                        let first_as_bytes = first.as_bytes();

                        for (index, s) in first_as_bytes.iter().enumerate() {
                            buffer[index] = *s;
                        }

                        match file.write_all(&buffer) {
                            Ok(_) => (),
                            Err(error) => return Err(error)
                        }
                    }

                }
                

                last = None;
                first = None;
            }
        }
    }

    Ok(())
}


// It reads the buffer and returns if a string should changed.
// there is a little drawback here: it assumes only one match in given haystack. We
// have to make it check every match. For that, we can make buffer size to haystack's size. 
/*pub fn evaluate_change_string<'a>(string: &'a Vec<u8>, from: String, first: &'a mut Option<String>, last: &'a mut Option<String>, change_str: &'a mut ChangeString) -> &'a ChangeString<'a> {
    let mut get_string = String::from_utf8_lossy(&string);
    let mut get_from: String = from;

    match get_string.contains(&get_from.clone()) {
        true => {
            *change_str = ChangeString::Whole(get_from.as_bytes().clone());

            return &change_str
        },
        false => {
            let mut clone_from = get_from.clone();

            while clone_from.pop().is_some() {
                match string.ends_with(clone_from.as_bytes()) {
                    true => *last = Some(clone_from.clone())/*return ChangeString::FromEnd(clone_from)*/,
                    false => ()
                }
            }

            let mut from_length = get_from.chars().count();

            match from_length {
                0 => (),
                1 => {
                    match string.starts_with(get_from.as_bytes()) {
                        true => *first = Some(get_from.clone())/*return ChangeString::FromStart(get_from)*/,
                        false => ()
                    }
                },
                _ => {
                    loop {
                        get_from.remove(0);

                        match string.starts_with(get_from.as_bytes()) {
                            true => *first = Some(get_from)/*return ChangeString::FromStart(get_from)*/,
                            false => ()
                        }

                        from_length = get_from.chars().count();

                        if from_length == 1 {
                            break;
                        }
                    }
                }
            }

            return &ChangeString::None
        }
    }
}*/


// not yet completed
/*pub fn evaluate_change_string(string: &Vec<u8>, from: &String) -> bool {
    let get_string = String::from_utf8_lossy(&string);

    get_string.contains(from)
}*/

fn replace_in_buffer(buffer: &mut Vec<u8>, from: &[u8], to: &[u8]) {
    if from.len() != to.len() {
        return; // Farklı uzunlukta değiştirme yapmıyoruz
    }

    let mut i = 0;
    while let Some(pos) = buffer[i..].windows(from.len()).position(|window| window == from) {
        let abs_pos = i + pos;
        buffer[abs_pos..abs_pos + to.len()].copy_from_slice(to);
        i = abs_pos + to.len(); // Aramaya devam et
    }
}


// this is a very complicated version of that function, because of that we avoid it:
/*fn replace_in_buffer(buffer: &mut Vec<u8>, from: &[u8], to: &[u8]) -> io::Result<()> {
    let mut i = 0;
    while let Some(pos) = buffer[i..].windows(from.len()).position(|window| window == from) {
        let abs_pos = i + pos;

        if from.len() == to.len() {
            // Aynı uzunlukta: Direkt değiştir
            buffer[abs_pos..abs_pos + to.len()].copy_from_slice(to);
        } else {
            // Farklı uzunlukta: Yeni bir Vec<u8> oluştur
            let mut new_buffer = Vec::with_capacity(buffer.len() - from.len() + to.len());
            new_buffer.extend_from_slice(&buffer[..abs_pos]);
            new_buffer.extend_from_slice(to);
            new_buffer.extend_from_slice(&buffer[abs_pos + from.len()..]);

            *buffer = new_buffer; // Yeni buffer'a geçir
            i = abs_pos + to.len();
            continue;
        }

        i = abs_pos + to.len();
    }

    Ok(())
}*/

// it benefits to find how many times needle occures on haystack.
/*fn count_occurrences(haystack: Cow<str>, needle: &str) -> usize {
    haystack.matches(needle).count()
}*/