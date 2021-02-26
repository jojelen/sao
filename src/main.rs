use std::env;
use std::io;
use std::io::Write;
use subprocess::*;
use crossterm::{style,Color};
use nix::unistd::execv;
use std::ffi::CString;
use regex::Regex;

fn split_line<'a>(line: &'a str, sep: &'static str) -> Option<(&'a str, &'a str)> {
    let sep_index: usize = line.find(sep)?;
    let (first, second) = line.split_at(sep_index);
    Some((first, &second[1..]))
}

fn err_msg(msg: &str) {
	eprintln!("{}: {}", style("ERROR").with(Color::Red), msg);
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    args.push(String::from("-n"));
    args.push(String::from("-I"));

	let out = Exec::cmd("rgrep").args(&args)
      .stdout(Redirection::Pipe)
        .capture().unwrap()
          .stdout_str();

    let lines = out.split('\n');
	let re = Regex::new(r":[0-9]+:").expect("Could not create regex");

    let mut index = 1;
	let mut file_names : Vec<&str> = Vec::new();
	let mut line_numbers : Vec<&str> = Vec::new();
    for l in lines{
		if l.is_empty() {
			continue;
		}

		if re.is_match(l) == false {
			println!("{}", l );
			continue;
		}

        let first_split = split_line(l, ":").expect("Failed first split");
        let file_name = style(first_split.0).with(Color::Red);

        let second_split = split_line(first_split.1, ":").expect("Failed second split");
        let line_num = style(second_split.0).with(Color::Yellow);

        println!("{}.){}:{}: {}", style(index.to_string()).with(Color::Green), file_name,
				 line_num, second_split.1);
		file_names.push(&first_split.0);
		line_numbers.push(&second_split.0);
        index += 1;
    }

	if index == 1 {
		return;
	}

	print!("Open: ");
	io::stdout().flush().unwrap();
    let mut input = String::new();
	io::stdin().read_line(&mut input).unwrap();
	let n: i32 = match input.trim().parse() {
		Err(_) => -1,
		Ok(num) => num,
	};

	if n > 0 && n as usize <= file_names.len() {
		let idx : usize = n as usize - 1;
		println!("Opening {} at line nr {}...", file_names[idx], n);

		let editor_path = CString::new("/usr/bin/nvim").expect("CString failed");
		let file_arg = CString::new(file_names[idx]).expect("CString failed");
		let mut line_num : String = String::from("+");
		line_num.push_str(line_numbers[idx]);
		let line_arg = CString::new(line_num).expect("CString failed");

		execv(&editor_path, &[&editor_path, &file_arg, &line_arg]).unwrap();
	}
	else if n > 0  {
		err_msg("Invalid input");
	}

	// Quits silently for non-number input.
}
