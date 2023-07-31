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
    rc::Rc, path::PathBuf, env, str::FromStr, process, time::Duration,
};

use crossterm::{
    event::{read, Event, KeyCode},
    queue,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType}, execute, cursor,
};

use defer_lite::defer;

use indicatif::{self, ProgressBar};


fn main() -> Result<()> {

    let paths = match collect_paths() {
        Ok(it) => it,
        Err(err) => {println!("cannot access {}\r\nerror: {}", get_cwd().to_str().unwrap(), err); process::abort()},
    };
    
    //println!("{paths:?}");

    enable_raw_mode()?;
    defer! {disable_raw_mode().unwrap(); execute!(stdout(), cursor::Show).unwrap();};

    let mut buffer = String::new();
    let mut stdout = stdout();
    let mut targ: String;
        
        execute!(stdout, Clear(ClearType::Purge), cursor::Hide)?;

    let mut idx = 0;
    loop {
        queue!(stdout, Clear(ClearType::All))?;
        write!(stdout, "\r\n")?;

        if let Some(res) = search(&paths, &buffer) {

            idx = idx.clamp(0, res.len()-1);

            unsafe {
                targ = res.get_unchecked(idx).clone();
            }

            write!(stdout, "{}\r\n", join_highlight(res, idx))
        } else {
            idx = usize::MAX;
            targ = ".".to_owned();
            write!(stdout, "Nobody here but us chickens!\r\n")
        }?;

        write!(stdout, "{buffer}\r\n")?;

        //input
        //read will block until it gets an event (this is perfect)
        if let Event::Key(k) = read()? {
            match k.code {
                KeyCode::Char(c) => {
                    idx = usize::MAX;
                    buffer.push(c);
                }
                KeyCode::Backspace => {
                    idx = usize::MAX;
                    buffer.pop();
                }

                KeyCode::Up => {
                    idx = idx.saturating_sub(1);
                }
                KeyCode::Down => {
                    idx = idx.saturating_add(1);
                }

                KeyCode::Esc => return Ok(()),
                KeyCode::Enter => {
                    let mut ctx =
                        copypasta_ext::try_context().expect("failed to get clipboard context");
                    ctx.set_contents(format!("cd {targ}")).unwrap();
                    println!("cd {targ}");
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

    let spinner = ProgressBar::new_spinner().with_message("walking directories...");
    spinner.enable_steady_tick(Duration::from_millis(50)); 

    let mut table = vec![];

    let start = get_cwd();

    collect(start, &mut table)?;
    
    spinner.is_finished();

    //dear god
    let ptr = table
        .into_iter()
        .map(String::into_boxed_str)
        .collect::<Vec<Box<str>>>();

    Ok(Rc::from(ptr))
}

//possibly optimize by allocating before hand
fn join_highlight(vec: Vec<String>, idx:usize) -> String {
    let mut out = String::with_capacity(vec.len()*vec[0].len());
    for (i, e) in vec.into_iter().enumerate() {
        if i == idx {
            out.push_str("\x1b[1;33m");
            out.push_str(&e);
            out.push_str("\x1b[0m");
        } else {
            out.push_str(&e);
        }
        out.push_str("\r\n");
    }

    out
}


fn get_cwd() -> PathBuf {
    env::args().nth(1)
        .map_or_else(
            || xdg_home::home_dir().unwrap(), 
            |v| PathBuf::from_str(&v).unwrap()
            )
}

fn search(paths: &Rc<[Box<str>]>, fil: &str) -> Option<Vec<String>> {
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
                res = b.len().cmp(&a.len());
            }

            res
        });
    }

    Some(paths.into_iter().map(ToString::to_string).collect())
}
