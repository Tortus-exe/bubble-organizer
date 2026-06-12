use time::Date;
use std::rc::{Rc};
use std::ops::{Deref, DerefMut};
use std::cmp::{PartialEq, Eq};
use std::cell::RefCell;
use std::sync::atomic::{AtomicU32, Ordering};
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Event(Rc<RefCell<_Event>>);

impl Deref for Event {
    type Target = Rc<RefCell<_Event>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Event {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Rc<RefCell<_Event>>> for Event {
    fn from(val: Rc<RefCell<_Event>>) -> Self {
        Event(val)
    }
}

impl PartialEq<Self> for _Event {
    fn eq(&self, other: &Self) -> bool {
        other.id == self.id
    }

    fn ne(&self, other: &Self) -> bool {
        other.id != self.id
    }
}

impl Eq for _Event {}

#[derive(Serialize, Deserialize, Debug)]
pub struct _Event { // TODO: find a way to make this private but keep the other one public with
                    // derefmut!
    date: Date,
    id: u32, 
    desc: String,
    condition: Vec<String>,
    children: Vec<Event>
}

static NEXT_ID: AtomicU32 = AtomicU32::new(0);
impl Event {
    pub fn new(date: Date, desc: &str) -> Self {
        let e = _Event {
            date,
            id: NEXT_ID.load(Ordering::Relaxed),
            desc: desc.to_string(),
            // parents: vec![],
            condition: vec![],
            children: vec![],
        };
        NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Event(Rc::new(RefCell::new(e)))
    }

    pub fn remove_child(&mut self, child: &Event) {
        let index = self.borrow().children.iter().position(|x| x.borrow().id == child.borrow().id);
        match index {
            Some(i) => self.borrow_mut().children.remove(i),
            None => panic!("item not found!"),
        };
    }

    pub fn description(&self) -> String {
        self.borrow().desc.clone()
    }

    pub fn date(&self) -> Date {
        self.borrow().date
    }

    pub fn id(&self) -> u32 {
        self.borrow().id
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<Event> {
        if self.borrow().id == target_id {
            return Some(self.clone());
        }
        for child in self.get_children() {
            if let Some(found) = child.find_by_id(target_id) {
                return Some(found);
            }
        }
        None
    }

    pub fn find_max_id(&self) -> u32 {
        let current_id = self.borrow().id;
        self.borrow()
            .children
            .iter()
            .map(|child| child.find_max_id())
            .fold(current_id, |a, b| a.max(b))
    }

    pub fn transfer(&mut self, child: &Event, target: &Event) {
        target.borrow_mut().children.push(child.clone());
        self.remove_child(child);
    }

    pub fn add_child(&mut self, child: &mut Event, condition: Vec<String>) {
        self.borrow_mut().children.push(child.clone());
        self.borrow_mut().condition = condition;
        // child.borrow_mut().parents.push((Rc::<RefCell<_Event>>::downgrade(self), condition));
    }

    pub fn get_children(&self) -> Vec<Event> {
        self.borrow().children.clone()
    }

    pub fn set_starting_id(id: u32) {
        NEXT_ID.store(id, Ordering::Relaxed);
    }

    pub fn save_to_file(&self, filename: &str) -> std::io::Result<()> {
        let json_string = serde_json::to_string_pretty(self)?;
        let mut file = File::create(filename)?;
        file.write_all(json_string.as_bytes())?;
        Ok(())
    }

    pub fn load_from_file(filename: &str) -> std::io::Result<Self> {
        let mut file = File::open(filename)?;
        let mut json_string = String::new();
        file.read_to_string(&mut json_string)?;
        
        let e: Event = serde_json::from_str(&json_string)?;
        Ok(e)
    }
}

impl ToString for Event {
    fn to_string(&self) -> String {
        let mut curr_row = vec![self.clone()];
        let mut outstr = String::new();
        let mut this_row_parent_connections = String::new();

        while !curr_row.is_empty() {
            let mut next_row: Vec<Event> = vec![];
            let _curr_row_counter = 0;
            let mut rowstr = String::new();
            let mut connectionsStr = String::new();
            let mut next_row_parent_connections = String::new();
            for cell in curr_row {
                let cell_str = format!("({}) ", cell.borrow().id+1);
                rowstr.push_str(cell_str.as_str());
                if !this_row_parent_connections.is_empty() {
                    connectionsStr.push(this_row_parent_connections.remove(0));
                    let pad_char = this_row_parent_connections.remove(0);
                    connectionsStr.push_str(std::iter::repeat_n(pad_char, cell_str.len() - 1).collect::<String>().as_str())
                }
                next_row.append(&mut cell.borrow().children.clone());
                if cell.borrow().children.len() == 1 {
                    next_row_parent_connections.push_str("│ ");
                } else {
                    next_row_parent_connections.push('├');
                    for _i in 2..cell.borrow().children.len() {
                        next_row_parent_connections.push_str("─┬");
                    }
                    next_row_parent_connections.push_str("─┐ ");
                }
            }
            this_row_parent_connections = next_row_parent_connections;
            if !connectionsStr.is_empty() {
                outstr.push_str(connectionsStr.as_str());
                outstr.push('\n');
            }
            outstr.push_str(rowstr.as_str());
            outstr.push('\n');
            curr_row = next_row;
        }
        outstr
    }
}