use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process, thread, time,
};

const CONFIG_PATH: &str = "config.txt";

struct Configs {
    read_folder: String,
    write_folder: String,
    movitel_code_suffix: String,
}

fn main() {
    println!("Starting application...");
    let configs = read_configs();

    let _ = process::Command::new("cmd.exe")
        .arg("/c")
        .arg("pause")
        .status();

    println!("Application started.");


    loop {
        println!("Looking for any file...");
        let mut is_empty = true;
        let dir = match fs::read_dir(&configs.read_folder) {
            Ok(files) => files,
            Err(e) => {
                println!("{}\nSomething went wrong, trying to read directory ", e);
                break;
            }
        };

        for entry in dir {
            is_empty = false;
            let file: fs::DirEntry = entry.expect("Unable to open file");
            let path: PathBuf = file.path();
            let file_name: String = file
                .file_name()
                .into_string()
                .expect("Unable to read file name");

            match file_name.chars().nth(0) {
                Some('V') => process_vodacom(&path, String::from(&configs.write_folder)),
                Some('C') => process_movitel(
                    &path,
                    String::from(&configs.write_folder),
                    String::from(&configs.movitel_code_suffix),
                ),
                _ => println!("unrecognized file name pattern"),
            }
        }

        thread::sleep(time::Duration::from_secs(5));
        if is_empty{
            println!("Nothing found.");
        }
        println!("Look up complete.");
    }
}

/**
 * Process Vodacom related files
 *
 * @path the path to the given file
 */
fn process_vodacom(path: &PathBuf, write_path: String) {
    println!("Processing vodacom file: {}...", path.file_name().unwrap().to_str().unwrap());
    let file = fs::File::open(path).expect("No such file");
    let buf = BufReader::new(file);
    let mut lines: Vec<String> = buf
        .lines()
        .map(|l| l.expect("Could not parse line"))
        .collect();

    for line in lines.iter_mut() {
        if line.len() < 41{
            continue;
        }
        line.insert_str(9, "00");
    }
    println!("Processing completed.");
    write_to_file(&lines, path, write_path);
    remove_file(path);
}

/**
 * Process Movitel related files
 *
 * @path the path to the given file
 */
fn process_movitel(path: &PathBuf, write_path: String, date: String) {
    println!("Processing movitel file: {}...", path.file_name().unwrap().to_str().unwrap());
    let file = fs::File::open(path).expect("No such file");
    let buf = BufReader::new(file);
    let mut lines: Vec<String> = buf
        .lines()
        .map(|l| l.expect("Could not parse line"))
        .collect();

    let mut suffix: String = String::from(lines.get(2).expect("Unable to find face value line")); // fetch the value of recharges
    suffix = String::from(suffix.split(":").nth(1).expect("Suffix out of bounds"));
    suffix = format!("{:0>4}", suffix);
    suffix.insert(0, 'M');

    lines.drain(0..37); // removes unneeded lines
    lines.remove(lines.len() - 1);

    for line in lines.iter_mut() {
        line.remove(13); // removes white space in the middle of string
        line.drain(0..2); // remove a 20
        let end_char = line
            .chars()
            .nth(line.len() - 1)
            .expect("Could not find last char"); // gets last char
        line.replace_range(0..0, &end_char.to_string()); // sets it to first char
        line.remove(line.len() - 1); // removes last character
        line.insert_str(0, &date); //  insert date
        line.insert_str(0, "1"); //  insert 1
        line.insert_str(line.len(), &suffix)
    }
    println!("Processing completed.");
    write_to_file(&lines, path, write_path);
    remove_file(path);
}

/**
 * Write data to file.
 *
 * @lines a vector containing string to be written to file
 * @path  the original file path to take the file name from
 */
fn write_to_file(lines: &Vec<String>, path: &PathBuf, write_path: String) {
    let name = String::from(
        path.file_name()
            .expect("Unable to read file name") // determine file name
            .to_str()
            .expect("Unable to convert file name to string"),
    );
    println!("Writing file: {}...", name);
    let mut path_string: String = write_path; // compose file path to out
    path_string.push_str(&name);
    let full_path = std::path::Path::new(&path_string);

    let mut f = fs::File::create(full_path).expect("Unable to create file"); // create file to write to
    for line in lines {
        write!(f, "{}\n", line).expect("Unable to write to file"); // write lines to file
    }
    println!("Write complete.");
}

/**
 * Removes a file from the sytem
 *
 * @path the path of the file to remove
 */
fn remove_file(path: &PathBuf) {
    println!("Removing file: {}...", path.file_name().unwrap().to_str().unwrap());
    fs::remove_file(path).expect("Unable to remove file");
    println!("File removed.")
}

fn read_configs() -> Configs {
    println!("Reading configuration file...");
    let file = fs::File::open(CONFIG_PATH).expect("No such file");
    let buf = BufReader::new(file);
    let mut lines: Vec<String> = buf
        .lines()
        .map(|l| l.expect("Could not parse line"))
        .collect();

    let configs = Configs {
        movitel_code_suffix: lines.pop().unwrap(),
        write_folder: lines.pop().unwrap(),
        read_folder: lines.pop().unwrap(),
    };

    println!("read folder: {}", configs.read_folder);
    println!("write folder: {}", configs.write_folder);
    println!("movitel code suffix: {}", configs.movitel_code_suffix);
    println!("Configurations read.");

    configs
}
