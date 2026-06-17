use crate::boards;

pub fn run() {
    println!("Supported boards:\n");
    for board in boards::list() {
        let marker = if board.experimental {
            " (experimental — flash not implemented yet)"
        } else {
            ""
        };
        println!(
            "  {} — flash via {}{}",
            board.name, board.flash_tool, marker
        );
    }
}
