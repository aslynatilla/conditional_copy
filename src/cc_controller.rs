use std::fs::{copy, create_dir, File};
use std::path::PathBuf;
pub(crate) struct Controller {
    cci_content: String,
    destination: PathBuf,
}

impl Controller {
    pub(crate) fn new(cci_instructions: String) -> Self {
        let destination = cci_instructions
            .lines()
            .skip_while(|&l| !l.eq("[destination]"))
            .skip(1)
            .next()
            .unwrap()
            .into();

        Controller {
            cci_content: cci_instructions,
            destination,
        }
    }

    pub(crate) fn destination(&self) -> PathBuf {
        self.destination.clone()
    }

    pub(crate) fn targets(&self) -> impl Iterator<Item = &str> {
        self.cci_content
            .lines()
            .skip_while(|&l| !l.eq("[targets]"))
            .skip(1)
            .take_while(|&l| !l.starts_with('['))
            .filter(|&l| !l.eq(""))
        //TODO: filter more by validating instructions
    }

    pub(crate) fn target_list(&self) -> Vec<PathBuf> {
        self.targets().map(PathBuf::from).collect()
    }

    pub(crate) fn copy_all_targets(&self) -> Result<(), std::io::Error> {
        let destination_dir = self.destination();
        let destination_dir_exists = destination_dir.as_path().exists();
        let targets = self.target_list();

        if targets.len() > 0 && !destination_dir_exists {
            create_dir(&destination_dir).unwrap();
        }

        for target in targets {
            let maybe_file = File::open(&target);
            match maybe_file {
                Ok(_) => {}
                //  This should not be necessary, and it should have been already validated
                _ => todo!("Handle non existing file in target somewhere!"),
            }

            //TODO: target.compose_to(destination_dir.clone());
            //  fn compose_to(dir : PathBuf) -> PathBuf
            //  * Alternative name: add_to
            let mut destination = destination_dir.clone();
            destination.push(target.file_name().unwrap());

            match File::open(destination.as_path()) {
                Err(e) => match e.kind() {
                    std::io::ErrorKind::PermissionDenied => eprintln!("Permission denied"),
                    std::io::ErrorKind::NotFound => {
                        File::create(&destination).unwrap();
                        copy(&target, destination).unwrap();
                    }
                    _ => {
                        eprintln!("Unhandled error!")
                    }
                },
                _ => {
                    eprintln!("File {:?} already exists.", destination.as_path());
                    todo!("Add logic to update the file conditionally.");
                }
            }
        }
        Ok(())
    }
}
