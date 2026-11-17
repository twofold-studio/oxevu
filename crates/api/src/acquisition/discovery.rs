use super::*;

#[derive(Clone, Copy, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(as = DiscoverSort)]
pub(crate) enum Sort {
    Popular,
    Rating,
    Newest,
    Oldest,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Search {
    #[serde(default)]
    pub query: String,
    #[serde(default = "movie")]
    pub kind: String,
    #[serde(default = "first_page")]
    pub page: u32,
    pub year: Option<u32>,
    pub genre: Option<u32>,
    pub min_rating: Option<f64>,
    pub language: Option<String>,
    pub sort: Option<Sort>,
}
fn movie() -> String {
    "movie".into()
}
fn first_page() -> u32 {
    1
}

impl Search {
    pub(super) fn provider_query(&self) -> Result<(String, Vec<(&'static str, String)>)> {
        title_key(&self.kind, 1)?;
        if self.query.len() > 200
            || !(1..=20).contains(&self.page)
            || self.year.is_some_and(|n| !(1880..=2100).contains(&n))
            || self.genre.is_some_and(|n| n == 0 || n > 1_000_000)
            || self
                .min_rating
                .is_some_and(|n| !n.is_finite() || !(0.0..=10.0).contains(&n))
            || self
                .language
                .as_ref()
                .is_some_and(|s| s.len() != 2 || !s.bytes().all(|c| c.is_ascii_lowercase()))
        {
            return Err(bad("invalid discovery filters"));
        }
        let browsing = self.query.trim().is_empty();
        if !browsing
            && (self.genre.is_some()
                || self.min_rating.is_some()
                || self.language.is_some()
                || self.sort.is_some())
        {
            return Err(bad(
                "genre, rating, original language and sort apply to browsing; clear the title search first",
            ));
        }
        let mut params = vec![
            ("page", self.page.to_string()),
            ("include_adult", "false".into()),
        ];
        if let Some(year) = self.year {
            params.push((
                if self.kind == "tv" {
                    "first_air_date_year"
                } else {
                    "primary_release_year"
                },
                year.to_string(),
            ));
        }
        let route = if browsing {
            if let Some(genre) = self.genre {
                params.push(("with_genres", genre.to_string()));
            }
            if let Some(rating) = self.min_rating {
                params.push(("vote_average.gte", rating.to_string()));
            }
            if self.min_rating.is_some() || matches!(self.sort, Some(Sort::Rating)) {
                params.push(("vote_count.gte", "100".into()));
            }
            if let Some(language) = &self.language {
                params.push(("with_original_language", language.clone()));
            }
            let sort = match self.sort.unwrap_or(Sort::Popular) {
                Sort::Popular => "popularity.desc",
                Sort::Rating => "vote_average.desc",
                Sort::Newest if self.kind == "tv" => "first_air_date.desc",
                Sort::Oldest if self.kind == "tv" => "first_air_date.asc",
                Sort::Newest => "primary_release_date.desc",
                Sort::Oldest => "primary_release_date.asc",
            };
            params.push(("sort_by", sort.into()));
            format!("discover/{}", self.kind)
        } else {
            params.push(("query", self.query.trim().to_owned()));
            format!("search/{}", self.kind)
        };
        Ok((route, params))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn provider_filters_distinguish_movies_series_and_search() {
        let movie: Search = serde_json::from_value(
            json!({"year":2020,"genre":28,"min_rating":7,"sort":"rating","language":"en"}),
        )
        .unwrap();
        let (route, args) = movie.provider_query().unwrap();
        assert_eq!(route, "discover/movie");
        assert!(args.contains(&("primary_release_year", "2020".into())));
        assert!(args.contains(&("with_genres", "28".into())));
        assert!(args.contains(&("with_original_language", "en".into())));
        let tv: Search =
            serde_json::from_value(json!({"kind":"tv","query":"House","year":2004})).unwrap();
        let (route, args) = tv.provider_query().unwrap();
        assert_eq!(route, "search/tv");
        assert!(args.contains(&("first_air_date_year", "2004".into())));
    }
    #[test]
    fn rejects_invalid_and_unsupported_combinations() {
        for value in [
            json!({"year":0}),
            json!({"genre":0}),
            json!({"min_rating":11}),
            json!({"language":"english"}),
            json!({"page":21}),
            json!({"query":"House","genre":18}),
        ] {
            let query: Search = serde_json::from_value(value).unwrap();
            assert!(query.provider_query().is_err());
        }
    }
}
