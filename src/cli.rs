use std::path::PathBuf;

#[derive(Debug, Clone, new)]
pub struct CliArguments {
    pub osu_source: PathBuf,
    pub packs_source: PathBuf,
}

pub fn get_arguments_parsed() -> CliArguments {
    let mut ca = CliArguments::new(PathBuf::from(""), PathBuf::from(""));
    {
        let mut parser = argparse::ArgumentParser::new();
        parser.set_description("Checks which beatmap sets you haven't played yet.");

        parser
            .refer(&mut ca.osu_source)
            .add_argument("osu_source", argparse::Store, "Your Osu! folder")
            .required();
        parser
            .refer(&mut ca.packs_source)
            .add_argument("packs_source", argparse::Store, "Beatmapsets folder (folder to '.osz's, which can be inside .zip, .7z or .rar archives, but not nested)")
            .required();
        parser.parse_args_or_exit();
    }
    ca
}
