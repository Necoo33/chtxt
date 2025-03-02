use std::{fmt, fmt::Display};
use std::env::current_dir;

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
                        println!("We cannot get your path for that reason on allforwarddir: {}", error);

                        Err(fmt::Error)
                    }
                }
            }
            ChangeSubject::Singular(singular) => {
                let file_path = current_dir.join(singular);

                match file_path.canonicalize() {
                    Ok(path) => write!(f, "{}", path.display()),
                    Err(error) => {
                        println!("We cannot get your path for that reason on singular: {}", error);

                        Err(fmt::Error)
                    }
                }
            },
            ChangeSubject::AllBackDir(back_dirs) => {
                let total_dir = current_dir.join(back_dirs).join("/*");

                println!("our path buf: {:#?}", total_dir);

                match total_dir.canonicalize() {
                    Ok(path) => write!(f, "{}", path.display()),
                    Err(error) => {
                        println!("We cannot get your path for that reason on allbackdir: {}", error);

                        Err(fmt::Error)
                    }
                }
            },
            ChangeSubject::SingularBackDir(back_dirs, singular) => {
                let total_dir = current_dir.join(back_dirs).join(singular);

                match total_dir.canonicalize() {
                    Ok(path) => write!(f, "{}", path.display()),
                    Err(error) => {
                        println!("We cannot get your path for that reason on singularbackdir: {}", error);

                        Err(fmt::Error)
                    }
                }
            },
            _ => Ok(()) // not implemented yet
        }
    }
}

#[derive(Debug)]
pub enum ArgumentCapturingPhase {
    Normal, AllowedExtensions, Options
}

/*impl Display for ArgumentCapturingPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgumentCapturingPhase::Options => write!(f, "--opt"),
            ArgumentCapturingPhase::AllowedExtensions => write!(f, "--ext"),
            ArgumentCapturingPhase::Normal => write!(f, "")
        }
    }
}

impl ArgumentCapturingPhase {
    pub fn create(argument: String) -> Self {
        match argument.as_str() {
            "--opt" => ArgumentCapturingPhase::Options,
            "--ext" => ArgumentCapturingPhase::AllowedExtensions,
            _ => ArgumentCapturingPhase::Normal
        }
    }
}*/