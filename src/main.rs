use core::fmt;
use std::fmt::Display;
use std::fs;
use std::io::{self, Read, Write};
use std::env::{args, current_dir};

fn main() -> io::Result<()> {
    let mut args: Vec<String> = args().collect();

    let mut from = String::new();
    let mut to = String::new();
    let mut subject = ChangeSubject::All;

    for (index, arg) in args.iter_mut().enumerate() {
        match index {
            1 => {
                if arg != "" {
                    from = arg.to_string()
                } else {
                    println!("first argument cannot be empty, exiting...");
                    
                    return Ok(())
                }
            },
            2 => {
                match arg.as_str() {
                    "" => println!("second argument is empty. That means we assume you want to replace that string with empty string."),
                    "--empty" | "--delete" | "-empty" | "-delete" | "-e" | "-d" => to = "".to_string(),
                    _ => to = arg.to_string(),
                    
                }
            }
            3 => {
                match arg.as_str() {
                    "" | "*" | "./*" | "./" | "." => (),
                    _ => {
                        let back_paths = arg.matches("../").count();

                        match back_paths {
                            0 => {
                                match arg.starts_with("./") {
                                    true => {
                                        if arg.ends_with("*") || arg.ends_with("/") {
                                            let arg = arg.replace("./", "");

                                            subject = ChangeSubject::AllForwardDir(arg.clone())
                                        } else {
                                            let arg = arg.replace("./", "");

                                            println!("arg in current situation: {}", arg);
            
                                            subject = ChangeSubject::Singular(arg.clone())
                                        }
                                    }
                                    false => {
                                        let arg = arg;

                                        println!("arg in current situation: {}", arg);
        
                                        subject = ChangeSubject::Singular(arg.clone())
                                    }
                                }
                            }
                            _ => {
                                let mut new_back_path = "".to_string();

                                for _ in 0..back_paths {
                                    new_back_path = format!("{}../", new_back_path);
                                }

                                let pure_name_of_file = arg.replace(&new_back_path, "");

                                match pure_name_of_file.as_str() {
                                    "" | "*" => {
                                        let arg = arg;

                                        subject = ChangeSubject::AllBackDir(arg.clone())
                                    }
                                    _ => {
                                        subject = ChangeSubject::SingularBackDir(new_back_path, pure_name_of_file)
                                    }
                                }
                            }
                        }
                    }
                }
            },
            4 => {
                if arg != "" {
                    let back_paths = arg.matches("../").count();

                    match back_paths {
                        0 => {
                            subject = ChangeSubject::Multiple(vec![subject.to_string(), arg.to_string()]);       
                        },
                        _ => {
                            let mut new_back_path = "".to_string();

                            for _ in 0..back_paths {
                                new_back_path = format!("{}../", new_back_path);
                            }

                            let pure_name_of_file = arg.replace(&new_back_path, "");

                            subject = ChangeSubject::MultipleBackDir(new_back_path, vec![subject.to_string(), pure_name_of_file])
                        }
                    }
                }
            },
            _ => {
                let back_paths = arg.matches("../").count();

                match back_paths {
                    0 => {
                        if let ChangeSubject::Multiple(ref mut args) = subject {
                            args.push(arg.to_string());
                        }     
                    },
                    _ => {
                        if let ChangeSubject::MultipleBackDir(_, ref mut files) = subject {
                            files.push(arg.to_string());
                        }
                    }
                }
            }
            _ => continue
        }
    }

    println!("Are You Sure? If you are, please type 'y/e/j' or another key to exit.");

    let answer = &mut String::new();

    match io::stdin().read_line(answer) {
        Ok(_) => (),
        Err(error) => {
            println!("We cannot read your input for that reason, exiting...: {}", error);

            return Ok(())
        }
    }

    match answer.trim() {
        "y" | "e" | "j" => (),
        _ => {
            println!("exiting...");

            return Ok(())
        }
    }

    let current_dir = current_dir();

    let current_dir = match current_dir {
        Ok(dir) => dir,
        Err(error) => {
            println!("We cannot read that path of the current dir for that reason, exiting...: {}", error);

            return Ok(())
        }
    };

    println!("your current directory: {:?}", current_dir);
    println!("Are you want to proceed? Type 'y/e/j' if you want.");

    let answer = &mut String::new();

    match io::stdin().read_line(answer) {
        Ok(_) => (),
        Err(error) => {
            println!("We cannot read your input for that reason, exiting...: {}", error);

            return Ok(())
        }
    }

    match answer.trim() {
        "y" | "e" | "j" => (),
        _ => return Ok(())
    }

    match subject {
        ChangeSubject::All => {
            for entry in fs::read_dir(&current_dir)? {
                let entry = entry?;
        
                let path = entry.path();
        
                match path.is_file() {
                    true => {
                        let file_content;
                        {
                            let mut bytes = vec![];
                            let mut file = fs::File::open(&path)?;
                            file.read_to_end(&mut bytes)?;

                            file_content = String::from_utf8_lossy(&bytes).to_string()
                        }
                
                        let updated_content = file_content.replace(&from, &to);
                
                        {
                            let mut file = fs::File::create(&path)?;
                            file.write_all(updated_content.as_bytes())?;
                        }
                
                        println!("{:#?}'s content changed", path);
                    },
                    false => ()
                }
            }
        },
        ChangeSubject::AllForwardDir(forward) => {
            let path = current_dir.join(forward);

            match path.canonicalize() {
                Ok(path) => {
                    for entry in fs::read_dir(&path)? {
                        let entry = entry?;
                
                        let path = entry.path();
                
                        match path.is_file() {
                            true => {
                                let file_content;
                                {
                                    let mut bytes = vec![];
                                    let mut file = fs::File::open(&path)?;
                                    file.read_to_end(&mut bytes)?;

                                    file_content = String::from_utf8_lossy(&bytes).to_string()
                                }
                        
                                let updated_content = file_content.replace(&from, &to);
                        
                                {
                                    let mut file = fs::File::create(&path)?;
                                    file.write_all(updated_content.as_bytes())?;
                                }
                        
                                println!("{:#?}'s content changed", path);
                            },
                            false => ()
                        }
                    }
                },
                Err(_) => {
                    println!("Your path input is invalid, exiting...");

                    return Ok(())
                }
            }
        },
        ChangeSubject::Singular(singular) => {
            let path = current_dir.join(&singular);

            match path.canonicalize() {
                Ok(path) => {
                    match path.is_file() {
                        true => {
                            let file_content;
                            {
                                let mut bytes = vec![];
                                let mut file = fs::File::open(&path)?;
                                file.read_to_end(&mut bytes)?;

                                file_content = String::from_utf8_lossy(&bytes).to_string()
                            }
                    
                            let updated_content = file_content.replace(&from, &to);
                    
                            {
                                let mut file = fs::File::create(&path)?;
                                file.write_all(updated_content.as_bytes())?;
                            }
                    
                            println!("{:#?}'s content changed", path);
                        },
                        false => {
                            println!("{} is not a file, exiting...", singular);

                            return Ok(())
                        }
                    }
                },
                Err(_) => {
                    println!("Your path input is invalid, exiting...");

                    return Ok(())
                }
            }
        },
        ChangeSubject::AllBackDir(back_dirs) => {
            let join_paths = current_dir.join(back_dirs);

            match join_paths.canonicalize() {
                Ok(path) => {
                    for entry in fs::read_dir(&path)? {
                        println!("our dir entry: {:#?}", entry);

                        let entry = entry?;
                
                        let path = entry.path();
                
                        match path.is_file() {
                            true => {
                                let file_content;
                                {
                                    let mut bytes = vec![];
                                    let mut file = fs::File::open(&path)?;
                                    file.read_to_end(&mut bytes)?;

                                    file_content = String::from_utf8_lossy(&bytes).to_string()
                                }
                        
                                let updated_content = file_content.replace(&from, &to);
                        
                                {
                                    let mut file = fs::File::create(&path)?;
                                    file.write_all(updated_content.as_bytes())?;
                                }
                        
                                println!("{:#?}'s content changed", path);
                            },
                            false => ()
                        }
                    }
                },
                Err(_) => {
                    println!("Your path input is invalid, exiting...");

                    return Ok(())
                }
            }
        },
        ChangeSubject::SingularBackDir(back_dirs, file) => {
            let path = current_dir.join(back_dirs).join(&file);

            match path.canonicalize() {
                Ok(path) => {
                    match path.is_file() {
                        true => {
                            let file_content;
                            {
                                let mut bytes = vec![];
                                let mut file = fs::File::open(&path)?;
                                file.read_to_end(&mut bytes)?;

                                file_content = String::from_utf8_lossy(&bytes).to_string()
                            }
                    
                            let updated_content = file_content.replace(&from, &to);
                    
                            {
                                let mut file = fs::File::create(&path)?;
                                file.write_all(updated_content.as_bytes())?;
                            }
                    
                            println!("{:#?}'s content changed", path);
                        },
                        false => {
                            println!("{} is not a file, exiting...", file);

                            return Ok(())
                        }
                    }
                },
                Err(_) => {
                    println!("Your path input is invalid, exiting...");

                    return Ok(())
                }
            }
        },
        ChangeSubject::Multiple(files) => {
            for file in files.into_iter() {
                let path = current_dir.join(file);

                match path.canonicalize() {
                    Ok(path) => {
                        match path.is_file() {
                            true => {
                                let file_content;
                                {
                                    let mut bytes = vec![];
                                    let mut file = fs::File::open(&path)?;
                                    file.read_to_end(&mut bytes)?;

                                    file_content = String::from_utf8_lossy(&bytes).to_string()
                                }
                        
                                let updated_content = file_content.replace(&from, &to);
                        
                                {
                                    let mut file = fs::File::create(&path)?;
                                    file.write_all(updated_content.as_bytes())?;
                                }
                        
                                println!("{:#?}'s content changed", path);
                            },
                            false => ()
                        }
                    },
                    Err(_) => {
                        println!("Your path input is invalid, exiting...");
    
                        return Ok(())
                    }
                }
            }
        },
        ChangeSubject::MultipleBackDir(back_paths, files) => {
            for file in files {
                let path = current_dir.join(back_paths.clone()).join(file);

                match path.canonicalize() {
                    Ok(path) => {
                        match path.is_file() {
                            true => {
                                let file_content;
                                {
                                    let mut bytes = vec![];
                                    let mut file = fs::File::open(&path)?;
                                    file.read_to_end(&mut bytes)?;

                                    file_content = String::from_utf8_lossy(&bytes).to_string()
                                }
                        
                                let updated_content = file_content.replace(&from, &to);
                        
                                {
                                    let mut file = fs::File::create(&path)?;
                                    file.write_all(updated_content.as_bytes())?;
                                }
                        
                                println!("{:#?}'s content changed", path);
                            },
                            false => ()
                        }
                    },
                    Err(_) => {
                        println!("Your path input is invalid, exiting...");
    
                        return Ok(())
                    }
                }
            }
        }
        _ => println!("Not implemented yet!")
    }

    Ok(())
}

pub enum ChangeSubject {
    Singular(String), Multiple(Vec<String>), All, AllForwardDir(String), SingularBackDir(String, String), AllBackDir(String), MultipleBackDir(String, Vec<String>)
}

impl Display for ChangeSubject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let current_dir = match current_dir() {
            Ok(dir) => dir,
            Err(error) => {
                 println!("We cannot get your path for that reason: {}", error);

                 return Err(fmt::Error)
             } 
         };

        match self {
            ChangeSubject::All => write!(f, "{}/*", current_dir.display()),
            ChangeSubject::AllForwardDir(forward) => {
                let file_path = current_dir.join(forward);

                match file_path.canonicalize() {
                    Ok(path) => write!(f, "{}", path.display()),
                    Err(error) => {
                        println!("We cannot get your path for that reason: {}", error);

                        Err(fmt::Error)
                    }
                }
            }
            ChangeSubject::Singular(singular) => {
                let file_path = current_dir.join(singular);

                match file_path.canonicalize() {
                    Ok(path) => write!(f, "{}", path.display()),
                    Err(error) => {
                        println!("We cannot get your path for that reason: {}", error);

                        Err(fmt::Error)
                    }
                }
            },
            ChangeSubject::AllBackDir(back_dirs) => {
                let total_dir = current_dir.join(back_dirs).join("/*");

                match total_dir.canonicalize() {
                    Ok(path) => write!(f, "{}", path.display()),
                    Err(error) => {
                        println!("We cannot get your path for that reason: {}", error);

                        Err(fmt::Error)
                    }
                }
            },
            ChangeSubject::SingularBackDir(back_dirs, singular) => {
                let total_dir = current_dir.join(back_dirs).join(singular);

                match total_dir.canonicalize() {
                    Ok(path) => write!(f, "{}", path.display()),
                    Err(error) => {
                        println!("We cannot get your path for that reason: {}", error);

                        Err(fmt::Error)
                    }
                }
            },
            _ => Ok(()) // not implemented yet
        }
    }
}