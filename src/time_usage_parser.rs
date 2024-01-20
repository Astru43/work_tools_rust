use std::{
    error::Error,
    fmt::{self, Display},
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use fancy_regex::Regex;
use lazy_static::lazy_static;

#[derive(Debug)]
pub struct Week {
    pub week_start: String,
    pub days: Vec<Day>,
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
    pub date: String,
    pub hours: Vec<Hours>,
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
    pub time: String,
    pub duration: f32,
    pub task: String,
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

fn compile_regex(str: &str) -> Regex {
    Regex::new(str).unwrap()
}

lazy_static! {
    // Group 1: Weeks start day
    static ref WEEK_REGEX: Regex =
        compile_regex(r"^## (Week \d\d?\.\d\d?(?:\.\d\d)?)(?: *- *\d\d?\.\d\d?(?:\.\d\d)?)");
    // Group 1: Date
    // Gorup 2: Start Time
    // Group 3: Day range
    static ref DAY_AND_TIME_REGEX: Regex = compile_regex(
        r"(?:\|\s*?(?:(\d\d?\.\d\d?) +)?(\d\d?:\d\d)\s*?\|)|(?:\|\s*?(\d+\s*?-\s*?\d+)\s*?\|)"
    );
    // Group 1: Total spent time in hours
    static ref HOURS_REGEX: Regex = compile_regex(r"(\d+(\.\d*)?)h");
    // Group 1: Start of task string
    // Gourp 2: Task number
    // Group 3: Meeting
    // Group 4: Task same as above keep as is (...)
    static ref TASK_REGEX: Regex =
        compile_regex(r"^(\d+\. .*)$|(?:\|[ \t]*(?:(\d+)\.|(meet)|(\.{3}))[ \t]*\|)");
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

fn get_task(hay: &str) -> (TaskType, String) {
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

fn get_duration(hay: &str) -> f32 {
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

pub fn parse(path: &Path) -> anyhow::Result<Option<Vec<Week>>> {
    let f = match File::open(path) {
        Ok(file) => file,
        Err(..) => {
            return Err(
                ParseError::new(format!("File {} not found", path.display()).as_str()).into(),
            )
        }
    };
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
                    current_day
                        .hours
                        .push(Hours::new(time.as_str().to_string(), duration, task))
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
        Ok(Some(weeks))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test if group has given string value
    ///
    /// ### Params
    /// `result` result from regex
    ///
    /// `group` group number
    ///
    /// `result_str` str to test against
    ///
    /// ### Usage
    /// ```
    /// let result = regex.captures(&test_str)
    /// test_success!(result, group, result_str)
    /// test_success!(result, 1, "test")
    /// ```
    macro_rules! test_for_success {
            ($result:ident, $group:tt, $test:tt $(, $case:literal )?) => {
                assert!(
                    $result.is_some()
                    $(, $case)?
                );
                assert_eq!(
                    $result.unwrap().get($group).unwrap().as_str(),
                    $test
                    $(, $case)?
                );
            };
        }

    macro_rules! test_for_fail {
            ($result:ident $(, $case:literal )?) => {
                assert!(
                    $result.is_none()
                    $(, $case)?
                );
            };
            ($result:ident, $group:tt $(, $case:literal )?) => {
                assert!(
                    $result.is_some()
                    $(, $case)?
                );
                assert!(
                    $result.unwrap().get($group).is_none()
                    $(, $case)?
                );
            };
        }

    #[test]
    fn week_regex() -> anyhow::Result<()> {
        // Case 1: Correct
        let result = WEEK_REGEX.captures("## Week 25.5 - 30.5")?;
        test_for_success!(result, 1, "Week 25.5", "Case 1");

        // Case 2: Correct
        let result = WEEK_REGEX.captures("## Week 5.5.23 - 30.5.23")?;
        test_for_success!(result, 1, "Week 5.5.23", "Case 2");

        // Case 3: Correct
        let result = WEEK_REGEX.captures("## Week 25.05.23 - 30.5")?;
        test_for_success!(result, 1, "Week 25.05.23", "Case 3");

        // Case 4: Fail
        let result = WEEK_REGEX.captures("## Week 2.5")?;
        test_for_fail!(result, "Case 4");

        // Case 5: Fail
        let result = WEEK_REGEX.captures("## Week 25.5.23")?;
        test_for_fail!(result, "Case 5");

        // Case 6: Fail
        let result = WEEK_REGEX.captures("## Week 5.05.23")?;
        test_for_fail!(result, "Case 6");

        Ok(())
    }

    macro_rules! get_as_ref {
        ($var:tt, $call:expr) => {
            let result = $call;
            let $var = result.as_ref();
        };
    }

    #[test]
    fn day_and_time_regex() -> anyhow::Result<()> {
        // Case 1: Correct syntax
        get_as_ref!(
            result,
            DAY_AND_TIME_REGEX.captures("| 19.5 18:50 | 1h | 2. |")?
        );
        test_for_success!(result, 1, "19.5", "Case 1");
        test_for_success!(result, 2, "18:50", "Case 1");
        test_for_fail!(result, 3, "Case 1");

        // Case 2: Correct syntax
        get_as_ref!(
            result,
            DAY_AND_TIME_REGEX.captures("| 20:00 | 5.25h | 3. |")?
        );
        test_for_success!(result, 2, "20:00", "Case 2");
        test_for_fail!(result, 1, "Case 2");
        test_for_fail!(result, 3, "Case 2");

        // Case 3: Correct syntax
        get_as_ref!(result, DAY_AND_TIME_REGEX.captures("| 10 - 30 |  |")?);
        test_for_success!(result, 3, "10 - 30", "Case 3");
        test_for_fail!(result, 1, "Case 3");
        test_for_fail!(result, 2, "Case 3");

        // Case 4: Incorrect syntax
        get_as_ref!(result, DAY_AND_TIME_REGEX.captures("|  |  |  |")?);
        test_for_fail!(result, "Case 4");

        // Case 5: Incorrect syntax
        // Cant contain extra letters
        get_as_ref!(result, DAY_AND_TIME_REGEX.captures("| 12.5h 18:30 | | |")?);
        test_for_fail!(result, "Case 5");

        Ok(())
    }

    #[test]
    fn hours_regex() -> anyhow::Result<()> {
        let result = HOURS_REGEX.captures("| 1h |")?;
        test_for_success!(result, 1, "1", "Case 1");

        let result = HOURS_REGEX.captures("|1.5h|")?;
        test_for_success!(result, 1, "1.5", "Case 2");

        let result = HOURS_REGEX.captures("|    1.25h|")?;
        test_for_success!(result, 1, "1.25", "Case 3");

        let result = HOURS_REGEX.captures("| h |")?;
        test_for_fail!(result, "Case 4");

        let result = HOURS_REGEX.captures("")?;
        test_for_fail!(result, "Case 5");

        let result = HOURS_REGEX.captures("100h")?;
        test_for_success!(result, 1, "100", "Case 6");

        let result = HOURS_REGEX.captures("10.25h")?;
        test_for_success!(result, 1, "10.25", "Case 7");

        Ok(())
    }

    #[test]
    fn task_regex() -> anyhow::Result<()> {
        let result = TASK_REGEX.captures("1. test")?;
        test_for_success!(result, 1, "1. test", "Case 1");

        let result = TASK_REGEX.captures("| 92. |")?;
        test_for_success!(result, 2, "92", "Case 2");

        let result = TASK_REGEX.captures("| meet|")?;
        test_for_success!(result, 3, "meet", "Case 3");

        let result = TASK_REGEX.captures("|  ...  |")?;
        test_for_success!(result, 4, "...", "Case 4");

        let result = TASK_REGEX.captures("...")?;
        test_for_fail!(result, "Case 5");

        let result = TASK_REGEX.captures("| 1. ")?;
        test_for_fail!(result, "Case 6");

        let result = TASK_REGEX.captures("Some random text")?;
        test_for_fail!(result, "Case 7");

        Ok(())
    }
}
