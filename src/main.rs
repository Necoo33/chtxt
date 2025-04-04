use std::fs;
use std::io::{self, Read, Write};
use std::env::{args, current_dir};
use std::time::Instant;

extern crate models;

use models::{ChangeSubject, ArgumentCapturingPhase};
use replacer::replace_string;

fn main() -> io::Result<()> {
    let mut args: Vec<String> = args().collect();

    let mut from = String::new();
    let mut to = String::new();
    let mut subject = ChangeSubject::All;

    let mut phase = ArgumentCapturingPhase::Normal;
    let mut extensions = vec![];

    let mut threshold: u64 = 8192;

    for (index, arg) in args.iter_mut().enumerate() {
        match index {
            1 => {
                if arg != "" {
                    match arg == "--opt" || arg == "--options" {
                        false => from = arg.to_string(),
                        true => {
                            phase = ArgumentCapturingPhase::Options;

                            break
                        }
                    }
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
                    "--ext" | "--extension" | "--extensions" => {
                        phase = ArgumentCapturingPhase::AllowedExtensions;

                        continue;
                    }
                    "--bt" | "--buffering-threshold" => {
                        phase = ArgumentCapturingPhase::BufferingThreshold;

                        continue;
                    },
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

                                            //println!("arg in current situation: {}", arg);
            
                                            subject = ChangeSubject::Singular(arg.clone())
                                        }
                                    }
                                    false => {
                                        let arg = arg;

                                        //println!("arg in current situation: {}", arg);
        
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
                match arg.as_str() {
                    "--ext" | "--extension" | "--extensions" => {
                        phase = ArgumentCapturingPhase::AllowedExtensions;

                        continue;
                    },
                    "--bt" | "--buffering-threshold" => {
                        phase = ArgumentCapturingPhase::BufferingThreshold;

                        continue;
                    },
                    _ => ()
                }

                match phase {
                    ArgumentCapturingPhase::AllowedExtensions => {
                        match arg.starts_with("."){
                            true =>  extensions.push(arg.chars().skip(1).collect::<String>()),
                            false => match !arg.starts_with("--") {
                                true => extensions.push(arg.clone()),
                                false => ()
                            },
                        }
                    },
                    ArgumentCapturingPhase::Normal => {
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
                    ArgumentCapturingPhase::BufferingThreshold => {
                        match arg.parse::<u64>() {
                            Ok(new_threshold) => threshold = new_threshold,
                            Err(_) => {
                                println!("Invalid buffering threshold argument, exiting...");

                                return Ok(())
                            }
                        }
                    }
                    _ => ()
                }
            },
            _ => {
                match arg.as_str() {
                    "--ext" | "--extension" | "--extensions" => {
                        phase = ArgumentCapturingPhase::AllowedExtensions;

                        continue;
                    },
                    "--bt" | "--buffering-threshold" => {
                        phase = ArgumentCapturingPhase::BufferingThreshold;

                        continue;
                    },
                    _ => ()
                }

                match phase {
                    ArgumentCapturingPhase::AllowedExtensions => {
                        match arg.starts_with("."){
                            true =>  extensions.push(arg.chars().skip(1).collect::<String>()),
                            false => match !arg.starts_with("--") {
                                true => extensions.push(arg.clone()),
                                false => ()
                            },
                        }
                    },
                    ArgumentCapturingPhase::Normal => {
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
                    },
                    ArgumentCapturingPhase::BufferingThreshold => {
                        match arg.parse::<u64>() {
                            Ok(new_threshold) => threshold = new_threshold,
                            Err(_) => {
                                println!("Invalid buffering threshold argument, exiting...");

                                return Ok(())
                            }
                        }
                    }
                    _ => ()
                }
            }
        }
    }
    
    match phase {
        ArgumentCapturingPhase::Options => {
            println!("Welcome to chtxt, that program is for changing given texts with another on file/files in specified path.");
            println!("Synthax for replacing text is like that: \n");
            println!("(binary) (text you want to replace) (text you want to put) (path specifier) (other flags and arguments) \n");
            println!("And, here is you flags that you can use: \n");
            println!("--opt, --options | --ext, --extension, --extensions |");

            return Ok(())
        },
        _ => ()
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

    //println!("your current directory: {:?}", current_dir);
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

    //let instant = Instant::now();

    match subject {
        ChangeSubject::All => {
            for entry in fs::read_dir(&current_dir)? {
                let entry = entry?;
        
                let path = entry.path();
        
                match path.is_file() {
                    true => {
                        match extensions.len() {
                            0 => {
                                match replace_string(&path, &from, &to, threshold) {
                                    Ok(_) => println!("{:#?}'s content changed", path),
                                    Err(error) => println!("{}", error)
                                }
                            },
                            _ => {
                                match path.extension() {
                                    Some(ext) => {
                                        for extension in extensions.iter() {
                                            if *ext == **extension {
                                                match replace_string(&path, &from, &to, threshold) {
                                                    Ok(_) => println!("{:#?}'s content changed", path),
                                                    Err(error) => println!("{}", error)
                                                }
                                            }
                                        }
                                    },
                                    None => ()
                                }  
                            }
                        }
              
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
                                match extensions.len() {
                                    0 => {
                                        match replace_string(&path, &from, &to, threshold) {
                                            Ok(_) => println!("{:#?}'s content changed", path),
                                            Err(error) => println!("{}", error)
                                        }
                                    },
                                    _ => {
                                        match path.extension() {
                                            Some(ext) => {
                                                for extension in extensions.iter() {
                                                    if *ext == **extension {
                                                        match replace_string(&path, &from, &to, threshold) {
                                                            Ok(_) => println!("{:#?}'s content changed", path),
                                                            Err(error) => println!("{}", error)
                                                        }
                                                    }
                                                }
                                            },
                                            None => ()
                                        }  
                                    }
                                }
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
                            match extensions.len() {
                                0 => {
                                    match replace_string(&path, &from, &to, threshold) {
                                        Ok(_) => println!("{:#?}'s content changed", path),
                                        Err(error) => println!("{}", error)
                                    }
                                },
                                _ => {
                                    match path.extension() {
                                        Some(ext) => {
                                            for extension in extensions.iter() {
                                                if *ext == **extension {
                                                    match replace_string(&path, &from, &to, threshold) {
                                                        Ok(_) => println!("{:#?}'s content changed", path),
                                                        Err(error) => println!("{}", error)
                                                    }
                                                }
                                            }
                                        },
                                        None => ()
                                    }  
                                }
                            }
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
                        let entry = entry?;
                
                        let path = entry.path();
                
                        match path.is_file() {
                            true => {
                                match extensions.len() {
                                    0 => {
                                        match replace_string(&path, &from, &to, threshold) {
                                            Ok(_) => println!("{:#?}'s content changed", path),
                                            Err(error) => println!("{}", error)
                                        }
                                    },
                                    _ => {
                                        match path.extension() {
                                            Some(ext) => {
                                                for extension in extensions.iter() {
                                                    if *ext == **extension {
                                                        match replace_string(&path, &from, &to, threshold) {
                                                            Ok(_) => println!("{:#?}'s content changed", path),
                                                            Err(error) => println!("{}", error)
                                                        }
                                                    }
                                                }
                                            },
                                            None => ()
                                        }  
                                    }
                                }
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
                            match extensions.len() {
                                0 => {
                                    match replace_string(&path, &from, &to, threshold) {
                                        Ok(_) => println!("{:#?}'s content changed", path),
                                        Err(error) => println!("{}", error)
                                    }
                                },
                                _ => {
                                    match path.extension() {
                                        Some(ext) => {
                                            for extension in extensions.iter() {
                                                if *ext == **extension {
                                                    match replace_string(&path, &from, &to, threshold) {
                                                        Ok(_) => println!("{:#?}'s content changed", path),
                                                        Err(error) => println!("{}", error)
                                                    }
                                                }
                                            }
                                        },
                                        None => ()
                                    }  
                                }
                            }
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
                                match extensions.len() {
                                    0 => {
                                        match replace_string(&path, &from, &to, threshold) {
                                            Ok(_) => println!("{:#?}'s content changed", path),
                                            Err(error) => println!("{}", error)
                                        }
                                    },
                                    _ => {
                                        match path.extension() {
                                            Some(ext) => {
                                                for extension in extensions.iter() {
                                                    if *ext == **extension {
                                                        match replace_string(&path, &from, &to, threshold) {
                                                            Ok(_) => println!("{:#?}'s content changed", path),
                                                            Err(error) => println!("{}", error)
                                                        }
                                                    }
                                                }
                                            },
                                            None => ()
                                        }  
                                    }
                                }
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
                                match extensions.len() {
                                    0 => {
                                        match replace_string(&path, &from, &to, threshold) {
                                            Ok(_) => println!("{:#?}'s content changed", path),
                                            Err(error) => println!("{}", error)
                                        }
                                    },
                                    _ => {
                                        match path.extension() {
                                            Some(ext) => {
                                                for extension in extensions.iter() {
                                                    if *ext == **extension {
                                                        match replace_string(&path, &from, &to, threshold) {
                                                            Ok(_) => println!("{:#?}'s content changed", path),
                                                            Err(error) => println!("{}", error)
                                                        }
                                                    }
                                                }
                                            },
                                            None => ()
                                        }  
                                    }
                                }
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

    // println!("elapsed time: {}", instant.elapsed().as_millis());

    Ok(())
}

