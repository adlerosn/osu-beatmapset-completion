use std::convert::TryFrom;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum OsuBeatmapGrade {
    SSSilver,
    SS,
    SSilver,
    S,
    A,
    B,
    C,
    D,
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum OsuBeatmapStatus {
    Played(OsuBeatmapGrade),
    NotPlayed,
    NotInstalled,
}

impl From<osu_db::listing::Grade> for OsuBeatmapStatus {
    fn from(grade: osu_db::listing::Grade) -> Self {
        match grade {
            osu_db::listing::Grade::SSPlus => OsuBeatmapStatus::Played(OsuBeatmapGrade::SSSilver),
            osu_db::listing::Grade::SPlus => OsuBeatmapStatus::Played(OsuBeatmapGrade::SSilver),
            osu_db::listing::Grade::SS => OsuBeatmapStatus::Played(OsuBeatmapGrade::SS),
            osu_db::listing::Grade::S => OsuBeatmapStatus::Played(OsuBeatmapGrade::S),
            osu_db::listing::Grade::A => OsuBeatmapStatus::Played(OsuBeatmapGrade::S),
            osu_db::listing::Grade::B => OsuBeatmapStatus::Played(OsuBeatmapGrade::S),
            osu_db::listing::Grade::C => OsuBeatmapStatus::Played(OsuBeatmapGrade::S),
            osu_db::listing::Grade::D => OsuBeatmapStatus::Played(OsuBeatmapGrade::S),
            osu_db::listing::Grade::Unplayed => OsuBeatmapStatus::NotPlayed,
        }
    }
}
impl From<Option<osu_db::listing::Grade>> for OsuBeatmapStatus {
    fn from(grade: Option<osu_db::listing::Grade>) -> Self {
        match grade {
            Some(g) => Self::from(g),
            None => OsuBeatmapStatus::NotInstalled,
        }
    }
}

pub trait Osu {
    fn boxed(self) -> Box<dyn Osu>;
    fn get_beatmapset(&self, beatmapset_id: u64) -> Option<OsuBeatmapSet>;
    fn get_beatmapset_maps(&self, beatmapset_id: u64) -> Vec<OsuBeatmap>;
    fn get_beatmap(&self, beatmapset_id: u64, beatmap_id: u64) -> Option<OsuBeatmap>;
    fn get_beatmap_grade_std(&self, beatmapset_id: u64, beatmap_id: u64) -> OsuBeatmapStatus;
    fn get_beatmap_grade_taiko(&self, beatmapset_id: u64, beatmap_id: u64) -> OsuBeatmapStatus;
    fn get_beatmap_grade_ctb(&self, beatmapset_id: u64, beatmap_id: u64) -> OsuBeatmapStatus;
    fn get_beatmap_grade_mania(&self, beatmapset_id: u64, beatmap_id: u64) -> OsuBeatmapStatus;
}
#[derive(Debug, Clone, new)]
pub struct Osu50HashResolver {
    pub folder: PathBuf,
}
impl Osu50HashResolver {
    pub fn resolve(&self, hash: &str) -> Result<PathBuf, String> {
        let final_path_buf = self
            .folder
            .join(hash.chars().take(1).collect::<String>())
            .join(hash.chars().take(2).collect::<String>())
            .join(hash);
        if final_path_buf.is_file() {
            Ok(final_path_buf)
        } else {
            Err(format!("{:?} is not a hashed file", final_path_buf))
        }
    }
}
#[derive(Debug, Clone, new)]
pub struct Osu40 {
    songs_path: Arc<PathBuf>,
    data_path: Arc<PathBuf>,
    replays_path: Arc<PathBuf>,
    osu_db: Arc<osu_db::listing::Listing>,
    collection_db: Arc<osu_db::collection::CollectionList>,
    scores_db: Arc<osu_db::score::ScoreList>,
}
#[derive(Debug, Clone, new)]
pub struct Osu50 {
    hash_resolver: Arc<Osu50HashResolver>,
    connection: Arc<rusqlite::Connection>,
}
impl Osu for Osu40 {
    fn boxed(self) -> Box<dyn Osu> {
        Box::new(self)
    }
    fn get_beatmapset(&self, beatmapset_id: u64) -> Option<OsuBeatmapSet> {
        Some(OsuBeatmapSet::new(Arc::new((*self).clone()), beatmapset_id))
    }
    fn get_beatmapset_maps(&self, beatmapset_id: u64) -> Vec<OsuBeatmap> {
        self.osu_db
            .beatmaps
            .iter()
            .filter(|x| x.beatmapset_id as u64 == beatmapset_id)
            .map(|x| {
                OsuBeatmap::new(
                    Arc::new((*self).clone()),
                    beatmapset_id,
                    x.beatmap_id as u64,
                )
            })
            .collect()
    }
    fn get_beatmap(&self, beatmapset_id: u64, beatmap_id: u64) -> Option<OsuBeatmap> {
        self.osu_db
            .beatmaps
            .iter()
            .filter(|x| {
                x.beatmapset_id as u64 == beatmapset_id && x.beatmap_id as u64 == beatmap_id
            })
            .map(|x| {
                OsuBeatmap::new(
                    Arc::new((*self).clone()),
                    beatmapset_id,
                    x.beatmap_id as u64,
                )
            })
            .next()
    }
    fn get_beatmap_grade_std(&self, beatmapset_id: u64, beatmap_id: u64) -> OsuBeatmapStatus {
        self.get_beatmap_(beatmapset_id, beatmap_id)
            .and_then(|x| Some(x.std_grade))
            .into()
    }
    fn get_beatmap_grade_taiko(&self, beatmapset_id: u64, beatmap_id: u64) -> OsuBeatmapStatus {
        self.get_beatmap_(beatmapset_id, beatmap_id)
            .and_then(|x| Some(x.taiko_grade))
            .into()
    }
    fn get_beatmap_grade_ctb(&self, beatmapset_id: u64, beatmap_id: u64) -> OsuBeatmapStatus {
        self.get_beatmap_(beatmapset_id, beatmap_id)
            .and_then(|x| Some(x.ctb_grade))
            .into()
    }
    fn get_beatmap_grade_mania(&self, beatmapset_id: u64, beatmap_id: u64) -> OsuBeatmapStatus {
        self.get_beatmap_(beatmapset_id, beatmap_id)
            .and_then(|x| Some(x.mania_grade))
            .into()
    }
}
impl Osu40 {
    fn get_beatmap_(
        &self,
        beatmapset_id: u64,
        beatmap_id: u64,
    ) -> Option<&osu_db::listing::Beatmap> {
        self.osu_db
            .beatmaps
            .iter()
            .filter(|x| {
                x.beatmapset_id as u64 == beatmapset_id && x.beatmap_id as u64 == beatmap_id
            })
            .next()
    }
}
impl Osu for Osu50 {
    fn boxed(self) -> Box<dyn Osu> {
        Box::new(self)
    }
    fn get_beatmapset(&self, beatmapset_id: u64) -> Option<OsuBeatmapSet> {
        Some(OsuBeatmapSet::new(Arc::new((*self).clone()), beatmapset_id))
    }
    fn get_beatmapset_maps(&self, beatmapset_id: u64) -> Vec<OsuBeatmap> {
        vec![]
    }
    fn get_beatmap(&self, beatmapset_id: u64, beatmap_id: u64) -> Option<OsuBeatmap> {
        None
    }
    fn get_beatmap_grade_std(&self, beatmapset_id: u64, beatmap_id: u64) -> OsuBeatmapStatus {
        OsuBeatmapStatus::NotInstalled
    }
    fn get_beatmap_grade_taiko(&self, beatmapset_id: u64, beatmap_id: u64) -> OsuBeatmapStatus {
        OsuBeatmapStatus::NotInstalled
    }
    fn get_beatmap_grade_ctb(&self, beatmapset_id: u64, beatmap_id: u64) -> OsuBeatmapStatus {
        OsuBeatmapStatus::NotInstalled
    }
    fn get_beatmap_grade_mania(&self, beatmapset_id: u64, beatmap_id: u64) -> OsuBeatmapStatus {
        OsuBeatmapStatus::NotInstalled
    }
}
impl TryFrom<&PathBuf> for Osu40 {
    type Error = String;
    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        if !path.is_dir() {
            return Err(format!("{:?} is not a directory", path));
        }
        let songs_path = path.join("Songs");
        if !songs_path.is_dir() {
            return Err(format!(
                "{:?} directory was not found in your osu!classic directory",
                songs_path
            ));
        }
        let data_path = path.join("Data");
        if !data_path.is_dir() {
            return Err(format!(
                "{:?} directory was not found in your osu!classic directory",
                data_path
            ));
        }
        let replays_path = path.join("Replays");
        if !replays_path.is_dir() {
            return Err(format!(
                "{:?} directory was not found in your osu!classic directory",
                replays_path
            ));
        }
        let osu_db_path = path.join("osu!.db");
        if !osu_db_path.is_file() {
            return Err(format!(
                "{:?} file was not found in your osu!classic directory",
                osu_db_path
            ));
        }
        let clct_db_path = path.join("collection.db");
        if !clct_db_path.is_file() {
            return Err(format!(
                "{:?} file was not found in your osu!classic directory",
                clct_db_path
            ));
        }
        let presence_db_path = path.join("presence.db");
        if !presence_db_path.is_file() {
            return Err(format!(
                "{:?} file was not found in your osu!classic directory",
                presence_db_path
            ));
        }
        let scores_db_path = path.join("scores.db");
        if !scores_db_path.is_file() {
            return Err(format!(
                "{:?} file was not found in your osu!classic directory",
                scores_db_path
            ));
        }
        let osu_db = osu_db::Listing::from_file(&osu_db_path).map_err(|err| {
            format!(
                "{:?} file was deemed unreadable because {:?}",
                osu_db_path, err
            )
        })?;
        let collection_db = osu_db::CollectionList::from_file(&clct_db_path)
            .map_err(|err| format!("{:?} file was deemed unreadable {:?}", clct_db_path, err))?;
        let scores_db = osu_db::ScoreList::from_file(&scores_db_path)
            .map_err(|err| format!("{:?} file was deemed unreadable {:?}", scores_db_path, err))?;
        Ok(Self::new(
            Arc::from(songs_path),
            Arc::from(data_path),
            Arc::from(replays_path),
            Arc::from(osu_db),
            Arc::from(collection_db),
            Arc::from(scores_db),
        ))
    }
}
impl TryFrom<&PathBuf> for Osu50 {
    type Error = String;
    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        if !path.is_dir() {
            return Err(format!("{:?} is not a directory", path));
        }
        let files_path = path.join("files");
        if !files_path.is_dir() {
            return Err(format!(
                "{:?} directory was not found in your osu!lazer directory",
                files_path
            ));
        }
        let client_path = path.join("client.db");
        if !client_path.is_file() {
            return Err(format!(
                "{:?} file was not found in your osu!lazer directory",
                files_path
            ));
        }
        let connection = rusqlite::Connection::open_with_flags(
            client_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|x| format!("{:?}", x))?;
        let mut connection_memory = rusqlite::Connection::open_in_memory().unwrap();
        rusqlite::backup::Backup::new(&connection, &mut connection_memory)
            .unwrap()
            .run_to_completion(100000, std::time::Duration::from_millis(0), None)
            .unwrap();
        Ok(Self::new(
            Arc::new(Osu50HashResolver::new(files_path)),
            Arc::new(connection_memory),
        ))
    }
}

#[derive(Clone, new)]
pub struct OsuBeatmapSet {
    osu: Arc<dyn Osu>,
    bms_id: u64,
}

impl OsuBeatmapSet {
    pub fn worst_rank(&self) -> OsuBeatmapStatus {
        let beatmaps = self.osu.get_beatmapset_maps(self.bms_id);
        if beatmaps.len() == 0 {
            return OsuBeatmapStatus::NotInstalled;
        }
        let mut ranks: Vec<OsuBeatmapStatus> = beatmaps
            .iter()
            .flat_map(|x| {
                vec![
                    x.std_grade(),
                    x.taiko_grade(),
                    x.ctb_grade(),
                    x.mania_grade(),
                ]
            })
            .collect();
        ranks
            .retain(|g| ![OsuBeatmapStatus::NotPlayed, OsuBeatmapStatus::NotInstalled].contains(g));
        ranks.sort();
        *ranks.last().unwrap_or(&OsuBeatmapStatus::NotPlayed)
    }
}

#[derive(Clone, new)]
pub struct OsuBeatmap {
    osu: Arc<dyn Osu>,
    bms_id: u64,
    bm_id: u64,
}

impl OsuBeatmap {
    fn std_grade(&self) -> OsuBeatmapStatus {
        self.osu.get_beatmap_grade_std(self.bms_id, self.bm_id)
    }
    fn taiko_grade(&self) -> OsuBeatmapStatus {
        self.osu.get_beatmap_grade_taiko(self.bms_id, self.bm_id)
    }
    fn ctb_grade(&self) -> OsuBeatmapStatus {
        self.osu.get_beatmap_grade_ctb(self.bms_id, self.bm_id)
    }
    fn mania_grade(&self) -> OsuBeatmapStatus {
        self.osu.get_beatmap_grade_mania(self.bms_id, self.bm_id)
    }
}
