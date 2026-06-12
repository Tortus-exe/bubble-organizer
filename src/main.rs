mod event;

use crate::event::Event;
use time::macros::date;

fn main() {
    let mut root = Event::new(date!(2026-01-03), "Go to the store");
    let mut eggs = Event::new(date!(2026-01-05), "make some eggs");
    let mut sandwich = Event::new(date!(2026-01-07), "make a sandwich");
    root.add_child(&mut sandwich, vec![]);
    root.add_child(&mut eggs, vec![]);

    println!("{}", root.to_string());
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
        unsafe { Event::force_reset_id(); } // need to reset the ids because we made a bunch up
                                            // there.
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
}
