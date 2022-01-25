use std::{str::FromStr, fmt, path::PathBuf, io::{self, BufRead}, fs::File};
use nix::{self, mount::MsFlags};

#[derive(Debug, PartialEq)]
pub enum FsType {
    Proc,
    Overlay,
    Tmpfs,
    Sysfs,
    Other(String)
}

impl FromStr for FsType {
    
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "proc" => Ok(FsType::Proc),
            "tmpfs" => Ok(FsType::Tmpfs),
            "overlay" => Ok(FsType::Overlay),
            "sysfs" => Ok(FsType::Sysfs),
            &_ => Ok(FsType::Other(s.to_owned()))
        }
    }
}

impl fmt::Display for FsType {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { 
        let fsname = match self {
            FsType::Proc => "proc",
            FsType::Overlay => "overlay",
            FsType::Tmpfs => "tmpfs",
            FsType::Sysfs => "sysfs",
            FsType::Other(s) => s.as_str()
        };
        write!(f, "{}", fsname)
    }

}

#[derive(Debug)]
pub struct MountingPoint {
    pub what: Option<String>,
    pub path: PathBuf,
    pub fstype: Option<FsType>,
    pub options: Option<String>,
    pub flags: Option<MsFlags>,
    pub fatal: Option<bool>,
    pub in_userns: Option<bool>,
    pub use_netns: Option<bool>
}

#[derive(Debug)]
pub struct MTab {
    mounting_points: Vec<MountingPoint>
}

impl MTab {

    pub fn new() -> Self {
        return MTab {
            mounting_points: MTab::get_mounting_points().unwrap()
        }
    }

    pub fn contains(&self, mounting_point: MountingPoint) -> bool {
        let filtered: Vec<&MountingPoint> = self.mounting_points.iter().
            filter(|mts| 
                mts.path == mounting_point.path && mts.fstype == mounting_point.fstype)
            .collect();
        filtered.len() > 0
    }

    pub fn get_mounting_points() -> Result<Vec<MountingPoint>, std::io::Error> {
        let mut results: Vec<MountingPoint> = vec![];
        let mtab = File::open("/etc/mtab")?;
        let reader = io::BufReader::new(mtab);
        for line in reader.lines() {
            let l = line?;
            let parts: Vec<&str> = l.split_whitespace().collect();
            if !parts.is_empty() {
                results.push(MountingPoint {
                    what: Some(parts[0].to_owned()),
                    path: PathBuf::from(parts[1]),
                    fstype: Some(FsType::from_str(parts[2]).unwrap()),
                    options: Some(parts[3].to_owned()),
                    flags: None,
                    fatal: None,
                    in_userns: None,
                    use_netns: None
                })
            }
        }
        Ok(results)
    }

}
