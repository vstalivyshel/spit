use arboard::{Clipboard, ImageData, SetExtLinux};
use screenshots::{image, Screen};
use std::{
    env,
    process::{Command, ExitCode, Stdio},
    time::SystemTime,
};

type AnyError = Box<dyn std::error::Error>;

const SELF_IS_DAEMONIZED: &str = "__self_is_daemonized";

fn usage() -> String {
    format!(
        r#"If no subcommand is provided, it will take a screenshot of the current screen and save it to the system clipboard.
Usage:
    {it} [SUBCOMMAND]

Subcommands:
    help - Show this message.
    save - Try to save a screenshot from the clipboard as a PNG file."#,
        it = env::current_exe()
            .expect("name of the current program")
            .display()
    )
}

fn capture_screenshot() -> Result<image::RgbaImage, AnyError> {
    let screen = Screen::all()?[0];
    let image = screen.capture()?;

    Ok(image)
}

fn screenshot_into_clipboard(clipboard: &mut Clipboard) -> Result<(), AnyError> {
    let image = capture_screenshot()?;

    let image_data = ImageData {
        width: image.width() as usize,
        height: image.height() as usize,
        bytes: image.as_raw().into(),
    };

    if cfg!(target_os = "linux") {
        clipboard.set().wait().image(image_data)?;
    } else {
        clipboard.set_image(image_data)?;
    }

    Ok(())
}

fn get_image_from_clipboard(clipboard: &mut Clipboard) -> Result<image::RgbaImage, AnyError> {
    let ImageData {
        width,
        height,
        bytes,
    } = clipboard.get_image()?;

    let image = image::RgbaImage::from_raw(width as u32, height as u32, bytes.into())
        .expect("valid image size");

    Ok(image)
}

fn save_image_cwd_as_png(image: &image::RgbaImage) -> Result<(), AnyError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    let file_path = format!("{now}.png");
    image.save_with_format(&file_path, image::ImageFormat::Png)?;
    println!("Image from clipboard is saved as \"{file_path}\"");

    Ok(())
}

fn save_image_from_clipboard(clipboard: &mut Clipboard) -> Result<(), AnyError> {
    let image = get_image_from_clipboard(clipboard)?;
    save_image_cwd_as_png(&image)?;

    Ok(())
}

fn run() -> Result<(), AnyError> {
    let mut clipboard = Clipboard::new()?;

    if let Some(subcmd) = env::args().nth(1) {
        match subcmd.as_str() {
            "help" => return Ok(println!("{}", usage())),
            "save" => save_image_from_clipboard(&mut clipboard),
            SELF_IS_DAEMONIZED if cfg!(target_os = "linux") => {
                screenshot_into_clipboard(&mut clipboard)
            }
            _ => Err(format!(r#"Unknown subcommand "{subcmd}"{usage}"#, usage = usage()).into()),
        }?;

        return Ok(());
    }

    if cfg!(target_os = "linux") {
        Command::new(env::current_exe()?)
            .arg(SELF_IS_DAEMONIZED)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .current_dir("/")
            .spawn()?;
    } else {
        screenshot_into_clipboard(&mut clipboard)?;
    }

    Ok(())
}

fn main() -> ExitCode {
    if let Err(e) = run() {
        eprintln!("ERROR: {e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
