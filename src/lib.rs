/**
 * The MIT License
 * Copyright (c) 2022 Guillem Castro
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 */

use std::{str::FromStr, fmt, path::{PathBuf, Path}, io::{self, BufRead}, fs::File};

#[derive(Debug, PartialEq)]
pub enum FsType {
    Proc,
    Overlay,
    Tmpfs,
    Sysfs,
    Btrfs,
    Ext2,
    Ext3,
    Ext4,
    Devtmpfs,
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
            "btrfs" => Ok(FsType::Btrfs),
            "ext2" => Ok(FsType::Ext2),
            "ext3" => Ok(FsType::Ext3),
            "ext4" => Ok(FsType::Ext4),
            "devtmpfs" => Ok(FsType::Devtmpfs),
            _ => Ok(FsType::Other(s.to_string()))
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
            FsType::Btrfs => "btrfs",
            FsType::Ext2 => "ext2",
            FsType::Ext3 => "ext3",
            FsType::Ext4 => "ext4",
            FsType::Devtmpfs => "devtmpfs",
            FsType::Other(ref fsname) => fsname
        };
        write!(f, "{}", fsname)
    }

}

#[derive(Debug)]
pub struct MountingPoint {
    pub what: String,
    pub path: PathBuf,
    pub fstype: FsType,
    pub options: MountOptions,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum ReadWrite {
    ReadOnly,
    ReadWrite
}

#[derive(Debug)]
pub struct MountOptions {
    pub read_write: ReadWrite,
    pub others: Vec<String>
}

impl MountOptions {
    
    pub fn new(options: &str) -> Self {
        let mut read_write = ReadWrite::ReadOnly;
        let mut others = Vec::new();
        for option in options.split(',') {
            match option {
                "ro" => read_write = ReadWrite::ReadOnly,
                "rw" => read_write = ReadWrite::ReadWrite,
                &_ => others.push(option.to_owned())
            }
        }
        MountOptions {
            read_write,
            others
        }
    }

}

#[derive(Debug)]
pub struct MTab {
    pub mounting_points: Vec<MountingPoint>
}

impl MTab {

    pub fn new() -> Result<Self, io::Error> {
        let mut mtab = File::open("/etc/mtab")?;
        return Ok(MTab {
            mounting_points: MTab::read_mounting_points(&mut mtab)?
        })
    }

    pub fn contains(&self, mounting_point: MountingPoint) -> bool {
        let filtered: Vec<&MountingPoint> = self.mounting_points.iter().
            filter(|mts| 
                mts.path == mounting_point.path && mts.fstype == mounting_point.fstype)
            .collect();
        filtered.len() > 0
    }

    pub fn is_mounted<P: AsRef<Path>>(&self, path: P) -> bool {
        let filtered: Vec<&MountingPoint> = self.mounting_points.iter().
            filter(|mts| 
                mts.path == path.as_ref().to_path_buf())
            .collect();
        filtered.len() > 0
    }

    pub fn read_mounting_points(file: &mut dyn std::io::Read) -> Result<Vec<MountingPoint>, std::io::Error> {
        let mut results: Vec<MountingPoint> = vec![];
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            let l = line?;
            let parts: Vec<&str> = l.split_whitespace().collect();
            if !parts.is_empty() {
                results.push(MountingPoint {
                    what: parts[0].to_string(),
                    path: PathBuf::from(parts[1]),
                    fstype: FsType::from_str(parts[2]).unwrap(),
                    options: MountOptions::new(parts[3]),
                })
            }
        }
        Ok(results)
    }

}

// unit tests
#[cfg(test)]
mod test {

    use super::*;

    struct FakeFile {
        s: String,
        read: bool
    }

    impl io::Read for FakeFile {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.read {
                return Ok(0);
            }
            let mut i = 0;
            while i < self.s.len() {
                buf[i] = self.s.as_bytes()[i];
                i += 1;
            }
            self.read = true;
            Ok(buf.len())
        }
    }

    #[test]
    fn test_load_mount_points() {
        let mut file = FakeFile { s: "tmpfs /tmp tmpfs rw,seclabel,nosuid,nodev,size=8026512k,nr_inodes=1048576,inode64 0 0".to_owned(), read: false };
        let munt_points = MTab::read_mounting_points(&mut file).unwrap();
        assert_eq!(munt_points.len(), 1);
        assert_eq!(munt_points[0].what, "tmpfs".to_owned());
        assert_eq!(munt_points[0].path, PathBuf::from("/tmp"));
        assert_eq!(munt_points[0].fstype, FsType::Tmpfs);
    }

    #[test]
    fn test_contains() {
        let mut file = FakeFile { s: "tmpfs /tmp tmpfs rw,seclabel,nosuid,nodev,size=8026512k,nr_inodes=1048576,inode64 0 0".to_owned(), read: false };
        let mtab = MTab { mounting_points: MTab::read_mounting_points(&mut file).unwrap() };
        let mp = MountingPoint {
            what: "tmpfs".to_owned(),
            path: PathBuf::from("/tmp"),
            fstype: FsType::Tmpfs,
            options: MountOptions::new("rw,seclabel,nosuid,nodev,size=8026512k,nr_inodes=1048576,inode64"),
        };
        assert_eq!(mtab.contains(mp), true);
    }

    #[test]
    fn test_is_mounted() {
        let mut file = FakeFile { s: "tmpfs /tmp tmpfs rw,seclabel,nosuid,nodev,size=8026512k,nr_inodes=1048576,inode64 0 0".to_owned(), read: false };
        let mtab = MTab { mounting_points: MTab::read_mounting_points(&mut file).unwrap() };
        assert_eq!(mtab.is_mounted("/tmp"), true);
    }

    #[test]
    fn test_mount_options() {
        let options = MountOptions::new("rw,seclabel,nosuid,nodev,size=8026512k,nr_inodes=1048576,inode64");
        assert_eq!(options.read_write, ReadWrite::ReadWrite);
        assert_ne!(options.others.len(), 0);
        let more_options = MountOptions::new("ro,seclabel,nosuid,nodev,size=8026512k,nr_inodes=1048576,inode64");
        assert_eq!(more_options.read_write, ReadWrite::ReadOnly);
        assert_ne!(more_options.others.len(), 0);
    }
}