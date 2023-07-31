#![deny(clippy::suspicious)]
#![deny(clippy::style)]
#![deny(clippy::complexity)]
#![deny(clippy::perf)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]

use std::{
    cmp::Ordering,
    fs::{self},
    io::{stdout, Result, Write},
    rc::Rc, path::PathBuf,
};

use crossterm::{
    event::{read, Event, KeyCode},
    queue,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};

use defer_lite::defer;

fn main() -> Result<()> {
    let paths = collect_paths()?;

    enable_raw_mode()?;
    defer! {disable_raw_mode().unwrap();};

    let mut buffer = String::new();
    let mut stdout = stdout();
    let mut targ: String;

    loop {
        queue!(stdout, Clear(ClearType::All))?;
        write!(stdout, "\r\n")?;

        if let Some(res) = search(&paths, &buffer) {
            unsafe {
                targ = res.lines().last().unwrap_unchecked().to_owned();
            }

            write!(stdout, "{res}\r\n")
        } else {
            targ = ".".to_owned();
            write!(stdout, "Nobody here but us chickens!\r\n")
        }?;

        write!(stdout, "{buffer}\r\n")?;

        //input
        //read will block until it gets an event (this is perfect)
        if let Event::Key(k) = read()? {
            match k.code {
                KeyCode::Char(c) => {
                    buffer.push(c);
                }
                KeyCode::Backspace => {
                    buffer.pop();
                }
                KeyCode::Esc => return Ok(()),
                KeyCode::Enter => {
                    let mut ctx =
                        copypasta_ext::try_context().expect("failed to get clipboard context");
                    ctx.set_contents(format!("cd {targ}")).unwrap();
                    return Ok(());
                }

                //fill in keys as needed
                _ => {}
            }
        }

        stdout.flush()?;
    }
}

fn collect_paths() -> Result<Rc<[Box<str>]>> {
    fn collect(path: PathBuf, table: &mut Vec<String>) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let new_path = entry.path().to_string_lossy().into_owned();

            if entry.file_type()?.is_dir() && !new_path.contains("/.") {
                table.push(new_path.clone());
                collect(PathBuf::from(&new_path), table)?;
            }
        }

        Ok(())
    }

    let mut table = vec![];
    collect(xdg_home::home_dir().unwrap(), &mut table)?;

    //dear god
    let ptr = table
        .into_iter()
        .map(String::into_boxed_str)
        .collect::<Vec<Box<str>>>();

    Ok(Rc::from(ptr))
}

fn search(paths: &Rc<[Box<str>]>, fil: &str) -> Option<String> {
    if fil.is_empty() {
        return None;
    }

    let mut paths: Vec<&str> = (*paths)
        .iter()
        .filter(|f| f.contains(fil))
        .map(|s| &(**s))
        .collect();

    if paths.is_empty() {
        return None;
    }

    unsafe {
        paths.sort_by(|a, b| {
            let mut res = b
                .find(fil)
                .unwrap_unchecked()
                .cmp(&a.find(fil).unwrap_unchecked());
            if res == Ordering::Equal {
                res = b.len().cmp(&a.len())
            }

            res
        });
    }

    Some(paths.join("\r\n"))
}
