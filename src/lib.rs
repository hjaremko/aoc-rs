use log::info;
use reqwest::header::{HeaderMap, COOKIE};
use reqwest::{Client, StatusCode};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use regex::Regex;

#[derive(Debug)]
enum FetchInputError {
    Reason(String),
}

impl Display for FetchInputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let reason = match &self {
            // FetchInputError::NotFound => "Puzzle not live yet or invalid day and year",
            FetchInputError::Reason(reason) => reason,
        };

        write!(f, "Error fetching puzzle input: {}", reason)
    }
}

impl Error for FetchInputError {}

pub struct AocApi {
    cookie: String,
    client: Client,
}

impl AocApi {
    pub fn with_cookie(cookie: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, format!("session={}", cookie).parse().unwrap());

        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()
            .unwrap();

        AocApi {
            cookie: cookie.to_string(),
            client,
        }
    }

    pub fn puzzle(&self, year: &str, day: &str) -> anyhow::Result<Puzzle> {
        Ok(Puzzle::new(self.client.clone(), year, day))
    }
}

pub struct Puzzle {
    client: Client,
    year: String,
    day: String,
    input: Option<PuzzleInput>,
}

impl Puzzle {
    fn new(client: Client, year: &str, day: &str) -> Self {
        Self {
            client,
            year: year.to_string(),
            day: day.to_string(),
            input: None,
        }
    }

    pub fn save_input_to_disk(&self) -> anyhow::Result<()> {
        let input_dir = Path::new("input");
        let input_path = input_dir.join(format! {"{}-{}.txt", &self.year, &self.day});
        if !input_dir.exists() {
            std::fs::create_dir(input_dir)?;
        }

        Ok(std::fs::write(
            input_path,
            self.input.as_ref().unwrap().clone().str,
        )?)
    }

    pub async fn fetch_input(&mut self) -> anyhow::Result<PuzzleInput> {
        let input = self
            .input
            .as_ref()
            .unwrap_or(
                &self
                    .read_input_from_disk("input")
                    .unwrap_or(self.download_input().await?),
            )
            .clone();
        self.input = Some(input);
        Ok(self.input.as_ref().unwrap().clone())
    }

    fn read_input_from_disk<P>(&self, input_dir: P) -> anyhow::Result<PuzzleInput>
        where
            P: AsRef<Path>,
    {
        let input_path = input_dir
            .as_ref()
            .join(format! {"{}-{}.txt", &self.year, &self.day});
        info!("Reading input from {:?}", &input_path);

        PuzzleInput::try_from_disk(&input_path)
    }

    async fn download_input(&self) -> anyhow::Result<PuzzleInput> {
        info!("Fetching input for day {}-{}", &self.day, &self.year);

        let url = format!(
            "https://adventofcode.com/{}/day/{}/input",
            &self.year, &self.day
        );
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(FetchInputError::Reason(response.text().await?).into());
        }

        Ok(PuzzleInput {
            str: response.text().await?,
        })
    }

    pub async fn submit(&self, answer: &str) -> anyhow::Result<AnswerResponse> {
        let params = [("answer", answer.to_string()), ("level", "1".to_string())];
        let url = format!(
            "https://adventofcode.com/{}/day/{}/answer",
            &self.year, &self.day
        );
        let response = self.client.post(&url).form(&params).send().await?;

        if !response.status().is_success() {
            return Err(FetchInputError::Reason(response.text().await?).into());
        }

        let response = response.text().await?;
        if response.contains("That's not the right answer.") {
            let re = Regex::new(r"Please wait (.*) (minute|second)")?;
            if response.contains("please wait 5 minutes") {}

            return Ok(AnswerResponse::WrongAnswer(WaitTime(0)));
        }

        Ok(AnswerResponse::Ok)
    }
}

#[derive(Debug, PartialEq)]
pub struct WaitTime(u32);

// impl TryFrom<dyn Into<String>> for WaitTime
// {
//     type Error = ();
//
//     fn try_from(value: S) -> Result<Self, Self::Error> {
//         Ok(WaitTime(0))
//     }
// }

#[derive(Debug, PartialEq)]
pub enum AnswerResponse {
    Ok,
    WrongAnswer(WaitTime),
}

#[derive(Clone)]
pub struct PuzzleInput {
    str: String,
}

impl PuzzleInput {
    pub fn try_from_disk<P>(path: &P) -> anyhow::Result<Self>
        where
            P: AsRef<Path>,
    {
        let input = std::fs::read_to_string(path.as_ref())?;
        Ok(Self { str: input })
    }
}

impl Display for PuzzleInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn answer_regex_one_minute() {
        let re = Regex::new(r"Please wait (?P<time>.*) (?P<unit>minute|second)").unwrap();
        let webpage = r#""<!DOCTYPE html>\n<html lang=\"en-us\">\n<head>\n<meta charset=\"utf-8\"/>\n<title>Day 1 - Advent of Code 2017</title>\n<!--[if lt IE 9]><script src=\"/static/html5.js\"></script><![endif]-->\n<link href='//fonts.googleapis.com/css?family=Source+Code+Pro:300&subset=latin,latin-ext' rel='stylesheet' type='text/css'/>\n<link rel=\"stylesheet\" type=\"text/css\" href=\"/static/style.css?26\"/>\n<link rel=\"stylesheet alternate\" type=\"text/css\" href=\"/static/highcontrast.css?0\" title=\"High Contrast\"/>\n<link rel=\"shortcut icon\" href=\"/favicon.png\"/>\n</head><!--\n\n\n\n\nOh, hello!  Funny seeing you here.\n\nI appreciate your enthusiasm, but you aren't going to find much down here.\nThere certainly aren't clues to any of the puzzles.  The best surprises don't\neven appear in the source until you unlock them for real.\n\nPlease be careful with automated requests; I'm not a massive company, and I can\nonly take so much traffic.  Please be considerate so that everyone gets to play.\n\nIf you're curious about how Advent of Code works, it's running on some custom\nPerl code. Other than a few integrations (auth, analytics, social media), I\nbuilt the whole thing myself, including the design, animations, prose, and all\nof the puzzles.\n\nThe puzzles are most of the work; preparing a new calendar and a new set of\npuzzles each year takes all of my free time for 4-5 months. A lot of effort\nwent into building this thing - I hope you're enjoying playing it as much as I\nenjoyed making it for you!\n\nIf you'd like to hang out, I'm @ericwastl on Twitter.\n\n- Eric Wastl\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n-->\n<body>\n<header><div><h1 class=\"title-global\"><a href=\"/\">Advent of Code</a></h1><nav><ul><li><a href=\"/2017/about\">[About]</a></li><li><a href=\"/2017/events\">[Events]</a></li><li><a href=\"https://teespring.com/stores/advent-of-code\" target=\"_blank\">[Shop]</a></li><li><a href=\"/2017/settings\">[Settings]</a></li><li><a href=\"/2017/auth/logout\">[Log Out]</a></li></ul></nav><div class=\"user\">hjaremko</div></div><div><h1 class=\"title-event\">&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;<span class=\"title-event-wrap\">//</span><a href=\"/2017\">2017</a><span class=\"title-event-wrap\"></span></h1><nav><ul><li><a href=\"/2017\">[Calendar]</a></li><li><a href=\"/2017/support\">[AoC++]</a></li><li><a href=\"/2017/sponsors\">[Sponsors]</a></li><li><a href=\"/2017/leaderboard\">[Leaderboard]</a></li><li><a href=\"/2017/stats\">[Stats]</a></li></ul></nav></div></header>\n\n<div id=\"sidebar\">\n<div id=\"sponsor\"><div class=\"quiet\">Our <a href=\"/2017/sponsors\">sponsors</a> help make Advent of Code possible:</div><div class=\"sponsor\"><a href=\"http://smartystreets.com/aoc\" target=\"_blank\" onclick=\"if(ga)ga('send','event','sponsor','sidebar',this.href);\" rel=\"noopener\">SmartyStreets</a> - U2VuZGluZyBDaH Jpc3RtYXMgY2Fy ZHMgdG8gYmFkIG FkZHJlc3Nlcz8K</div></div>\n</div><!--/sidebar-->\n\n<main>\n<article><p>That's not the right answer.  If you're stuck, make sure you're using the full input data; there are also some general tips on the <a href=\"/2017/about\">about page</a>, or you can ask for hints on the <a href=\"https://www.reddit.com/r/adventofcode/\" target=\"_blank\">subreddit</a>.  Please wait one minute before trying again. (You guessed <span style=\"white-space:nowrap;\"><code>32</code>.)</span> <a href=\"/2017/day/1\">[Return to Day 1]</a></p></article>\n</main>\n\n<!-- ga -->\n<script>\n(function(i,s,o,g,r,a,m){i['GoogleAnalyticsObject']=r;i[r]=i[r]||function(){\n(i[r].q=i[r].q||[]).push(arguments)},i[r].l=1*new Date();a=s.createElement(o),\nm=s.getElementsByTagName(o)[0];a.async=1;a.src=g;m.parentNode.insertBefore(a,m)\n})(window,document,'script','//www.google-analytics.com/analytics.js','ga');\nga('create', 'UA-69522494-1', 'auto');\nga('set', 'anonymizeIp', true);\nga('send', 'pageview');\n</script>\n<!-- /ga -->\n</body>\n</html>""#;

        for cap in re.captures_iter(webpage) {
            assert_eq!(cap.name("time").unwrap().as_str(), "one");
            assert_eq!(cap.name("unit").unwrap().as_str(), "minute");
        }
    }

    #[test]
    fn answer_regex_5_minutes() {
        let re = Regex::new(r"Please wait (?P<time>.*) (?P<unit>minute|second)").unwrap();
        let webpage = r#""<!DOCTYPE html>\n<html lang=\"en-us\">\n<head>\n<meta charset=\"utf-8\"/>\n<title>Day 1 - Advent of Code 2017</title>\n<!--[if lt IE 9]><script src=\"/static/html5.js\"></script><![endif]-->\n<link href='//fonts.googleapis.com/css?family=Source+Code+Pro:300&subset=latin,latin-ext' rel='stylesheet' type='text/css'/>\n<link rel=\"stylesheet\" type=\"text/css\" href=\"/static/style.css?26\"/>\n<link rel=\"stylesheet alternate\" type=\"text/css\" href=\"/static/highcontrast.css?0\" title=\"High Contrast\"/>\n<link rel=\"shortcut icon\" href=\"/favicon.png\"/>\n</head><!--\n\n\n\n\nOh, hello!  Funny seeing you here.\n\nI appreciate your enthusiasm, but you aren't going to find much down here.\nThere certainly aren't clues to any of the puzzles.  The best surprises don't\neven appear in the source until you unlock them for real.\n\nPlease be careful with automated requests; I'm not a massive company, and I can\nonly take so much traffic.  Please be considerate so that everyone gets to play.\n\nIf you're curious about how Advent of Code works, it's running on some custom\nPerl code. Other than a few integrations (auth, analytics, social media), I\nbuilt the whole thing myself, including the design, animations, prose, and all\nof the puzzles.\n\nThe puzzles are most of the work; preparing a new calendar and a new set of\npuzzles each year takes all of my free time for 4-5 months. A lot of effort\nwent into building this thing - I hope you're enjoying playing it as much as I\nenjoyed making it for you!\n\nIf you'd like to hang out, I'm @ericwastl on Twitter.\n\n- Eric Wastl\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n-->\n<body>\n<header><div><h1 class=\"title-global\"><a href=\"/\">Advent of Code</a></h1><nav><ul><li><a href=\"/2017/about\">[About]</a></li><li><a href=\"/2017/events\">[Events]</a></li><li><a href=\"https://teespring.com/stores/advent-of-code\" target=\"_blank\">[Shop]</a></li><li><a href=\"/2017/settings\">[Settings]</a></li><li><a href=\"/2017/auth/logout\">[Log Out]</a></li></ul></nav><div class=\"user\">hjaremko</div></div><div><h1 class=\"title-event\">&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;<span class=\"title-event-wrap\">//</span><a href=\"/2017\">2017</a><span class=\"title-event-wrap\"></span></h1><nav><ul><li><a href=\"/2017\">[Calendar]</a></li><li><a href=\"/2017/support\">[AoC++]</a></li><li><a href=\"/2017/sponsors\">[Sponsors]</a></li><li><a href=\"/2017/leaderboard\">[Leaderboard]</a></li><li><a href=\"/2017/stats\">[Stats]</a></li></ul></nav></div></header>\n\n<div id=\"sidebar\">\n<div id=\"sponsor\"><div class=\"quiet\">Our <a href=\"/2017/sponsors\">sponsors</a> help make Advent of Code possible:</div><div class=\"sponsor\"><a href=\"http://smartystreets.com/aoc\" target=\"_blank\" onclick=\"if(ga)ga('send','event','sponsor','sidebar',this.href);\" rel=\"noopener\">SmartyStreets</a> - U2VuZGluZyBDaH Jpc3RtYXMgY2Fy ZHMgdG8gYmFkIG FkZHJlc3Nlcz8K</div></div>\n</div><!--/sidebar-->\n\n<main>\n<article><p>That's not the right answer.  If you're stuck, make sure you're using the full input data; there are also some general tips on the <a href=\"/2017/about\">about page</a>, or you can ask for hints on the <a href=\"https://www.reddit.com/r/adventofcode/\" target=\"_blank\">subreddit</a>.  Please wait 5 minutes before trying again. (You guessed <span style=\"white-space:nowrap;\"><code>32</code>.)</span> <a href=\"/2017/day/1\">[Return to Day 1]</a></p></article>\n</main>\n\n<!-- ga -->\n<script>\n(function(i,s,o,g,r,a,m){i['GoogleAnalyticsObject']=r;i[r]=i[r]||function(){\n(i[r].q=i[r].q||[]).push(arguments)},i[r].l=1*new Date();a=s.createElement(o),\nm=s.getElementsByTagName(o)[0];a.async=1;a.src=g;m.parentNode.insertBefore(a,m)\n})(window,document,'script','//www.google-analytics.com/analytics.js','ga');\nga('create', 'UA-69522494-1', 'auto');\nga('set', 'anonymizeIp', true);\nga('send', 'pageview');\n</script>\n<!-- /ga -->\n</body>\n</html>""#;

        for cap in re.captures_iter(webpage) {
            assert_eq!(cap.name("time").unwrap().as_str(), "5");
            assert_eq!(cap.name("unit").unwrap().as_str(), "minute");
        }
    }

    #[test]
    fn wait_time_from_one_minute()
    {
        assert_eq!(WaitTime::try_from("one minute"), WaitTime(60));
    }


    #[test]
    fn wait_time_from_5_minutes()
    {
        assert_eq!(WaitTime::try_from("5 minutes"), WaitTime(60 * 5));
    }

    #[test]
    fn wait_time_from_1_second()
    {
        assert_eq!(WaitTime::try_from("one second"), WaitTime(1));
    }

    #[test]
    fn wait_time_from_2_seconds()
    {
        assert_eq!(WaitTime::try_from("2 seconds"), WaitTime(2));
    }

    #[test]
    fn wait_time_from_invalid()
    {
        assert!(WaitTime::try_from("invalid").is_err());
    }

    #[test]
    fn wrong_answer() {
        let api = AocApi::with_cookie("53616c7465645f5f8d6c2aaea366c1208a149e39028e06832be00347ad2e434b759ba87cf4c44b8936f700d8c8588570");
        let puzzle = api.puzzle("2017", "1").unwrap();
        let response = aw!(puzzle.submit("invalid")).unwrap();
        assert_eq!(response, AnswerResponse::WrongAnswer(WaitTime(0)));
    }

    /*
    "<!DOCTYPE html>\n<html lang=\"en-us\">\n<head>\n<meta charset=\"utf-8\"/>\n<title>Day 1 - Advent of Code 2017</title>\n<!--[if lt IE 9]><script src=\"/static/html5.js\"></script><![endif]-->\n<link href='//fonts.googleapis.com/css?family=Source+Code+Pro:300&subset=latin,latin-ext' rel='stylesheet' type='text/css'/>\n<link rel=\"stylesheet\" type=\"text/css\" href=\"/static/style.css?26\"/>\n<link rel=\"stylesheet alternate\" type=\"text/css\" href=\"/static/highcontrast.css?0\" title=\"High Contrast\"/>\n<link rel=\"shortcut icon\" href=\"/favicon.png\"/>\n</head><!--\n\n\n\n\nOh, hello!  Funny seeing you here.\n\nI appreciate your enthusiasm, but you aren't going to find much down here.\nThere certainly aren't clues to any of the puzzles.  The best surprises don't\neven appear in the source until you unlock them for real.\n\nPlease be careful with automated requests; I'm not a massive company, and I can\nonly take so much traffic.  Please be considerate so that everyone gets to play.\n\nIf you're curious about how Advent of Code works, it's running on some custom\nPerl code. Other than a few integrations (auth, analytics, social media), I\nbuilt the whole thing myself, including the design, animations, prose, and all\nof the puzzles.\n\nThe puzzles are most of the work; preparing a new calendar and a new set of\npuzzles each year takes all of my free time for 4-5 months. A lot of effort\nwent into building this thing - I hope you're enjoying playing it as much as I\nenjoyed making it for you!\n\nIf you'd like to hang out, I'm @ericwastl on Twitter.\n\n- Eric Wastl\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n-->\n<body>\n<header><div><h1 class=\"title-global\"><a href=\"/\">Advent of Code</a></h1><nav><ul><li><a href=\"/2017/about\">[About]</a></li><li><a href=\"/2017/events\">[Events]</a></li><li><a href=\"https://teespring.com/stores/advent-of-code\" target=\"_blank\">[Shop]</a></li><li><a href=\"/2017/settings\">[Settings]</a></li><li><a href=\"/2017/auth/logout\">[Log Out]</a></li></ul></nav><div class=\"user\">hjaremko</div></div><div><h1 class=\"title-event\">&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;<span class=\"title-event-wrap\">//</span><a href=\"/2017\">2017</a><span class=\"title-event-wrap\"></span></h1><nav><ul><li><a href=\"/2017\">[Calendar]</a></li><li><a href=\"/2017/support\">[AoC++]</a></li><li><a href=\"/2017/sponsors\">[Sponsors]</a></li><li><a href=\"/2017/leaderboard\">[Leaderboard]</a></li><li><a href=\"/2017/stats\">[Stats]</a></li></ul></nav></div></header>\n\n<div id=\"sidebar\">\n<div id=\"sponsor\"><div class=\"quiet\">Our <a href=\"/2017/sponsors\">sponsors</a> help make Advent of Code possible:</div><div class=\"sponsor\"><a href=\"http://smartystreets.com/aoc\" target=\"_blank\" onclick=\"if(ga)ga('send','event','sponsor','sidebar',this.href);\" rel=\"noopener\">SmartyStreets</a> - U2VuZGluZyBDaH Jpc3RtYXMgY2Fy ZHMgdG8gYmFkIG FkZHJlc3Nlcz8K</div></div>\n</div><!--/sidebar-->\n\n<main>\n<article><p>That's not the right answer.  If you're stuck, make sure you're using the full input data; there are also some general tips on the <a href=\"/2017/about\">about page</a>, or you can ask for hints on the <a href=\"https://www.reddit.com/r/adventofcode/\" target=\"_blank\">subreddit</a>.  Please wait one minute before trying again. (You guessed <span style=\"white-space:nowrap;\"><code>32</code>.)</span> <a href=\"/2017/day/1\">[Return to Day 1]</a></p></article>\n</main>\n\n<!-- ga -->\n<script>\n(function(i,s,o,g,r,a,m){i['GoogleAnalyticsObject']=r;i[r]=i[r]||function(){\n(i[r].q=i[r].q||[]).push(arguments)},i[r].l=1*new Date();a=s.createElement(o),\nm=s.getElementsByTagName(o)[0];a.async=1;a.src=g;m.parentNode.insertBefore(a,m)\n})(window,document,'script','//www.google-analytics.com/analytics.js','ga');\nga('create', 'UA-69522494-1', 'auto');\nga('set', 'anonymizeIp', true);\nga('send', 'pageview');\n</script>\n<!-- /ga -->\n</body>\n</html>"

    let aoc = Aoc::new("cookie");

    let puzzle = aoc.fetch_puzzle();
    puzzle.save_input("input");
    puzzle.submit_answer("42");
    puzzle.stars();
    puzzle.content();

    let leaderboard = aoc.fetch_leaderboard(); //private //self

     */

    #[test]
    fn get_input_invalid_cookie() {
        let api = AocApi::with_cookie("53616c7465645f5f8d6c2aaea366c1208a149e39028e06832be00347ad2e434b759ba87cf4c44b8936f700d8c8588571");
        // let input = aw!(api.fetch_puzzle_input("2021", "1"));
        // assert!(input.is_err(), "{:?}", input);
    }

    #[test]
    fn get_input_invalid_year() {
        let api = AocApi::with_cookie("53616c7465645f5f8d6c2aaea366c1208a149e39028e06832be00347ad2e434b759ba87cf4c44b8936f700d8c8588570");
        // let input = aw!(api.fetch_puzzle_input("2022", "18"));
        // assert!(input.is_err(), "{:?}", input);
    }

    #[test]
    fn get_input() {
        let api = AocApi::with_cookie("53616c7465645f5f8d6c2aaea366c1208a149e39028e06832be00347ad2e434b759ba87cf4c44b8936f700d8c8588570");
        /*
                let input = aw!(api.fetch_puzzle_input("2017", "18")).unwrap();
                assert_eq!(
                    input,
                    r#"set i 31
        set a 1
        mul p 17
        jgz p p
        mul a 2
        add i -1
        jgz i -2
        add a -1
        set i 127
        set p 735
        mul p 8505
        mod p a
        mul p 129749
        add p 12345
        mod p a
        set b p
        mod b 10000
        snd b
        add i -1
        jgz i -9
        jgz a 3
        rcv b
        jgz b -1
        set f 0
        set i 126
        rcv a
        rcv b
        set p a
        mul p -1
        add p b
        jgz p 4
        snd a
        set a b
        jgz 1 3
        snd b
        set f 1
        add i -1
        jgz i -11
        snd a
        jgz f -16
        jgz a -19
        "#
                );

                 */
    }
}
