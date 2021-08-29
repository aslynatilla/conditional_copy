use std::path::PathBuf;

pub(crate) struct Controller {
    cci_content: String,
    destination: PathBuf,
}

impl Controller {
    pub(crate) fn new(cci_instructions: String) -> Self {
        let destination = cci_instructions
            .lines()
            .skip_while(|&l| l.eq("[destination]"))
            .skip(1)
            .next()
            .unwrap()
            .into();
        //TODO: validate destination

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

    pub(crate) fn target_list(&self) -> Vec<String> {
        self.targets().map(str::to_string).collect()
    }
}
