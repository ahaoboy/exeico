use exeico::get_ico;

fn main() {
    let (Some(exe_path), Some(id), Some(ico_path)) = (
        std::env::args().nth(1),
        std::env::args().nth(2),
        std::env::args().nth(3),
    ) else {
        println!("exeico <exe_path> <id> <ico_path>");
        return;
    };

    let id = id.parse::<i32>().expect("Invalid icon ID").abs() as u32;
    let bin = std::fs::read(exe_path).expect("Failed to read EXE file");
    let ico = get_ico(&bin, id).expect("Failed to get icon");
    std::fs::write(ico_path, ico).expect("Failed to write icon file");
}
