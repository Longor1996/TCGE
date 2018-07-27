use std::path::Path;
use std::path::PathBuf;
use std::fs;
use std::io::{self, Read};
use std::ffi;

#[derive(Debug, Fail)]
pub enum ResError {
    #[fail(display = "I/O error")]
    Io(#[cause] io::Error),

    #[fail(display = "Failed to read CString from file that contains 0")]
    FileContainsNil,

    #[fail(display = "Failed get executable path")]
    FailedToGetExePath,
}

impl From<io::Error> for ResError {
    fn from(other: io::Error) -> Self {
        ResError::Io(other)
    }
}

pub struct Resources {
    root_path: PathBuf,
}

impl Resources {
    pub fn from_exe_path() -> Result<Resources, ResError> {
        let exe_file_name = ::std::env::current_exe()
            .map_err(|_| ResError::FailedToGetExePath)?;
        let exe_path = exe_file_name.parent()
            .ok_or(ResError::FailedToGetExePath)?;
        Ok(Resources {
            root_path: exe_path.into()
        })
    }

    pub fn load_cstring(&self, resource_name: &str) -> Result<ffi::CString, ResError> {
        let mut file = fs::File::open(
            resource_name_to_path(&self.root_path,resource_name)
        )?;

        // allocate buffer of the same size as file
        let mut buffer: Vec<u8> = Vec::with_capacity(
            file.metadata()?.len() as usize + 1
        );
        file.read_to_end(&mut buffer)?;

        // check for nul byte
        if buffer.iter().find(|i| **i == 0).is_some() {
            return Err(ResError::FileContainsNil);
        }

        Ok(unsafe { ffi::CString::from_vec_unchecked(buffer) })
    }

    pub fn open(&self, resource_name: &str) -> Result<fs::File, io::Error> {
        fs::File::open(
            resource_name_to_path(
                &self.root_path,
                resource_name
            )
        )
    }
}

impl Drop for Resources {
    fn drop(&mut self) {
        // later
    }
}

fn resource_name_to_path(root_dir: &Path, location: &str) -> PathBuf {
    let mut path: PathBuf = root_dir.into();

    path = path.join("assets/");

    // TODO: Location package splitting.

    for part in location.split("/") {
        path = path.join(part);
    }

    path
}