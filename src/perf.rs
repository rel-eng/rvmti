// Copyright 2018 rel-eng
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:
//   1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.
//   2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.
//   3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::io::{self, Read, Write, ErrorKind};
use std::env;
use std::fs::{File, DirBuilder, OpenOptions};
use std::path::{Path, PathBuf};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::os::unix::io::AsRawFd;
use std::ptr;

use log::{debug, error};
use thiserror::Error;
use byteorder::{ReadBytesExt, NativeEndian, ByteOrder};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use nix::sys::mman::{mmap, munmap, ProtFlags, MapFlags};
use chrono::prelude::Local;

use super::rvmti;
use super::demangle;

pub fn create_dump_dir() -> Result<PathBuf, CreteDumpDirError> {
    let cur_dir = env::current_dir().map_err(CreteDumpDirError::IoError)?;
    let jit_dir = cur_dir.join(".debug").join("jit");
    let _ = DirBuilder::new().recursive(true).mode(0o755).create(&jit_dir).map_err(CreteDumpDirError::IoError)?;
    let date = Local::now().format("%Y%m%d").to_string();
    let prefix = format!("java-jit-{}", date);
    let mut rng = thread_rng();
    for _ in 0u32..(1u32 << 31) {
        let suffix: String = (&mut rng).sample_iter(&Alphanumeric).take(8).collect();
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
                let mapped_file = unsafe {
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
                                      _class_source_file_name: Option<String>, address: usize, length: usize,
                                      _line_numbers: Option<Vec<rvmti::LineNumberEntry>>,
                                      _address_locations: Option<Vec<rvmti::AddressLocationEntry>>,
                                      _stack_info: Option<Vec<super::StackInfo>>,
                                      code_index: u64, timestamp: i64, code: &Vec<u8>) -> Result<(), WriteRecordError>
    {
        let pid = get_pid();
        let tid = get_tid();
        let combined_name = demangle::MethodType::new(&name.signature)
            .and_then(|mt| demangle::ClassType::new(&class_signature.signature)
                .map(|cs| mt.display_as_method_definition(&name.name, &cs)))
            .unwrap_or_else(|_| format!("{}.{}{}", class_signature.signature, name.name, name.signature));
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

    pub fn write_line_numbers(&mut self, name: &rvmti::MethodName, class_signature: &rvmti::ClassSignature,
                          class_source_file_name: &Option<String>, address: usize,
                          line_numbers: &Option<Vec<rvmti::LineNumberEntry>>,
                          address_locations: &Option<Vec<rvmti::AddressLocationEntry>>,
                          stack_info: &Option<Vec<super::StackInfo>>,
                          timestamp: i64) -> Result<(), WriteRecordError>
    {
        if stack_info.is_some() {
            // Inlining info is available
            let stack = stack_info.as_ref().unwrap();
            if !stack.is_empty() {
                return self.write_line_numbers_with_stack_info(stack, address, timestamp);
            }
            return Ok(());
        }
        if line_numbers.is_some() && address_locations.is_some() && class_source_file_name.is_some() {
            // Line numbers info is available, but no inlining info present
            return self.write_line_numbers_without_stack_info(name, class_signature,
                                                  class_source_file_name.as_ref().unwrap(),
                                                  line_numbers.as_ref().unwrap(),
                                                  address_locations.as_ref().unwrap(),
                                                  address, timestamp);
        }
        // No line numbers info present
        Ok(())
    }

    fn write_line_numbers_without_stack_info(&mut self, _name: &rvmti::MethodName,
                                             class_signature: &rvmti::ClassSignature,
                                             class_source_file_name: &str,
                                             line_numbers: &Vec<rvmti::LineNumberEntry>,
                                             address_locations: &Vec<rvmti::AddressLocationEntry>,
                                             address: usize, timestamp: i64) -> Result<(), WriteRecordError>
    {
        let mut record_size = 32u32;
        let mut entries_count = 0u64;
        let class_location = demangle::ClassType::new(&class_signature.signature)
            .map(|v| v.package_as_file_path(class_source_file_name))
            .unwrap_or_else(|_| class_source_file_name.to_owned());
        for location in address_locations {
            let maybe_line_number = self.find_line_number_entry(location.location as i32, line_numbers);
            if maybe_line_number.is_some() {
                entries_count += 1u64;
                let name_bytes = class_location.as_bytes();
                record_size += 17u32 + name_bytes.len() as u32;
            }
        }
        let mut record = [0u8; 32];
        let first_record_part = [
            2u32, // id = JIT_CODE_DEBUG_INFO
            record_size, // record size
        ];
        let second_record_part = [
            timestamp as u64, // timestamp
            address as u64, // code_addr
            entries_count, // number of entries
        ];
        NativeEndian::write_u32_into(&first_record_part, &mut record[0..8]);
        NativeEndian::write_u64_into(&second_record_part, &mut record[8..32]);
        let _ = self.file.write(&record).map_err(WriteRecordError::IoError)?;
        for location in address_locations {
            let line = self.find_line_number_entry(location.location as i32, line_numbers);
            if !line.is_some() {
                continue;
            }
            let mut entry = [0u8; 16];
            let first_entry_part = [
                location.start_address as u64, // addr
            ];

            let second_entry_part = [
                line.unwrap().line_number, //lineno
                0i32, //discrim
            ];
            let name_bytes = class_location.as_bytes();
            NativeEndian::write_u64_into(&first_entry_part, &mut entry[0..8]);
            NativeEndian::write_i32_into(&second_entry_part, &mut entry[8..16]);
            let _ = self.file.write(&entry).map_err(WriteRecordError::IoError)?;
            let _ = self.file.write(&name_bytes).map_err(WriteRecordError::IoError)?;
            let _ = self.file.write(&[0u8; 1]).map_err(WriteRecordError::IoError)?;
        }
        Ok(())
    }

    fn write_line_numbers_with_stack_info(&mut self, stack_info: &Vec<super::StackInfo>,
                                          address: usize, timestamp: i64) -> Result<(), WriteRecordError>
    {
        let (record_size, entries_count) = self.pre_calc_record_size(stack_info);
        if entries_count == 0u64 {
            // No suitable entries, return right away
            return Ok(());
        }
        let mut record = [0u8; 32];
        let first_record_part = [
            2u32, // id = JIT_CODE_DEBUG_INFO
            record_size, // record size
        ];
        let second_record_part = [
            timestamp as u64, // timestamp
            address as u64, // code_addr
            entries_count, // number of entries
        ];
        NativeEndian::write_u32_into(&first_record_part, &mut record[0..8]);
        NativeEndian::write_u64_into(&second_record_part, &mut record[8..32]);
        let _ = self.file.write(&record).map_err(WriteRecordError::IoError)?;
        for info in stack_info {
            if info.stack_frames.is_empty() {
                continue;
            }
            let maybe_frame = self.find_frame(&info.stack_frames);
            if maybe_frame.is_some() {
                let frame = maybe_frame.unwrap();
                let method = &frame.method;
                let maybe_line_numbers = &method.line_numbers;
                if !maybe_line_numbers.is_some() {
                    continue;
                }

                let line_numbers = maybe_line_numbers.as_ref().unwrap();
                let line = self.find_line_number_entry(frame.byte_code_index, line_numbers);
                if !line.is_some() {
                    continue;
                }
                let mut entry = [0u8; 16];
                let first_entry_part = [
                    info.pc_address as u64, // addr
                ];

                let second_entry_part = [
                    line.unwrap().line_number, //lineno
                    0i32, //discrim
                ];
                let class_location = method.class.source_file_name.as_ref()
                    .map(|n| demangle::ClassType::new(&method.class.signature.signature)
                    .map(|v| v.package_as_file_path(&n)).unwrap_or_else(|_| n.to_owned()))
                    .unwrap_or_else(|| "".to_owned());
                let name_bytes = class_location.as_bytes();
                NativeEndian::write_u64_into(&first_entry_part, &mut entry[0..8]);
                NativeEndian::write_i32_into(&second_entry_part, &mut entry[8..16]);
                let _ = self.file.write(&entry).map_err(WriteRecordError::IoError)?;
                let _ = self.file.write(&name_bytes).map_err(WriteRecordError::IoError)?;
                let _ = self.file.write(&[0u8; 1]).map_err(WriteRecordError::IoError)?;
            }
        }
        Ok(())
    }

    fn pre_calc_record_size(&self, stack_info: &Vec<super::StackInfo>) -> (u32, u64) {
        let mut record_size = 32u32;
        let mut entries_count = 0u64;
        for info in stack_info {
            if info.stack_frames.is_empty() {
                // No line numbers info, skip
                continue;
            }
            let frame = self.find_frame(&info.stack_frames);
            if frame.is_some() {
                entries_count += 1u64;
                let method = &frame.unwrap().method;
                let class_location = method.class.source_file_name.as_ref()
                    .map(|n| demangle::ClassType::new(&method.class.signature.signature)
                    .map(|v| v.package_as_file_path(&n)).unwrap_or_else(|_| n.to_owned()))
                    .unwrap_or_else(|| "".to_owned());
                let name_bytes = class_location.as_bytes();
                record_size += 17u32 + name_bytes.len() as u32;
            }
        }
        (record_size, entries_count)
    }

    fn find_frame<'a>(&self, stack_frames: &'a Vec<super::StackFrameInfo>) -> Option<&'a super::StackFrameInfo> {
        // Take first suitable frame
        stack_frames.iter().find(|&f| {
            let method = &f.method;
            let line_numbers = &method.line_numbers;
            let class = &method.class;
            // not native, has line numbers info, has source file name
            let has_source_info = !method.native_method && line_numbers.is_some()
                && !line_numbers.as_ref().unwrap().is_empty() && class.source_file_name.is_some();
            if !has_source_info {
                return false;
            } else {
                // suitable line number is found
                let line_num = line_numbers.as_ref().and_then(|nums| self.find_line_number_entry(f.byte_code_index, &nums));
                return line_num.is_some();
            }
        })
    }

    fn find_line_number_entry<'a>(&self, byte_code_index: i32,
                              line_numbers: &'a Vec<rvmti::LineNumberEntry>) -> Option<&'a rvmti::LineNumberEntry>
    {
        let mut result: Option<&rvmti::LineNumberEntry> = None;
        for entry in line_numbers {
            if entry.start_location as i64 <= byte_code_index as i64 {
                result = Some(&entry)
            } else {
                break;
            }
        }
        return result;
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

#[derive(Error, Debug)]
pub enum WriteHeaderError {
    #[error("I/O error: {0}")]
    IoError(#[source] io::Error),
    #[error("Failed to get e_machine value for the current process: {0}")]
    FailedToGetEMachine(#[source] GetEMachineError),
    #[error("Failed to get current timestamp: {0}")]
    FailedToGetTimestamp(#[source] nix::errno::Errno),
}

#[derive(Error, Debug)]
pub enum WriteRecordError {
    #[error("I/O error: {0}")]
    IoError(#[source] io::Error),
    #[error("Failed to get current timestamp: {0}")]
    FailedToGetTimestamp(#[source] nix::errno::Errno),
}

#[derive(Error, Debug)]
pub enum GetEMachineError {
    #[error("I/O error: {0}")]
    IoError(#[source] io::Error),
    #[error("Not an ELF file")]
    NotAnElfFile,
}

#[derive(Error, Debug)]
pub enum CreteDumpDirError {
    #[error("I/O error: {0}")]
    IoError(#[source] io::Error),
    #[error("Too many failed attempts to create random temp dir")]
    DirNameConflict,
}

#[derive(Error, Debug)]
pub enum NewDumpFileError {
    #[error("I/O error: {0}")]
    IoError(#[source] io::Error),
    #[error("MMap error: {0}")]
    MmapError(#[source] nix::Error),
    #[error("SysConf error: {0}")]
    SysconfError(#[source] nix::Error),
    #[error("Memory page size is unknown")]
    UnknownPageSize,
}
