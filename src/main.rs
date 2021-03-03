#[macro_use]
extern crate derive_new;

mod cli;
mod osu;
mod osz_finder;
mod pathtree_stylizer;

use crate::osu::{Osu, Osu40, Osu50, OsuBeatmapGrade, OsuBeatmapStatus};
use crate::osz_finder::find_oszs;
use crate::pathtree_stylizer::PathTreeStylized;
use std::convert::TryFrom;
use std::path::PathBuf;

type ResultOsuOpener = Result<Box<dyn Osu>, String>;
type FnOsuOpener = dyn Fn(&PathBuf) -> ResultOsuOpener;

const FN_OSU_OPENER: [&FnOsuOpener; 1] = [
    //2] = [
    &(|x| Osu40::try_from(x).map(|a| a.boxed())),
    // &(|x| Osu50::try_from(x).map(|a| a.boxed())),
];

fn main() -> Result<(), String> {
    let args = crate::cli::get_arguments_parsed();
    if !args.packs_source.is_dir() && !args.packs_source.is_file() {
        return Err(format!(
            "{:?} is neither a directory nor a file",
            args.packs_source
        ));
    }
    let osu_open_result: Vec<ResultOsuOpener> =
        FN_OSU_OPENER.iter().map(|x| x(&args.osu_source)).collect();
    let osu_open_successes: Option<&Box<dyn Osu>> = osu_open_result
        .iter()
        .filter_map(|x| x.as_ref().ok())
        .next();
    let osu_open_errors: Vec<&String> = osu_open_result
        .iter()
        .filter_map(|x| x.as_ref().err())
        .collect();
    if let Some(osu) = osu_open_successes {
        let oszs = find_oszs(&args.packs_source);
        if oszs.len() == 0 {
            return Err(format!("{:?} contains no beatmapset", args.packs_source));
        } else {
            let osz_ids: Vec<(&PathBuf, u64)> = oszs
                .iter()
                .map(|osz_path: &PathBuf| {
                    (
                        osz_path,
                        osz_path
                            .file_stem()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .chars()
                            .take_while(|c| c.is_numeric())
                            .collect::<String>()
                            .parse::<u64>()
                            .unwrap(),
                    )
                })
                .collect();
            let osz_statuses: Vec<(PathBuf, OsuBeatmapStatus)> = osz_ids
                .iter()
                .map(|(osz_path, bms_id): &(&PathBuf, u64)| {
                    (
                        (*osz_path).clone(),
                        osu.get_beatmapset(*bms_id)
                            .and_then(|bms| Some(bms.worst_rank()))
                            .unwrap_or(OsuBeatmapStatus::NotInstalled),
                    )
                })
                .collect();
            let mut pathtree_stylized = PathTreeStylized::from(&osz_statuses);
            pathtree_stylized.fill_data_greatest();
            pathtree_stylized.sort();
            pathtree_stylized.reverse();
            let style_obviously_pending = ansi_term::Style::new()
                .bold()
                .fg(ansi_term::Color::Red)
                .on(ansi_term::Color::Yellow);
            let style_pending = ansi_term::Style::new().bold().fg(ansi_term::Color::Red);
            let style_done = ansi_term::Style::new().bold().fg(ansi_term::Color::Green);
            let style_base = ansi_term::Style::new().dimmed();
            pathtree_stylized.set_colors(
                Some(
                    vec![
                        (
                            OsuBeatmapStatus::NotInstalled,
                            (
                                style_obviously_pending.prefix().to_string(),
                                style_obviously_pending.suffix().to_string(),
                            ),
                        ),
                        (
                            OsuBeatmapStatus::NotPlayed,
                            (
                                style_pending.prefix().to_string(),
                                style_pending.suffix().to_string(),
                            ),
                        ),
                        (
                            OsuBeatmapStatus::Played(OsuBeatmapGrade::SSSilver),
                            (
                                style_done.prefix().to_string(),
                                style_done.suffix().to_string(),
                            ),
                        ),
                        (
                            OsuBeatmapStatus::Played(OsuBeatmapGrade::SSilver),
                            (
                                style_done.prefix().to_string(),
                                style_done.suffix().to_string(),
                            ),
                        ),
                        (
                            OsuBeatmapStatus::Played(OsuBeatmapGrade::SS),
                            (
                                style_done.prefix().to_string(),
                                style_done.suffix().to_string(),
                            ),
                        ),
                        (
                            OsuBeatmapStatus::Played(OsuBeatmapGrade::S),
                            (
                                style_done.prefix().to_string(),
                                style_done.suffix().to_string(),
                            ),
                        ),
                        (
                            OsuBeatmapStatus::Played(OsuBeatmapGrade::A),
                            (
                                style_done.prefix().to_string(),
                                style_done.suffix().to_string(),
                            ),
                        ),
                        (
                            OsuBeatmapStatus::Played(OsuBeatmapGrade::B),
                            (
                                style_done.prefix().to_string(),
                                style_done.suffix().to_string(),
                            ),
                        ),
                        (
                            OsuBeatmapStatus::Played(OsuBeatmapGrade::C),
                            (
                                style_done.prefix().to_string(),
                                style_done.suffix().to_string(),
                            ),
                        ),
                        (
                            OsuBeatmapStatus::Played(OsuBeatmapGrade::D),
                            (
                                style_done.prefix().to_string(),
                                style_done.suffix().to_string(),
                            ),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                ),
                Some((
                    style_base.prefix().to_string(),
                    style_base.suffix().to_string(),
                )),
            );
            // println!("{:#?}", pathtree_stylized);
            println!("{}", pathtree_stylized);
        }
    } else {
        return Err(osu_open_errors
            .into_iter()
            .cloned()
            .collect::<Vec<String>>()
            .join(", "));
    }
    Ok(())
}
