use emulator::*;
use parser::*;
use raylib::prelude::*;

mod emulator;
mod parser;

#[path = "electron-2/lib.rs"]
mod electron_2;
use electron_2::Emulator as EmulatorV2;

const WINDOW_SIZE: (i32, i32) = (720, 720);

fn format_data(mut data: String, len: usize) -> String {
    for _ in 0..len - data.len() {
        data.push(' ')
    }
    data
}

// --- V1 Helpers ---

fn print_port(emulator: &Emulator, port: u8) {
    let mut port_data = format!("{:b}", emulator.ports.out[port as usize]);
    for _ in 0..8 - port_data.len() {
        port_data.insert(0, '0');
    }
    print!(
        "     Port {}: ({})  ",
        port,
        format_data(emulator.ports.out[port as usize].to_string(), 3),
    );
    for char in port_data.chars() {
        if char == '0' {
            print!("░░")
        } else {
            print!("▓▓");
        }
    }
    println!();
}

fn draw_terminal_screen(emulator: &Emulator) {
    print!("▓▓▓▒▒▒░░░       Pipelines         ░░░▒▒▒▓▓▓    ");
    println!("▓▓▓▒▒▒░░░          Ports        ░░░▒▒▒▓▓▓");
    println!("___________________________________________");
    print!("| FETCH   | DECODE  | EXECUTE | WRITEBACK |");
    print_port(emulator, 0);
    print!(
        "{}{}{}{}  |",
        &emulator.fetch_register.operation.get_name(),
        emulator.decode_register.operation.get_name(),
        emulator.execute_register.operation.get_name(),
        emulator.write_back_register.operation.get_name()
    );
    print_port(emulator, 1);
    print!("▓▓▓▒▒▒░░░           ALU          ░░░▒▒▒▓▓▓ ");
    print_port(emulator, 2);
    print!("___________________________________________");
    print_port(emulator, 3);
    print!("| Accumalator |           Flags           |");
    print_port(emulator, 4);
    print!(
        "|      {}    ",
        format_data(emulator.alu.accumalator.to_string(), 3)
    );
    print!(
        "| Equals: {}             |",
        format_data(emulator.alu.flags.equals.to_string(), 5)
    );
    print_port(emulator, 5);
    print!(
        "|             | Greater: {}            |",
        format_data(emulator.alu.flags.greater_than.to_string(), 5)
    );
    print_port(emulator, 6);
    print!(
        "|             | Less: {}               |",
        format_data(emulator.alu.flags.less_than.to_string(), 5)
    );
    print_port(emulator, 7);
    println!(
        "|             | Overflow: {}           |",
        format_data(emulator.alu.flags.less_than.to_string(), 5) // Note: V1 reused less_than for overflow print?? Fixed in V2
    );
    println!();
    println!("__________________________________________");
    println!();
    println!("▓▓▓▒▒▒░░░         Memory         ░░░▒▒▒▓▓▓");
    println!("__________________________________________");
    println!("| Registers |");
    for i in 0..8 { 
        println!(
            "|   {}: {}  |",
            i,
            format_data(emulator.registers.read(i).to_string(), 3)
        );
    }
}

fn draw_ports(emulator: &Emulator, d: &mut RaylibDrawHandle, on_texture: &Texture2D, off_texture: &Texture2D) {
    for (port, _) in emulator.ports.out.iter().enumerate() {
        let mut port_data = format!("{:b}", emulator.ports.out[port]);
        for _ in 0..8 - port_data.len() {
            port_data.insert(0, '0');
        }
        for (i, char) in port_data.char_indices() {
            if char == '1' {
                d.draw_texture_pro(
                    on_texture,
                    Rectangle::new(0.0, 0.0, on_texture.width as f32, on_texture.height as f32),
                    Rectangle::new(
                    (i as i32 * WINDOW_SIZE.0 / 8) as f32,
                    (port as i32 * WINDOW_SIZE.1 / 8) as f32,
                    (WINDOW_SIZE.0 / 8) as f32,
                    (WINDOW_SIZE.1 / 8) as f32,
                    ),
                    Vector2::zero(),
                    0.0,
                    Color::WHITE,
                )
            } else {
                d.draw_texture_pro(
                    off_texture,
                    Rectangle::new(0.0, 0.0, off_texture.width as f32, off_texture.height as f32),
                    Rectangle::new(
                    (i as i32 * WINDOW_SIZE.0 / 8) as f32,
                    (port as i32 * WINDOW_SIZE.1 / 8) as f32,
                    (WINDOW_SIZE.0 / 8) as f32,
                    (WINDOW_SIZE.1 / 8) as f32,
                    ),
                    Vector2::zero(),
                    0.0,
                    Color::WHITE,
                )
            }
        }
    }
}

// --- V2 Helpers ---

fn print_port_v2(emulator: &EmulatorV2, port: u8) {
    let mut port_data = format!("{:b}", emulator.ports_out[port as usize]);
    for _ in 0..8 - port_data.len() {
        port_data.insert(0, '0');
    }
    print!(
        "     Port {}: ({})  ",
        port,
        format_data(emulator.ports_out[port as usize].to_string(), 3),
    );
    for char in port_data.chars() {
        if char == '0' {
            print!("░░")
        } else {
            print!("▓▓");
        }
    }
    println!();
}

fn draw_terminal_screen_v2(emulator: &EmulatorV2) {
    print!("▓▓▓▒▒▒░░░     Electron 2 Pipeline     ░░░▒▒▒▓▓▓    ");
    println!("▓▓▓▒▒▒░░░          Ports        ░░░▒▒▒▓▓▓");
    println!("___________________________________________");
    print!("| FETCH   | DECODE  | EXECUTE | WRITEBACK |");
    print_port_v2(emulator, 0);
    
    // Formatting pipeline op names
    let f_name = format_data(emulator.fetch_reg.operation.get_name(), 10);
    let d_name = format_data(emulator.decode_reg.operation.get_name(), 10);
    let e_name = format_data(emulator.execute_reg.operation.get_name(), 10);
    let w_name = format_data(emulator.writeback_reg.operation.get_name(), 10);

    print!(
        "|{}|{}|{}|{}|",
        f_name.get(0..9).unwrap_or(&f_name),
        d_name.get(0..9).unwrap_or(&d_name),
        e_name.get(0..9).unwrap_or(&e_name),
        w_name.get(0..11).unwrap_or(&w_name)
    );
    
    print_port_v2(emulator, 1);
    print!("▓▓▓▒▒▒░░░           ALU          ░░░▒▒▒▓▓▓ ");
    print_port_v2(emulator, 2);
    print!("___________________________________________");
    print_port_v2(emulator, 3);
    print!("| Accumulator |           Flags           |");
    print_port_v2(emulator, 4);
    print!(
        "|      {}    ",
        format_data(emulator.alu.accumulator.to_string(), 3)
    );
    print!(
        "| Equals: {}             |",
        format_data(emulator.alu.flags.equals.to_string(), 5)
    );
    print_port_v2(emulator, 5);
    print!(
        "|             | Greater: {}            |",
        format_data(emulator.alu.flags.greater.to_string(), 5)
    );
    print_port_v2(emulator, 6);
    print!(
        "|             | Less: {}               |",
        format_data(emulator.alu.flags.less.to_string(), 5)
    );
    print_port_v2(emulator, 7);
    println!(
        "|             | Overflow: {}           |",
        format_data(emulator.alu.flags.overflow.to_string(), 5)
    );
    println!();
    println!("__________________________________________");
    println!();
    println!("▓▓▓▒▒▒░░░         Memory         ░░░▒▒▒▓▓▓");
    println!("__________________________________________");
    println!("| Registers |      RAM      |     Stack    |");
    for i in 0..8 {
        // Show Registers 0-7, RAM 0-7 and 8-15, Stack Pointer
        let reg_val = format_data(emulator.registers.read(i as i32).to_string(), 3);
        let ram_val_1 = format_data(emulator.ram[i].to_string(), 3);
        let ram_val_2 = format_data(emulator.ram[i+8].to_string(), 3);
        
        let stack_marker = if emulator.sp == i as i32 { "< SP" } else if emulator.sp == (i+8) as i32 { "< SP" } else { "    " };
        
        println!(
            "| R{}: {}  | #{:02}: {} #{:02}: {} | {}",
            i, reg_val, i, ram_val_1, i+8, ram_val_2, stack_marker
        );
    }
}

fn draw_ports_v2(emulator: &EmulatorV2, d: &mut RaylibDrawHandle, on_texture: &Texture2D, off_texture: &Texture2D) {
    for (port, _) in emulator.ports_out.iter().enumerate() {
        let mut port_data = format!("{:b}", emulator.ports_out[port]);
        for _ in 0..8 - port_data.len() {
            port_data.insert(0, '0');
        }
        for (i, char) in port_data.char_indices() {
            if char == '1' {
                d.draw_texture_pro(
                    on_texture,
                    Rectangle::new(0.0, 0.0, on_texture.width as f32, on_texture.height as f32),
                    Rectangle::new(
                    (i as i32 * WINDOW_SIZE.0 / 8) as f32,
                    (port as i32 * WINDOW_SIZE.1 / 8) as f32,
                    (WINDOW_SIZE.0 / 8) as f32,
                    (WINDOW_SIZE.1 / 8) as f32,
                    ),
                    Vector2::zero(),
                    0.0,
                    Color::WHITE,
                )
            } else {
                d.draw_texture_pro(
                    off_texture,
                    Rectangle::new(0.0, 0.0, off_texture.width as f32, off_texture.height as f32),
                    Rectangle::new(
                    (i as i32 * WINDOW_SIZE.0 / 8) as f32,
                    (port as i32 * WINDOW_SIZE.1 / 8) as f32,
                    (WINDOW_SIZE.0 / 8) as f32,
                    (WINDOW_SIZE.1 / 8) as f32,
                    ),
                    Vector2::zero(),
                    0.0,
                    Color::WHITE,
                )
            }
        }
    }
}

fn clear_terminal_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let mut file_name = String::new();
    let mut terminal_output = true;
    let mut clock_speed = 1.0;
    let mut show_fps = false;
    let mut use_v2 = false;

    for (i, str) in args.iter().enumerate() {
        if str == "-f" {
            file_name = args.get(i + 1).unwrap_or(&String::new()).clone()
        }
        if str == "-c" {
            clock_speed = args.get(i + 1).unwrap().parse::<f32>().unwrap();
        }
        if str == "-nt" {
            terminal_output = false;
        }
        if str == "-fps" {
            show_fps = true;
        }
        if str == "-v2" {
            use_v2 = true;
        }
    }

    if file_name.is_empty() {
        println!("Error: No file name given. Use -f <filename>");
        return;
    }

    let (mut rl, thread) = raylib::init()
        .width(WINDOW_SIZE.0)
        .title(if use_v2 { "Electron 2 Emulator" } else { "Electron Emulator" })
        .height(WINDOW_SIZE.1)
        .build();

    let mut last_clock = std::time::Instant::now();
    let tick_speed = (1.0/clock_speed * 1000.0) as u128;
    
    let on_texture = rl.load_texture_from_image(&thread, &Image::load_image_from_mem(".png", &include_bytes!("on.png").to_vec(), include_bytes!("on.png").len() as i32).unwrap()).unwrap();
    let off_texture = rl.load_texture_from_image(&thread, &Image::load_image_from_mem(".png", &include_bytes!("off.png").to_vec(), include_bytes!("off.png").len() as i32).unwrap()).unwrap();

    if use_v2 {
        // --- V2 Execution ---
        println!("Starting Electron 2 Emulator...");
        let code = std::fs::read_to_string(&file_name).expect("Failed to read file");
        let mut emulator = EmulatorV2::new(code);

        while !rl.window_should_close() {
            if (std::time::Instant::now() - last_clock).as_millis() > tick_speed {
                emulator.clock();
                last_clock = std::time::Instant::now();
                clear_terminal_screen();
                if terminal_output {
                    draw_terminal_screen_v2(&emulator);
                }
            }
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);
            draw_ports_v2(&emulator, &mut d, &on_texture, &off_texture);
            if show_fps {
                d.draw_text(&d.get_fps().to_string(), 0, 0, 25, Color::WHITE);
            }
        }

    } else {
        // --- V1 Execution (Legacy) ---
        let program = ProgramLoader::load_program(&file_name);
        let mut emulator = Emulator::new(program);

        while !rl.window_should_close() {
            if (std::time::Instant::now() - last_clock).as_millis() > tick_speed {
                emulator.clock();
                last_clock = std::time::Instant::now();
                clear_terminal_screen();
                if terminal_output {
                    draw_terminal_screen(&emulator);
                }
            }
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);
            draw_ports(&emulator, &mut d, &on_texture, &off_texture);
            if show_fps {
                d.draw_text(&d.get_fps().to_string(), 0, 0, 25, Color::WHITE);
            }
        }
    }
}
