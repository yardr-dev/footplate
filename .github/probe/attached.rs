// The same, after attaching to its parent's console, as footplate's does now.
#![windows_subsystem = "windows"]

#[link(name = "kernel32")]
extern "system" {
    fn AttachConsole(process_id: u32) -> i32;
}

fn main() {
    // ATTACH_PARENT_PROCESS
    unsafe { AttachConsole(0xFFFF_FFFF) };
    println!("footplate 0.0.0");
}
