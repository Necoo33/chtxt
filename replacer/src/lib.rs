use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;

pub fn replace_string(path: &PathBuf, from: &String, to: &String) -> Result<(), std::io::Error> {
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