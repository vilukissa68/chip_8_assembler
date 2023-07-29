use anyhow::{Result, Error, anyhow};

const COMMENT_DELIM: &str = ";";

// Make sure register is 0-F
fn register(input: &str) -> Result<String> {
    let mut s: String = input.to_uppercase();
    if s.contains("V") {
        s = s.trim_start_matches("V").to_string();
    } else {
        return Err(anyhow!("Invalid register: {}", input));
    }
    s = format!("{:X}", u8::from_str_radix(&s, 16)?);
    return Ok(s);

}

// Normalize string to hex string of given length
fn num(input: &str, length: u64) -> Result<String> {
    let mut s: String = input.to_uppercase();
    if s.contains("X") {
        s = s.trim_start_matches("0X").to_string();
        let len = s.chars().count() - length as usize;
        s.drain(0..len);
    }
    else if s.contains("B") {
        s = s.trim_start_matches("0B").to_string();
        s = format!("{:X}", u8::from_str_radix(&s, 2)?);
    } else if s.contains("$") {
        s = s.trim_start_matches("$").to_string();
    } else if s.contains("0") {
        s = s.trim_start_matches("0").to_string();
        s = format!("{:X}", u8::from_str_radix(&s, 8)?);
    } else {
        return Err(anyhow!("Invalid number: {}", input));
    }
    if s.len() < length as usize {
        // Pad by pushing zeros to the front
        let mut pad = String::new();
        for _ in 0..(length as usize - s.len()) {
            pad.push('0');
        }
        s = format!("{}{}", pad, s);
    }
    return Ok(s);

}

fn decode(line: &str) -> Result<String> {
    // Remove comments
    let line = &line[0..line.find(COMMENT_DELIM).unwrap_or(line.len())].trim();
    let line = line.replace(",", " ");
    if line.len() == 0 {
        return Ok("".to_string());
    }

    // Check for label
    if line.contains(":") {
        return Ok("".to_string());
    }

    let mut split = line.split_whitespace().collect::<Vec<&str>>();
    match split.len() {
        1 => match split[0] {
            "CLS" => return Ok("00E0".to_string()),
            "RET" => return Ok("00EE".to_string()),
            _ => return Err(anyhow!("Unknown instruction: {}", split[0])),
        },
        2 => match split[0] {
            "SYS" => {
                return Ok(format!("0{}", num(split[1], 3)?));
            },
            "JP" => {
                return Ok(format!("1{}", num(split[1], 3)?));
            },
            "CALL" => {
                return Ok(format!("2{}", num(split[1], 3)?));
            },
            "SHR" => return Ok(format!("8{}06", register(split[1])?)),
            "SHL" => return Ok(format!("8{}0E", register(split[1])?)),
            "SKP" => return Ok(format!("E{}9E", register(split[1])?)),
            "SKNP" => return Ok(format!("E{}A1", register(split[1])?)),
            _ => return Err(anyhow!("Unknown instruction: {}", split[0])),
        },
        3 => match split[0] {
            "SE" => {
                if split[2].contains("V") {
                    return Ok(format!("5{}{}0", register(split[1])?, register(split[2])?));
                }
                return Ok(format!("3{}{}", register(split[1])?, num(split[2], 2)?));
            },
            "SNE" => {
                if split[2].contains("V") {
                    return Ok(format!("9{}{}0", register(split[1])?, register(split[2])?));
                }
                return Ok(format!("4{}{}", register(split[1])?, num(split[2], 2)?));
            },
            "LD" => {
                if split[1] == "I" {
                    if split[2].contains("V") {
                        return Ok(format!("F{}55", register(split[2])?));
                    }
                    return Ok(format!("A{}", num(split[2], 3)?));
                }
                else if split[1] == "B" {
                    return Ok(format!("F{}33", register(split[2])?));
                } else if split[1] == "F" {
                    return Ok(format!("F{}29", register(split[2])?));
                } else if split[1] == "ST" {
                    return Ok(format!("F{}18", register(split[2])?));
                } else if split[1] == "DT" {
                    return Ok(format!("F{}15", register(split[2])?));
                } else if split[2] == "DT" {
                    return Ok(format!("F{}07", register(split[1])?));
                } else if split[2] == "K" {
                    return Ok(format!("F{}0A", register(split[1])?));
                } else if split[2] == "I" {
                    return Ok(format!("F{}65", register(split[1])?));
                } else if split[2].contains("V") {
                    return Ok(format!("8{}{}0", register(split[1])?, register(split[2])?));
                }
                return Ok(format!("6{}{}", register(split[1])?, num(split[2], 2)?));
            },
            "ADD" => {
                if split[2].contains("V") {
                    return Ok(format!("8{}{}4", register(split[1])?, register(split[2])?));
                }
                return Ok(format!("7{}{}", register(split[1])?, num(split[2], 2)?));
            },
            "OR" => return Ok(format!("8{}{}1", register(split[1])?, register(split[2])?)),
            "AND" => return Ok(format!("8{}{}2", register(split[1])?, register(split[2])?)),
            "XOR" => return Ok(format!("8{}{}3", register(split[1])?, register(split[2])?)),
            "SUB" => return Ok(format!("8{}{}5", register(split[1])?, register(split[2])?)),
            "SUBN" => return Ok(format!("8{}{}7", register(split[1])?, register(split[2])?)),
            "JP" => {
                if split[1].contains("V0") {
                    return Ok(format!("B{}", num(split[2], 3)?))
                }
                return Err(anyhow!("{} not available for JP use V0", split[1]));
            }
            "RND" => return Ok(format!("C{}{}", register(split[1])?, num(split[2],2)?)),

            _ => return Err(anyhow!("Unknown instruction: {}", split[0])),
        },
        4 => match split[0] {
            // TODO: Check limit for DRW
            "DRW" => return Ok(format!("D{}{}{}", register(split[1])?, register(split[2])?, num(split[3], 1)?)),
            _ => return Err(anyhow!("Unknown instruction: {}", split[0])),
        }
        _ => return Err(anyhow!("Unknown instruction: {}", split[0])),
    }
}


fn main() {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    // Read file
    let mut file = std::fs::read_to_string(&args[1]).expect("Unable to read file");
    for (idx, line) in file.lines().enumerate() {
        let d = decode(line);
        if d.is_err() {
            let err_msg = format!("Error on line {} ({})", idx, line);
            println!("{} | {}", err_msg, d.unwrap_err());
            return;
        }
        let d = d.unwrap();
        println!(" {:30} | {}", line, d);
    }
}
