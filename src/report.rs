use colored::Colorize;
use lincolns::Position;
use std::path::Path;

pub(crate) fn report<P>(
    path: P,
    content: &str,
    pos: &Position,
    error: &str,
) where
    P: AsRef<Path>,
{
    let Position { line, col } = pos;
    println!("⚠️ {}", format!("error: {}", error).red());
    println!();
    let lines = content
        .lines()
        .enumerate()
        .filter(|(idx, _)| (line - 3..=*line + 1).contains(idx))
        .collect::<Vec<_>>();
    let max_line = lines
        .last()
        .map(|(idx, _)| idx.to_string().len())
        .unwrap_or_default();
    for (idx, matched) in lines {
        if idx == line - 1 {
            print!("{} ", ">".red());
        } else {
            print!("  ");
        }
        println!(
            " {}{}",
            format!("{}{} |", " ".repeat(max_line - idx.to_string().len()), idx).dimmed(),
            matched
        )
    }
    println!();
    println!(
        "{}",
        format!("at {}:{}:{}", path.as_ref().display(), line, col).dimmed()
    );
}
