use anyhow::{Result, Error, anyhow};

const COMMENT_DELIM: &str = ";";

// Normalize string to hex string
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
    }
    else {
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
            _ => println!("Unknown instruction: {}", split[0]),
        },
        2 => match split[0] {
            "SYS" => {
                let addr: String = num(split[1], 3).unwrap();
                return Ok(format!("0{}", addr));
            },
            "JP" => {
                let addr: String = num(split[1], 3).unwrap();
                return Ok(format!("1{}", addr));
            },
            "CALL" => {
                let addr: String = num(split[1], 3).unwrap();
                return Ok(format!("2{}", addr));
            },
            _ => println!("Unknown instruction: {}", split[0]),
        },
        3 => match split[0] {
            "SE" => {
                let reg1 = split[1].trim_start_matches("V");
                if reg1.len() != 1 {
                    return Err(anyhow!("Invalid register: {}", reg1));
                }
                if split[2].contains("V") {
                    let reg2 = split[2].trim_start_matches("V");
                    if reg2.len() != 1 {
                        return Err(anyhow!("Invalid register: {}", reg2));
                    }
                    return Ok(format!("5{}{}0", reg1, reg2));
                } else {
                    let nn = num(split[2], 2).unwrap();
                    return Ok(format!("3{}{}", reg1, nn));
                }
            },
            "SNE" => {
                let reg = split[1].trim_start_matches("V");
                if reg.len() != 1 {
                    return Err(anyhow!("Invalid register: {}", reg));
                }
                let nn = num(split[2], 2).unwrap();
                return Ok(format!("4{}{}", reg, nn));
            },
            "LD" => {
                let reg1 = split[1].trim_start_matches("V");
                if reg1.len() != 1 {
                    return Err(anyhow!("Invalid register: {}", reg1));
                }
                if split[2].contains("V") {
                    let reg2 = split[2].trim_start_matches("V");
                    if reg2.len() != 1 {
                        return Err(anyhow!("Invalid register: {}", reg2));
                    }
                    return Ok(format!("8{}{}0", reg1, reg2));
                }
                let nn = num(split[2], 2).unwrap();
                return Ok(format!("6{}{}", reg1, nn));
            },
            "ADD" => {
                let reg1 = split[1].trim_start_matches("V");
                if reg1.len() != 1 {
                    return Err(anyhow!("Invalid register: {}", reg1));
                }
                if split[2].contains("V") {
                    let reg2 = split[2].trim_start_matches("V");
                    if reg2.len() != 1 {
                        return Err(anyhow!("Invalid register: {}", reg2));
                    }
                    return Ok(format!("8{}{}4", reg1, reg2));
                }

                let nn = num(split[2], 2).unwrap();
                return Ok(format!("7{}{}", reg1, nn));
            },

            _ => println!("Unknown instruction: {}", split[0]),
        },
        _ => println!("Unknown instruction: {}", split[0]),
    }
    return Ok("".to_string());
}


fn main() {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    // Read file
    let mut file = std::fs::read_to_string(&args[1]).expect("Unable to read file");
    for (idx, line) in file.lines().enumerate() {
        let d = decode(line).expect("Error on line {idx}. {line}");
        println!(" {:30} | {}", line, d);
    }
}
