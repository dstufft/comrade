use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref LINE_RE1: Regex = Regex::new(r"^\[([^]]+)\] (.+?)\r?\n$").unwrap();
}

pub fn parse(line: &str) -> Option<(&str, &str)> {
    LINE_RE1
        .captures(line)
        .map(|caps| (caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses() {
        let input = "[Fri Apr 29 17:25:01 2022] Mrshaman healed Mrswarrior over time for 500 hit points by Prophet's Gift of the Ruchu.\r\n";
        let result = parse(input).unwrap();

        assert_eq!(result, ("Fri Apr 29 17:25:01 2022", "Mrshaman healed Mrswarrior over time for 500 hit points by Prophet's Gift of the Ruchu."));
    }
}
