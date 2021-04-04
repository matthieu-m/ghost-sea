extern crate ghost_sea;
extern crate static_rc;

mod linked_list;
mod ghost_linked_list;

use self::ghost_linked_list::GhostLinkedList;
use self::linked_list::LinkedList;

fn main() {
    let mut list: LinkedList<String> = LinkedList::default();

    assert!(list.is_empty());
    //assert_eq!(0, list.len());

    assert_eq!(None, list.front_mut());
    assert_eq!(None, list.back_mut());
    assert_eq!(None, list.pop_front());
    assert_eq!(None, list.pop_back());

    list.push_front("Hello, World!".to_string());
    list.push_back("Hello, You!".to_string());

    assert!(!list.is_empty());
    //assert_eq!(2, list.len());
    assert_eq!(Some("Hello, World!"), list.front().map(|s| &**s));
    assert_eq!(Some("Hello, You!"), list.back().map(|s| &**s));

    //println!("{:?}", list);


}
