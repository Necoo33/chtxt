use std::borrow::Cow;
use std::fmt::Error;
use std::fs::File;
use std::io::{self, Read, Write};

use std::path::PathBuf;

#[cfg(target_os = "linux")]
use std::os::unix::fs::MetadataExt;

#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

// there is two replacement strategies. The most logical thing to do is if from string is too small just use
// replace string and if it's large use streaming.

pub fn replace_string(path: &PathBuf, from: &String, to: &String, threshold: u64) -> Result<(), std::io::Error> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) => return Err(error)
    };

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::MetadataExt;

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
pub fn stream(file: &mut File, from: &String, to: &String) -> Result<(), std::io::Error> {
    let byte_size = from.len();
    loop {
        let mut buffer = vec![0; byte_size]; // 8 KB buffer

        match file.read(&mut buffer) {
            Ok(0) => break, // Okuma bitti
            Ok(n) => n,
            Err(e) => {
                println!("that error occured when we try to read bytes: {}", e);
                
                continue;
            }
        };

        match evaluate_change_string(&buffer, from) {
            true => {
                replace_in_buffer(&mut buffer, from.as_bytes(), to.as_bytes());

                match file.write_all(&buffer) {
                    Ok(_) => (),
                    Err(error) => return Err(error)
                };

            },
            false => ()
        }

        /*if let Err(e) = new_file.write_all(&buffer[..bytes_read]) {
            println!("That error occured when we try to write bytes to the file: {}", e);
        }*/
    }

    Ok(())
}


// It reads the buffer and returns if a string should changed.
// there is a little drawback here: it assumes only one match in given haystack. We
// have to make it check every match. For that, we can make buffer size to haystack's size. 
/*pub fn evaluate_change_string(string: &Vec<u8>, from: String) -> ChangeString {
    let mut get_string = String::from_utf8_lossy(&string);
    let mut get_from = from;

    match get_string.contains(&get_from) {
        true => return ChangeString::Whole(get_from),
        false => {
            let mut clone_from = get_from.clone();

            while clone_from.pop().is_some() {
                match string.ends_with(clone_from.as_bytes()) {
                    true => return ChangeString::FromEnd(clone_from),
                    false => ()
                }
            }

            let mut from_length = get_from.chars().count();

            match from_length {
                0 => (),
                1 => {
                    match string.starts_with(get_from.as_bytes()) {
                        true => return ChangeString::FromStart(get_from),
                        false => ()
                    }
                },
                _ => {
                    loop {
                        get_from.remove(0);

                        match string.starts_with(get_from.as_bytes()) {
                            true => return ChangeString::FromStart(get_from),
                            false => ()
                        }

                        from_length = get_from.chars().count();

                        if from_length == 1 {
                            break;
                        }
                    }
                }
            }

            return ChangeString::None
        }
    }
}*/


// not yet completed
pub fn evaluate_change_string(string: &Vec<u8>, from: &String) -> bool {
    let get_string = String::from_utf8_lossy(&string);

    get_string.contains(from)
}

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
fn count_occurrences(haystack: Cow<str>, needle: &str) -> usize {
    haystack.matches(needle).count()
}