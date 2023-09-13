use regex::Regex;

fn main() {
    let re = Regex::new(r"Hello (?<name>\w+)!").unwrap();
    let Some(caps) = re.captures("Hello Test!") else {
        println!("no match");
        return;
    };

    println!("Name is {}", &caps["name"]);
    println!("Hello, world!");
}
