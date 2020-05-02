extern crate clap;
use clap::{Arg, App};
use std::io::{BufReader,BufRead};
use std::fs::File;

fn main() {
    let args = App::new("cat")
                    .version("1.0")
                    .author("Colin D. <colin@quirl.co.nz>")
                    .about("concatenate files and print on the standard output")
                    .arg(Arg::with_name("number-nonblank").short("b").long("number-nonblank").help("Print line numbers for non blank lines. Overrides -n").overrides_with("number"))
                    .arg(Arg::with_name("show-ends").short("E").long("show-ends").help("display $ at end of each line"))
                    .arg(Arg::with_name("number").short("-n").long("number").help("number all output lines"))
                    .arg(Arg::with_name("squeeze-blank").short("-s").long("squeeze-blank").help("suppress repeated empty output lines"))
                    .arg(Arg::with_name("show-tabs").short("-T").long("show-tabs").help("display TAB characters as ^I"))
                    .arg(Arg::with_name("files").takes_value(true).value_name("FILE").multiple(true).index(1).default_value("/dev/stdin").hide_default_value(true))
                    .get_matches();

    let number_nonblank = args.is_present("number-nonblank");
    let number = number_nonblank || args.is_present("number");
    let squeeze_blank = args.is_present("squeeze-blank");
    let show_tabs = args.is_present("show-tabs");
    let files = args.values_of("files").unwrap();

    let mut linenumber = 0;
    let mut currently_squeezing = false;

    for filename in files {
        let filehandle = match File::open(filename) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("{}: {}", filename, err);
                continue;
            }
        };

        let filehandle = BufReader::new(filehandle);
        for line in filehandle.lines() {
            let mut line = match line {
                Ok(line) => line,
                Err(err) => {
                    eprintln!("{}: {}", filename, err);
                    continue;
                }
            };

            let is_whitespace = line.trim().len() == 0;
            if is_whitespace && squeeze_blank{
                if currently_squeezing {
                    continue;
                }
            }
            currently_squeezing = is_whitespace;

            let mut output = String::new();
            if number {
                if !(number_nonblank && is_whitespace) {
                    linenumber += 1;
                    output = format!("{:^8}", linenumber);
                }
            }

            if show_tabs {
                line = line.replace("\t", "^I");
            }

            output.push_str(line.as_str());
            println!("{}", output);
        }
    }
}