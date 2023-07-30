use anyhow::{Result, anyhow};

const COMMENT_DELIM: &str = ";";
const LABEL_DELIM: &str = ":";
//const MAX_MEMORY: usize = 4096;

static INSTRCTIONS: &'static [&'static str] = &["CLS", "RET", "SYS", "JP", "CALL", "SE", "SNE", "LD", "ADD", "OR", "AND", "XOR", "SUB", "SHR", "SUBN", "SHL", "RND", "DRW", "SKP", "SKNP"];

#[derive(Debug)]
struct JumpLabel {
    name: String, // Name in source code
    address: u16, // Address in memory
    address_str: String, // Address in string form
    size: u16, // Size in bytes (2 per instruction)
    instructions: Vec<String>,
}

#[derive(Debug)]
struct AliasLabel {
    from: String, // Change from this
    to: String, // To this
}



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
        s = format!("{:X}", u8::from_str_radix(&s, 10)?);
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

fn preprocess(file: String) -> (Vec<String>, Vec<AliasLabel>, Vec<JumpLabel>) {

    let mut output: Vec<String> = Vec::new();

    // Aliases
    let mut jump_labels: Vec<JumpLabel> = Vec::new();
    let mut alias_labels: Vec<AliasLabel> = Vec::new();

    // Remove unnecessary characters
    for (_idx, line) in file.lines().enumerate() {
        let line = &line[0..line.find(COMMENT_DELIM).unwrap_or(line.len())].trim();
        let line = line.replace(",", " ");
        let line = line.to_ascii_uppercase();
        if line.len() > 0 {
            output.push(line);
        }
    }

    // Find labels
    let mut tracking_lable = false;
    let mut tracking_idx = 0;
    for (_idx, line) in output.iter_mut().enumerate() {

        if line.starts_with(LABEL_DELIM) {
            let split = line.split_whitespace().collect::<Vec<&str>>();

            // Detect alias
            if split[0] == (format!{"{}ALIAS", LABEL_DELIM}).as_str() {
                alias_labels.push(AliasLabel { from: (split[1]).to_string(), to: (split[2]).to_string() });
            } else if split[0] == LABEL_DELIM {

                // Detect jump labels
                if !tracking_lable {
                    tracking_lable = true;
                } else {
                    tracking_idx += 1;
                }

                let label = JumpLabel {
                    name: split[1].to_string(),
                    address: u16::max_value(), // Use max since no address has been given yet
                    address_str: String::new(),
                    size: 0, // Start with 0 since no data added to label yet
                    instructions: Vec::new(),
                };

                jump_labels.push(label);
            }
        }  else {
            if tracking_lable {
                // Separate between data and jumps
                let split = line.split_whitespace().collect::<Vec<&str>>();
                if INSTRCTIONS.contains(&split[0]) {
                    jump_labels[tracking_idx].size += 2; // Add size
                    jump_labels[tracking_idx].instructions.push(line.to_string()); // Add size
                } else {
                    // Normalize data to hex
                    for (_idx, data) in split.iter().enumerate() {
                        let data = num(data, 2).unwrap();
                        jump_labels[tracking_idx].size += 2; // Add size
                        jump_labels[tracking_idx].instructions.push(data); // Add size
                    }
                }
            }
        }
        // Start addressing from 0x200 which is the start of the program
        let mut current_address = 0x200;

        // Find start label
        for label in &mut jump_labels {
            if label.name == "START" {
                label.address = current_address;
                current_address += label.size; // TODO: Check that there is not off by one error here
            }
        }
        // Find all other labels
        for label in &mut jump_labels {
            if label.name != "START" {
                label.address = current_address;
                current_address += label.size; // TODO: Check that there is not off by one error here
            }
        }

        // Convert addresses to hex strings
        for label in &mut jump_labels {
            label.address_str = format!("0x{:X}", label.address);
        }
    }

    // Remove aliases from output
    let remove_str = format!{"{}ALIAS", LABEL_DELIM};
    output.retain(|x| !x.starts_with(remove_str.as_str()));

    return (output, alias_labels, jump_labels);
}

fn decode(line: &str, aliases: &Vec<AliasLabel>, jumps: &Vec<JumpLabel>) -> Result<String> {
    // Check for label
    let mut split = line.split_whitespace().collect::<Vec<&str>>();

    if split[0] == ":" {
        // Return jump address
        for jump in jumps {
            if split[1] == jump.name {
                // Addresses are always 3 nibbles long for one 0 is needed to pad
                let address = format!("0{:X}", jump.address);
                return Ok(address);
            }
        }
        return Err(anyhow!("Unknown label : {}", split[1]));
    }

    // Check split for aliases
    for c in split.iter_mut() {
        for alias in aliases {
            if *c == alias.from {
                *c = &alias.to;
            }
        }
    }

    // Check split for jumps
    for c in split.iter_mut() {
        for jump in jumps {
            if *c == jump.name {
                *c = &jump.address_str;
            }
        }
    }

    // Handle data labels
    if split[0].starts_with("0X") || split[0].starts_with("0B") || split[0].starts_with("$") {
        let mut data = String::new();
        for i in 0..split.len() {
            let num = num(split[i], 2)?;
            data.push_str(&num);
        }
        return Ok(data.replace("0X", ""));
    }


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

    // Init memory map
    //let mut memory: [u8; MAX_MEMORY] = [0; MAX_MEMORY];

    // Read file
    let file = std::fs::read_to_string(&args[1]).expect("Unable to read file");
    let (mut preprocessed, aliases, jumps) = preprocess(file);
    for (idx, line) in preprocessed.iter_mut().enumerate() {
        let d = decode(line, &aliases, &jumps);
        if d.is_err() {
            let err_msg = format!("Error on line {} ({})", idx, line);
            println!("{} | {}", err_msg, d.unwrap_err());
            return;
        }
        let d = d.unwrap();
        println!(" {:30} | {}", line, d);
    }
}
