use uniffi_geometry::Point;
use uniffi_todolist::TodoEntry;

uniffi::use_udl_record!(uniffi_todolist, TodoEntry);
uniffi::use_udl_record!(uniffi_geometry, Point);

// Needed because we are importing types from UDL but using them via procmacros
pub struct UniFfiTag;

#[uniffi::export]
pub fn add_entry_add(entry: TodoEntry, point: Point) -> () {
    println!("Adding entry {entry:?} at {point:?}");
}
