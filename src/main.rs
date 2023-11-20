use std::env;
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;

fn main() {
    let mut backlight = Backlight::new().unwrap();
    let args: Vec<String> = env::args().collect();
    if let Some(arg) = args.get(1) {
        let brightness = str::parse::<f32>(arg).expect("arg should be f32");
        let _ = backlight.bright(brightness);
    } else {
        backlight.auto();
    }
}

pub struct Backlight {
    file: File,
    min_brightness: u64,
    max_brightness: u64,
    span: u64,
    current: Option<u64>,
    has_write_permission: bool,
}

impl Backlight {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let path = "/sys/class/backlight/intel_backlight";
        let min_brightness = 0;

        let brightness_path = Path::new(path).join("brightness");
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(brightness_path)?;
        let current_brightness = read(&mut file)?;
        let has_write_permission = write(&mut file, current_brightness).is_ok();

        let max_brightness = fs::read_to_string(Path::new(path).join("max_brightness"))?
            .trim()
            .parse()?;

        Ok(Self {
            file,
            min_brightness,
            max_brightness,
            span: max_brightness - min_brightness,
            current: Some(current_brightness as u64),
            has_write_permission,
        })
    }

    pub fn auto(&mut self) {
        if let Some(current) = self.current {
            // if currently brighter than 2%
            if current > 2 * (self.span / 100) {
                let _ = self.bright(0.0);
                return;
            }
        }
        let _ = self.bright(100.0);
    }

    pub fn bright(&mut self, percent: f32) -> Result<(), Box<dyn Error>> {
        let one_percent = self.span as f32 / 100.0;
        let value = (one_percent * percent) as u64;
        self.set(value)
    }

    pub fn get(&mut self) -> Result<u64, Box<dyn Error>> {
        let value = read(&mut self.file)? as u64;
        self.current = Some(value);
        Ok(value)
    }

    pub fn set(&mut self, value: u64) -> Result<(), Box<dyn Error>> {
        let value = value.clamp(self.min_brightness, self.max_brightness);

        if self.has_write_permission {
            write(&mut self.file, value as f64)?;
        } else {
            return Err(Box::new(std::io::Error::from(ErrorKind::PermissionDenied)));
        }

        self.current = Some(value);
        Ok(())
    }
}

pub fn read(file: &mut File) -> Result<f64, Box<dyn Error>> {
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    file.seek(SeekFrom::Start(0))?;
    Ok(content.trim().parse()?)
}

pub fn write(file: &mut File, value: f64) -> Result<(), Box<dyn Error>> {
    file.write_all(value.to_string().as_bytes())?;
    file.seek(SeekFrom::Start(0))?;
    Ok(())
}
