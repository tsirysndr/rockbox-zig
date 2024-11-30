use std::cell::RefCell;

#[derive(Default, Clone, Debug)]
pub struct NavigationHistory {
    pages: RefCell<Vec<(String, String)>>,
}

impl NavigationHistory {
    // Create a new navigation history
    pub fn new() -> Self {
        Self {
            pages: RefCell::new(vec![("Albums".to_string(), "albums-page".to_string())]),
        }
    }

    pub fn new_from(page_name: String, page_id: String) -> Self {
        Self {
            pages: RefCell::new(vec![(page_name, page_id)]),
        }
    }

    // Push a new page to the history
    pub fn push(&self, page_name: String, page_id: String) {
        self.pages.borrow_mut().push((page_name, page_id));
    }

    // Pop the last page from the history
    pub fn pop(&self) -> (String, String) {
        let mut pages = self.pages.borrow_mut();

        // Always keep at least one page in the history
        if pages.len() > 1 {
            pages.pop().unwrap().clone()
        } else {
            pages.last().unwrap().clone()
        }
    }

    // Peek at the current page without removing it
    pub fn current(&self) -> (String, String) {
        self.pages.borrow().last().unwrap().clone()
    }

    // Clear the history, resetting to the default page
    pub fn reset(&self) {
        let mut pages = self.pages.borrow_mut();
        pages.clear();
        pages.push(("Albums".to_string(), "albums-page".to_string()));
    }

    pub fn len(&self) -> usize {
        self.pages.borrow().len()
    }
}
