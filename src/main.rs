use evdev::{
    uinput::VirtualDevice, AttributeSet, EventType, InputEvent, KeyCode, KeyEvent, RelativeAxisCode,
};

use std::{
    env,
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    thread::sleep,
    time::Duration,
};

fn main() -> std::io::Result<()> {
    let mut keys = AttributeSet::<KeyCode>::new();
    keys.insert(KeyCode::BTN_LEFT);

    let mut device = VirtualDevice::builder()?
        .name("fake-input")
        .with_relative_axes(&AttributeSet::from_iter([
            RelativeAxisCode::REL_X,
            RelativeAxisCode::REL_Y,
        ]))?
        .with_keys(&keys)?
        .build()
        .unwrap();

    println!("Set position");
    sleep(Duration::from_secs(4));
    let (move_to_x, move_to_y) = get_cursor_pos();
    println!("Done");

    sleep(Duration::from_secs(3));

    let (current_x, current_y) = get_cursor_pos();
    let move_x = InputEvent::new(
        EventType::RELATIVE.0,
        RelativeAxisCode::REL_X.0,
        move_to_x - current_x,
    );
    let move_y = InputEvent::new(
        EventType::RELATIVE.0,
        RelativeAxisCode::REL_Y.0,
        move_to_y - current_y,
    );
    device.emit(&[move_x, move_y]).unwrap();

    left_click(&mut device);

    Ok(())
}

fn left_click(device: &mut VirtualDevice) {
    let code = KeyCode::BTN_LEFT.code();
    let down_event = *KeyEvent::new(KeyCode(code), 1);
    device.emit(&[down_event]).unwrap();
    sleep(Duration::from_millis(1));
    let up_event = InputEvent::new(EventType::KEY.0, code, 0);
    device.emit(&[up_event]).unwrap();
}

fn get_cursor_pos() -> (i32, i32) {
    let runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap();
    let his = env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
    let socket_path = PathBuf::from(runtime_dir)
        .join("hypr")
        .join(his)
        .join(".socket.sock")
        .canonicalize()
        .unwrap();
    let mut stream = UnixStream::connect(&socket_path).unwrap();
    let data = "/cursorpos";
    stream.write_all(data.as_bytes()).unwrap();
    stream.flush().unwrap();
    let mut buf = String::new();
    buf.clear();
    stream.read_to_string(&mut buf).unwrap();
    let (x, y) = buf.split_once(", ").unwrap();
    (x.parse::<i32>().unwrap(), y.parse::<i32>().unwrap())
}
