// Copyright 2018 rel-eng
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:
//   1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.
//   2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.
//   3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

extern crate nix;
extern crate libc;
extern crate byteorder;
extern crate rand;
extern crate time;
extern crate log;

use std::io::{self, Read, Write, ErrorKind};
use std::env;
use std::fs::{File, DirBuilder, OpenOptions};
use std::path::{Path, PathBuf};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::os::unix::io::AsRawFd;
use std::ptr;

use self::byteorder::{ReadBytesExt, NativeEndian, ByteOrder};
use self::rand::{thread_rng, Rng};
use self::nix::sys::mman::{mmap, munmap, ProtFlags, MapFlags};

use super::rvmti;

pub fn create_dump_dir() -> Result<PathBuf, CreteDumpDirError> {
    let cur_dir = env::current_dir().map_err(CreteDumpDirError::IoError)?;
    let jit_dir = cur_dir.join(".debug").join("jit");
    let _ = DirBuilder::new().recursive(true).mode(0o755).create(&jit_dir).map_err(CreteDumpDirError::IoError)?;
    let date = time::strftime("%Y%m%d", &time::now()).map_err(CreteDumpDirError::DateFormatError)?;
    let prefix = format!("java-jit-{}", date);
    let mut rng = thread_rng();
    for _ in 0u32..(1u32 << 31) {
        let suffix: String = rng.gen_ascii_chars().take(8).collect();
        let dir = format!("{}.{}", prefix, suffix);
        let path = jit_dir.join(&dir);
        match DirBuilder::new().mode(0o700).create(&path) {
            Ok(_) => {
                debug!("Jit dump directory: {:?}", &path);
                return Ok(path)
            },
            Err(ref e) if e.kind() == ErrorKind::AlreadyExists => {}
            Err(e) => return Err(CreteDumpDirError::IoError(e)),
        }
    }
    Err(CreteDumpDirError::DirNameConflict)
}

#[derive(Debug)]
pub struct DumpFile {
    file: File,
    mapped_file: *mut libc::c_void,
    map_size: libc::size_t,
}

// Required for global thread-safe store of initialized environments
unsafe impl Send for DumpFile {}

impl DumpFile {

    pub fn new<P: AsRef<Path>>(path: P) -> Result<DumpFile, NewDumpFileError> {
        let page_size = nix::unistd::sysconf(nix::unistd::SysconfVar::PAGE_SIZE)
            .map_err(NewDumpFileError::SysconfError)?;
        match page_size {
            Some(size) => {
                let map_size = size as libc::size_t;
                let pid = get_pid();
                let file_name = format!("jit-{}.dump", pid);
                let file_path = path.as_ref().join(file_name);
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create_new(true)
                    .mode(0o0666)
                    .open(&file_path)
                    .map_err(NewDumpFileError::IoError)?;
                debug!("Jit dump file: {:?}", &file_path);
                let mut prot_flags = ProtFlags::empty();
                prot_flags.insert(ProtFlags::PROT_READ);
                prot_flags.insert(ProtFlags::PROT_EXEC);
                let mut mapped_file = unsafe {
                    mmap(ptr::null_mut() as *mut libc::c_void, map_size, prot_flags,
                         MapFlags::MAP_PRIVATE, file.as_raw_fd(), 0)
                        .map_err(NewDumpFileError::MmapError)?
                };

                return Ok(DumpFile{file, mapped_file, map_size})
            },
            None => return Err(NewDumpFileError::UnknownPageSize),
        }
    }

    pub fn write_header(&mut self) -> Result<(), WriteHeaderError> {
        let e_machine = get_e_machine().map_err(WriteHeaderError::FailedToGetEMachine)?;
        let timestamp = get_timestamp().map_err(WriteHeaderError::FailedToGetTimestamp)?;
        let pid = get_pid();
        let mut header = [0u8; 40];
        let first_header_part = [
            0x4a695444u32, // magic
            1u32, // version
            40u32, // header size
            e_machine as u32, // e_machine
            0u32, // reserved
            pid as u32, // pid
        ];
        let second_header_part = [
            timestamp as u64, // timestamp
            0u64, // flags
        ];
        NativeEndian::write_u32_into(&first_header_part, &mut header[0..24]);
        NativeEndian::write_u64_into(&second_header_part, &mut header[24..]);
        let _ = self.file.write(&header).map_err(WriteHeaderError::IoError)?;
        Ok(())
    }

    pub fn write_code_close_record(&mut self) -> Result<(), WriteRecordError> {
        let timestamp = get_timestamp().map_err(WriteRecordError::FailedToGetTimestamp)?;
        let mut record = [0u8; 16];
        let first_record_part = [
            3u32, // id = JIT_CODE_CLOSE
            16u32, // record size
        ];
        let second_record_part = [
            timestamp as u64, // timestamp
        ];
        NativeEndian::write_u32_into(&first_record_part, &mut record[0..8]);
        NativeEndian::write_u64_into(&second_record_part, &mut record[8..]);
        let _ = self.file.write(&record).map_err(WriteRecordError::IoError)?;
        Ok(())
    }

    pub fn write_jit_code_load(&mut self, name: String, address: usize, length: usize, code_index: u64,
                               timestamp: i64, code: &Vec<u8>) -> Result<(), WriteRecordError> {
        let pid = get_pid();
        let tid = get_tid();
        let name_bytes = name.as_bytes();
        let mut record = [0u8; 56];
        let first_record_part = [
            0u32, // id = JIT_CODE_LOAD
            (56 + name_bytes.len() + 1 + length) as u32, // record size
        ];
        let second_record_part = [
            timestamp as u64, // timestamp
        ];
        let third_record_part = [
            pid as u32, // pid
            tid as u32, // tid
        ];
        let fourth_record_part = [
            address as u64, // vma
            address as u64, // code_addr
            length as u64, // code_size
            code_index, // code_index
        ];
        NativeEndian::write_u32_into(&first_record_part, &mut record[0..8]);
        NativeEndian::write_u64_into(&second_record_part, &mut record[8..16]);
        NativeEndian::write_u32_into(&third_record_part, &mut record[16..24]);
        NativeEndian::write_u64_into(&fourth_record_part, &mut record[24..56]);
        let _ = self.file.write(&record).map_err(WriteRecordError::IoError)?;
        let _ = self.file.write(&name_bytes).map_err(WriteRecordError::IoError)?;
        let _ = self.file.write(&[0u8; 1]).map_err(WriteRecordError::IoError)?;
        let _ = self.file.write(&code).map_err(WriteRecordError::IoError)?;
        Ok(())
    }

    pub fn write_compiled_method_load(&mut self, name: rvmti::MethodName, class_signature: rvmti::ClassSignature,
                                      class_source_file_name: Option<String>, address: usize, length: usize,
                                      line_numbers: Option<Vec<rvmti::LineNumberEntry>>,
                                      address_locations: Option<Vec<rvmti::AddressLocationEntry>>,
                                      stack_info: Option<Vec<super::StackInfo>>,
                                      code_index: u64, timestamp: i64, code: &Vec<u8>) -> Result<(), WriteRecordError>
    {
        let pid = get_pid();
        let tid = get_tid();
        let combined_name = format!("{}.{}{}", class_signature.signature, name.name, name.signature);
        let name_bytes = combined_name.as_bytes();
        let mut record = [0u8; 56];
        let first_record_part = [
            0u32, // id = JIT_CODE_LOAD
            (56 + name_bytes.len() + 1 + length) as u32, // record size
        ];
        let second_record_part = [
            timestamp as u64, // timestamp
        ];
        let third_record_part = [
            pid as u32, // pid
            tid as u32, // tid
        ];
        let fourth_record_part = [
            address as u64, // vma
            address as u64, // code_addr
            length as u64, // code_size
            code_index, // code_index
        ];
        NativeEndian::write_u32_into(&first_record_part, &mut record[0..8]);
        NativeEndian::write_u64_into(&second_record_part, &mut record[8..16]);
        NativeEndian::write_u32_into(&third_record_part, &mut record[16..24]);
        NativeEndian::write_u64_into(&fourth_record_part, &mut record[24..56]);
        let _ = self.file.write(&record).map_err(WriteRecordError::IoError)?;
        let _ = self.file.write(&name_bytes).map_err(WriteRecordError::IoError)?;
        let _ = self.file.write(&[0u8; 1]).map_err(WriteRecordError::IoError)?;
        let _ = self.file.write(&code).map_err(WriteRecordError::IoError)?;
        Ok(())
    }

}

impl Drop for DumpFile {

    fn drop(&mut self) {
        let result = unsafe {
            munmap(self.mapped_file, self.map_size)
        };
        match result {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to unmap dump file: {}", e);
            }
        };
    }

}

pub fn get_timestamp() -> Result<i64, nix::errno::Errno> {
    let mut ts: libc::timespec = libc::timespec {tv_sec: 0, tv_nsec: 0};
    let result = unsafe {
        libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts)
    };
    if result == 0 {
        return Ok(ts.tv_sec as i64 * 1000000000i64 + ts.tv_nsec as i64);
    } else {
        return Err(nix::errno::Errno::last());
    }
}

fn get_tid() -> i32 {
    libc::pid_t::from(nix::unistd::gettid()) as i32
}

fn get_pid() -> i32 {
    libc::pid_t::from(nix::unistd::Pid::this()) as i32
}

fn get_e_machine() -> Result<u16, GetEMachineError> {
    let mut f = File::open("/proc/self/exe").map_err(GetEMachineError::IoError)?;
    let mut id = [0; 16];
    let _ = f.read_exact(&mut id).map_err(GetEMachineError::IoError)?;
    if id[0] != 0x7f || id[1] != 0x45 || id[2] != 0x4c || id[3] != 0x46 {
        return Err(GetEMachineError::NotAnElfFile);
    }
    let mut info = [0u16; 2];
    let _ = f.read_u16_into::<NativeEndian>(&mut info).map_err(GetEMachineError::IoError)?;
    Ok(info[1])
}

#[derive(Fail, Debug)]
pub enum WriteHeaderError {
    #[fail(display = "I/O error: {}", _0)]
    IoError(#[cause] io::Error),
    #[fail(display = "Failed to get e_machine value for the current process: {}", _0)]
    FailedToGetEMachine(#[cause] GetEMachineError),
    #[fail(display = "Failed to get current timestamp: {}", _0)]
    FailedToGetTimestamp(#[cause] nix::errno::Errno),
}

#[derive(Fail, Debug)]
pub enum WriteRecordError {
    #[fail(display = "I/O error: {}", _0)]
    IoError(#[cause] io::Error),
    #[fail(display = "Failed to get current timestamp: {}", _0)]
    FailedToGetTimestamp(#[cause] nix::errno::Errno),
}

#[derive(Fail, Debug)]
pub enum GetEMachineError {
    #[fail(display = "I/O error: {}", _0)]
    IoError(#[cause] io::Error),
    #[fail(display = "Not an ELF file")]
    NotAnElfFile,
}

#[derive(Fail, Debug)]
pub enum CreteDumpDirError {
    #[fail(display = "I/O error: {}", _0)]
    IoError(#[cause] io::Error),
    #[fail(display = "Date format error: {}", _0)]
    DateFormatError(#[cause] time::ParseError),
    #[fail(display = "Too many failed attempts to create random temp dir")]
    DirNameConflict,
}

#[derive(Fail, Debug)]
pub enum NewDumpFileError {
    #[fail(display = "I/O error: {}", _0)]
    IoError(#[cause] io::Error),
    #[fail(display = "MMap error: {}", _0)]
    MmapError(#[cause] nix::Error),
    #[fail(display = "SysConf error: {}", _0)]
    SysconfError(#[cause] nix::Error),
    #[fail(display = "Memory page size is unknown")]
    UnknownPageSize,
}
