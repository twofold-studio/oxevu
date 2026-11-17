use super::*;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct Episode {
    pub season: u32,
    pub episode: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Scope {
    pub mode: String,
    pub seasons: Vec<u32>,
    pub episodes: Vec<Episode>,
    pub cutoff: Option<String>,
}
impl Default for Scope {
    fn default() -> Self {
        Self {
            mode: "all".into(),
            seasons: Vec::new(),
            episodes: Vec::new(),
            cutoff: None,
        }
    }
}
impl Scope {
    pub fn normalize(&mut self, kind: &str, today: &str) -> Result<()> {
        if !matches!(
            self.mode.as_str(),
            "all" | "existing" | "future" | "seasons" | "episodes"
        ) || self.seasons.len() > 100
            || self.episodes.len() > 1000
            || self.seasons.iter().any(|s| *s > 999)
            || self
                .episodes
                .iter()
                .any(|e| e.season > 999 || e.episode > 9999)
            || (kind == "movie" && self.mode != "all")
        {
            return Err(bad("invalid request scope"));
        }
        self.seasons.sort_unstable();
        self.seasons.dedup();
        self.episodes.sort();
        self.episodes.dedup();
        if (self.mode == "seasons" && self.seasons.is_empty())
            || (self.mode == "episodes" && self.episodes.is_empty())
        {
            return Err(bad("select at least one season or episode"));
        }
        if self.mode != "seasons" {
            self.seasons.clear();
        }
        if self.mode != "episodes" {
            self.episodes.clear();
        }
        self.cutoff = matches!(self.mode.as_str(), "existing" | "future").then(|| today.to_owned());
        Ok(())
    }

    pub fn covers(&self, season: u32, episode: u32, air_date: Option<&str>) -> bool {
        match self.mode.as_str() {
            "all" => season > 0,
            "existing" => {
                season > 0
                    && air_date
                        .zip(self.cutoff.as_deref())
                        .is_some_and(|(date, cutoff)| date <= cutoff)
            }
            "future" => {
                season > 0
                    && air_date
                        .zip(self.cutoff.as_deref())
                        .is_some_and(|(date, cutoff)| date > cutoff)
            }
            "seasons" => self.seasons.contains(&season),
            "episodes" => self.episodes.contains(&Episode { season, episode }),
            _ => false,
        }
    }
}
