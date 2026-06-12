mod event;

use crate::event::Event;
use time::macros::format_description;
use clap::{Parser, Subcommand};
use std::path::Path;
use time::Date;

#[derive(Parser, Debug)]
#[command(name = "event-tree")]
#[command(about = "A CLI tool to modify serializable Event trees", long_about = None)]
struct Cli {
    /// Path to the JSON file tracking the Event Tree
    #[arg(short, long, default_value = "tree.json")]
    file: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a new tree file with a root event node
    Init {
        #[arg(short, long)]
        desc: String,
        #[arg(short='D', long, help = "Format: YYYY-MM-DD")]
        date: String,
    },
    /// Print the entire tree visualization
    Print,
    /// Add a child node to an existing node
    AddChild {
        #[arg(long, help = "ID of the parent node")]
        parent_id: u32,
        #[arg(short, long)]
        desc: String,
        #[arg(short='D', long, help = "Format: YYYY-MM-DD")]
        date: String,
        #[arg(short, long, value_delimiter = ',')]
        conditions: Vec<String>,
    },
    /// Move an event node from an existing parent to a new parent
    Transfer {
        #[arg(long, help = "ID of current parent")]
        current_parent_id: u32,
        #[arg(long, help = "ID of the target node to move")]
        child_id: u32,
        #[arg(long, help = "ID of the target parent node")]
        target_parent_id: u32,
    },
    /// Delete a child node under a parent node
    RemoveChild {
        #[arg(long)]
        parent_id: u32,
        #[arg(long)]
        child_id: u32,
    },
    GetDescription {
        #[arg(short)]
        id: u32
    }
}

fn main() {
    let cli = Cli::parse();

    // Check if handling explicit initialization
    if let Commands::Init { desc, date } = &cli.command {
        let parsed_date = Date::parse(date, format_description!("[year]-[month]-[day]"))
            .expect("Invalid date format. Please use YYYY-MM-DD");
        let root = Event::new(parsed_date, &desc);
        root.save_to_file(&cli.file).expect("Failed to initialize file");
        println!("Successfully created new event tree root at '{}'", cli.file);
        return;
    }

    // Ensure the tree file already exists for all other commands
    if !Path::new(&cli.file).exists() {
        eprintln!("Error: File '{}' not found. Please run 'init' first.", cli.file);
        std::process::exit(1);
    }

    // Load existing tree from file
    let mut root = Event::load_from_file(&cli.file).expect("Failed to read the tree file");

    match cli.command {
        Commands::Init { .. } => unreachable!(), // Handled above

        Commands::Print => {
            println!("\n--- Current Event Tree Structure ---");
            print!("{}", root.to_string());
        }

        Commands::AddChild { parent_id, desc, date, conditions } => {
            let parsed_date = Date::parse(&date, format_description!("[year]-[month]-[day]"))
                .expect("Invalid date format. Use YYYY-MM-DD");
            
            Event::set_starting_id(root.find_max_id()+1);
            println!("{}", root.find_max_id()+1);
            if let Some(mut parent) = root.find_by_id(parent_id-1) {
                let mut child = Event::new(parsed_date, &desc);
                let child_id = child.id();
                parent.add_child(&mut child, conditions);
                
                root.save_to_file(&cli.file).expect("Failed to save tree changes");
                println!("Successfully added node [ID: {}] to Parent [ID: {}]", child_id+1, parent_id);
            } else {
                eprintln!("Error: Parent node with ID {} not found.", parent_id);
            }
        }

        Commands::Transfer { current_parent_id, child_id, target_parent_id } => {
            let mut current_parent = root.find_by_id(current_parent_id-1);
            let child = root.find_by_id(child_id-1);
            let target_parent = root.find_by_id(target_parent_id-1);

            match (current_parent.as_mut(), child.as_ref(), target_parent.as_ref()) {
                (Some(p), Some(c), Some(t)) => {
                    p.transfer(c, t);
                    root.save_to_file(&cli.file).expect("Failed to save changes");
                    println!("Moved node {} from Parent {} to Target {}", child_id, current_parent_id, target_parent_id);
                }
                _ => {
                    eprintln!("Error: One or more IDs provided do not exist in the file tree.");
                }
            }
        }

        Commands::GetDescription { id } => {
            let res = root.find_by_id(id-1);
            match res {
                Some(e) => println!("{}: {}", e.date(), e.description()),
                None => eprintln!("Error: One or more IDs provided do not exist in the file tree."),
            };
        }

        Commands::RemoveChild { parent_id, child_id } => {
            let mut parent = root.find_by_id(parent_id-1);
            let child = root.find_by_id(child_id-1);

            match (parent.as_mut(), child.as_ref()) {
                (Some(p), Some(c)) => {
                    p.remove_child(c);
                    root.save_to_file(&cli.file).expect("Failed to save changes");
                    println!("Removed child node {} from parent {}", child_id, parent_id);
                }
                _ => {
                    eprintln!("Error: Parent ID {} or Child ID {} not found.", parent_id, child_id);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use time::macros::date;
    use crate::Event;

    #[test]
    fn test_make_organizer() {
        let mut root = Event::new(date!(2026-01-03), "Go to the store");
        let mut eggs = Event::new(date!(2026-01-05), "make some eggs");
        let mut sandwich = Event::new(date!(2026-01-07), "make a sandwich");
        eggs.add_child(&mut sandwich, vec![]);
        root.add_child(&mut eggs, vec![]);

        assert_eq!(root.get_children(), vec![eggs]);
        assert_eq!(root.get_children()[0].get_children(), vec![sandwich]);
    }

    #[test]
    fn test_print_event() {
        Event::set_starting_id(0);
        let root = Event::new(date!(2026-01-03), "Go to the store");
        assert_eq!(root.to_string(), "(1) \n".to_string());
    }

    #[test]
    fn test_dependent_events() {
        let mut root = Event::new(date!(2026-01-03), "Go to the store");
        let mut eggs = Event::new(date!(2026-01-05), "make some eggs");
        let mut sandwich = Event::new(date!(2026-01-05), "make a sandwich");
        root.add_child(&mut eggs, vec!["bought eggs".to_string()]);
        root.add_child(&mut sandwich, vec!["bought bread".to_string(), "bought ham".to_string()]);

        assert_eq!(root.get_children(), vec![eggs, sandwich]);
    }

    #[test]
    fn test_write_save_to_file_linear_timeline() -> std::io::Result<()> {
        let mut root = Event::new(date!(2026-01-03), "Go to the store");
        let mut eggs = Event::new(date!(2026-01-05), "make some eggs");
        let mut sandwich = Event::new(date!(2026-01-05), "make a sandwich");
        sandwich.add_child(&mut eggs, vec!["bought eggs".to_string()]);
        root.add_child(&mut sandwich, vec!["bought bread".to_string(), "bought ham".to_string()]);

        root.save_to_file("tmp.txt")?;
        let new_tree = Event::load_from_file("tmp.txt")?;

        assert_eq!(root.to_string(), new_tree.to_string());
        Ok(())
    }
}
