#[derive(Debug)]
pub struct Week {
    week_start: String,
    days: Vec<Day>,
}

impl Week {
    pub fn new(week_start: String) -> Self {
        Self {
            week_start,
            days: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Day {
    date: String,
    hours: Vec<Hours>,
}

impl Day {
    pub fn new(date: String) -> Self {
        Self {
            date,
            hours: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Hours {
    time: String,
    duration: f32,
    task: String,
}

impl Hours {
    pub fn new(time: String, duration: f32, task: String) -> Self {
        Self {
            time,
            duration,
            task,
        }
    }
}

pub mod time_usage_parser {
    use std::{
        error::Error,
        fmt::{self, Display},
        fs::File,
        io::{BufRead, BufReader},
        path::Path,
    };

    use anyhow::Result;
    use fancy_regex::Regex;
    use lazy_static::lazy_static;

    use crate::{Day, Hours, Week};

    fn compile_regex(str: &str) -> Regex {
        Regex::new(str).unwrap()
    }

    lazy_static! {
        // Group 1: Weeks start day
        static ref WEEK_REGEX: Regex =
            compile_regex(r"^## (Week +\d\d?\.\d\d?(?:\.\d\d)?)(?: *- *\d\d?\.\d\d?(?:\.\d\d)?)");
        // Group 1: Date
        // Gorup 2: Start Time
        // Group 3: Day range
        static ref DAY_AND_TIME_REGEX: Regex = compile_regex(
            r"(?:(?:(\d\d?\.\d\d?(?!\d*h)) )?(\d\d?:\d\d?))|(?:\|\s+?(\d+\s*?-\s*?\d+)\s+?\|)"
        );
        // Group 1: Total spent time in hours
        static ref HOURS_REGEX: Regex = compile_regex(r"(\d(\.\d*)?)h");
        // Group 1: Start of task string
        // Gourp 2: Task number
        // Group 3: Meeting
        // Group 4: Task same as above keep as is (...)
        static ref TASK_REGEX: Regex =
            compile_regex(r"(^\d+\. .*)|\| *(?:(\d+)\.|(meet)|(\.{3})) *\|");
    }

    #[derive(PartialEq, Debug)]
    enum TaskType {
        TaskString,
        TaskNumber,
        Meeting,
        TaskContinue,
        None,
    }

    #[derive(Debug)]
    pub struct ParseError {
        msg: String,
    }

    impl ParseError {
        fn new(str: &str) -> Self {
            Self {
                msg: str.to_string(),
            }
        }
    }

    impl Display for ParseError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.msg)
        }
    }

    impl Error for ParseError {
        fn description(&self) -> &str {
            &self.msg
        }
    }

    fn get_task(hay: &String) -> (TaskType, String) {
        let Ok(Some(task)) = TASK_REGEX.captures(&hay) else {
            return (TaskType::None, String::new());
        };

        if let Some(taks) = task.get(1) {
            return (TaskType::TaskString, taks.as_str().to_string());
        } else if let Some(taks) = task.get(2) {
            return (TaskType::TaskNumber, taks.as_str().to_string());
        } else if let Some(_) = task.get(3) {
            return (TaskType::Meeting, String::from("Meetting"));
        } else {
            return (TaskType::TaskContinue, String::from("..."));
        }
    }

    fn get_duration(hay: &String) -> f32 {
        let Ok(duration) = HOURS_REGEX.captures(&hay) else {
            return 0.0;
        };

        match duration {
            Some(val) => match val.get(1) {
                Some(val) => val.as_str().parse().unwrap_or(0.0),
                None => 0.0,
            },
            None => 0.0,
        }
    }

    pub fn parse(path: &Path) -> Result<Option<Vec<Week>>> {
        let f = File::open(path)?;
        let buf = BufReader::new(f);
        let mut weeks: Vec<Week> = Vec::new();

        for line in buf.lines() {
            let hay = line?;
            let week = WEEK_REGEX.captures(&hay)?;
            if week.is_some() {
                let caps = week.unwrap();
                weeks.push(Week::new(caps[1].to_string()));
                continue;
            }

            let day_and_time = DAY_AND_TIME_REGEX.captures(&hay)?;
            if day_and_time.is_some() && weeks.len() > 0 {
                let duration = get_duration(&hay);
                let (task_type, task) = get_task(&hay);
                // Should not be possible
                if task_type == TaskType::TaskString {
                    return Err(ParseError::new("Invalid format for task in date table").into());
                }

                let index = weeks.len() - 1;
                let current_week = &mut weeks[index];
                let days_and_times = day_and_time.unwrap();
                if let Some(date) = days_and_times.get(1) {
                    current_week.days.push(Day::new(date.as_str().to_string()))
                } else if let Some(date_range) = days_and_times.get(3) {
                    let mut day = Day::new(date_range.as_str().to_string());
                    day.hours
                        .push(Hours::new(String::from('*'), duration, task));
                    current_week.days.push(day);
                    continue;
                }
                if let Some(time) = days_and_times.get(2) {
                    if let Some(current_day) = current_week.days.last_mut() {
                        current_day.hours.push(Hours::new(
                            time.as_str().to_string(),
                            duration,
                            task,
                        ))
                    }
                }

                continue;
            }

            let task_string = get_task(&hay);
            if task_string.0 == TaskType::TaskString {
                let task = task_string.1;
                'mainLoop: for week in weeks.as_mut_slice() {
                    for day in week.days.as_mut_slice() {
                        for hours in day.hours.as_mut_slice() {
                            let is_task_number = hours.task.parse::<i32>().is_ok();
                            if is_task_number && task.contains(&hours.task) {
                                hours.task = task[3..].to_string();
                                break 'mainLoop;
                            }
                        }
                    }
                }
            }
        }

        if weeks.len() == 0 {
            Ok(None)
        } else {
            println!("{:#?}", weeks);
            Ok(Some(weeks))
        }
    }
}