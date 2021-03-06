extern crate libc;
extern crate taglib_sys;

// std library imports
use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::NulError;
use std::os::raw::c_void;
use std::os::raw::c_char;
use std::str::Utf8Error;
use std::path::PathBuf;

// taglib-sys imports
use taglib_sys::*;

/* Define a file interface */
#[derive(Debug)]
pub struct TagLibFile {
    file_handle: *mut TagLib_File,
    tag: TagLibTag,
}

/* Various kinds of errors that we can get from using a file */
#[derive(Debug)]
pub enum FileError {
    OpenFailure,
    SaveFailure,
    PathAsString,
    NullPathString(NulError),
    InvalidTagFile
}

impl TagLibFile {

    /* Open a file with tag information */
    pub fn new<P: Into<PathBuf>>(filename: P) -> Result<TagLibFile, FileError> {
        // get the filename as a string, then a c string
        let cs_filename = filename
            .into()
            .to_str()
            .ok_or(FileError::PathAsString)
            .and_then(|filename| {
                CString::new(filename).map_err(|err| FileError::NullPathString(err))
            })?;

        unsafe {
            // start off by setting the string management options 
            // this does mean that we need to manually free all the strings that get returned to us, however.
            taglib_set_string_management_enabled(false as i32);
            // try to open the file using the ffi
            let file_ptr = taglib_file_new(cs_filename.as_ptr());
            // Todo: Should the struct member be a reference instead?
            if file_ptr.is_null() {
                return Err(FileError::OpenFailure);
            } else {
                // Check to see if the tag file is valid (true/false as int)
                if taglib_file_is_valid(file_ptr) == 0 { 
                    return Err(FileError::InvalidTagFile)
                }
                // pub fn taglib_file_is_valid(file: *const TagLib_File) -> ::std::os::raw::c_int;
                // Get the tag. We want to do this here, so that any references to it only live as long as the file (which is dropped through the drop trait)
                let tag_ptr = taglib_file_tag(file_ptr);
                return Ok(TagLibFile {
                    file_handle: file_ptr,
                    tag: TagLibTag::from_ptr(tag_ptr),
                });
            }
        }
    }

    pub fn save(self: &Self) -> Result<(), FileError> { 
        unsafe {
            let status_code = taglib_file_save(self.file_handle);
            // status code returns true on success, so compare with 0/non-zero
            if status_code == 0 { 
                Err(FileError::SaveFailure)
            } else { 
                Ok(())
            }
        }
    }

    // return a reference to the tag that only lives as long as the file
    pub fn tag(self: &Self) -> &TagLibTag { 
        &self.tag
    }
}

impl Drop for TagLibFile {
    fn drop(&mut self) {
        // free the taglib file!
        unsafe {
            taglib_file_free(self.file_handle);
        }
    }
}

type StringReadError = Result<String, Utf8Error>;

type StringWriteError = Result<(), NulError>; 

#[derive(Debug)]
pub struct TagLibTag {
    tag: *mut TagLib_Tag
}

// Todo: should this be merged with taglib file?
impl TagLibTag { 
    pub fn from_ptr(ptr: *mut TagLib_Tag) -> TagLibTag { 
        TagLibTag { tag: ptr }
    }

    fn read_and_parse(c_string_pointer: *mut c_char) -> StringReadError {
        unsafe {
        let str_slice = CStr::from_ptr(c_string_pointer);
            // try and parse that ptr into a string
            let str_res : StringReadError = str_slice.to_str().map(|s| s.to_owned());
            // free the pointer - TODO: Make this optional!
            taglib_free(c_string_pointer as *mut c_void);
            // and return the owned string
            str_res
        }
    }

    pub fn title(self: &Self) -> StringReadError {
        unsafe {
            Self::read_and_parse(taglib_tag_title(self.tag))
        }
    }

    pub fn artist(self: &Self) -> StringReadError {
        unsafe {
            Self::read_and_parse(taglib_tag_artist(self.tag))
        }
    }

    pub fn album(self: &Self) -> StringReadError {
        unsafe {
            Self::read_and_parse(taglib_tag_album(self.tag))
        }
    }

    pub fn comment(self: &Self) -> StringReadError {
        unsafe {
            Self::read_and_parse(taglib_tag_comment(self.tag))
        }
    }

    pub fn genre(self: &Self) -> StringReadError {
        unsafe {
            Self::read_and_parse(taglib_tag_genre(self.tag))
        }
    }

    pub fn year(self: &Self) -> Option<u32> {
        unsafe {
            match taglib_tag_year(self.tag) {
                0 => None,
                v => Some(v)
            }
        }
    }

    pub fn track(self: &Self) -> Option<u32> {
        unsafe {
            match taglib_tag_track(self.tag) {
                0 => None,
                v => Some(v)
            }
        }
    }

    pub fn bpm(self: &Self) -> Option<u32> {
        unsafe {
            match taglib_tag_bpm(self.tag) {
                0 => None,
                v => Some(v)
            }
        }
    }

    pub fn set_title(self: &Self, title: &str) -> StringWriteError {
        unsafe {
            CString::new(title).map(|cstr| {
                let title_ptr = cstr.as_ptr();
                taglib_tag_set_title(self.tag, title_ptr);
            })
        }
    }

    pub fn set_artist(self: &Self, artist: &str) -> StringWriteError {
        unsafe {
            CString::new(artist).map(|cstr| {
                let artist_ptr = cstr.as_ptr();
                taglib_tag_set_artist(self.tag, artist_ptr);
            })
        }
    }

    pub fn set_album(self: &Self, album: &str) -> StringWriteError {
        unsafe {
            CString::new(album).map(|cstr| {
                let album_ptr = cstr.as_ptr();
                taglib_tag_set_album(self.tag, album_ptr);
            })
        }
    }

    pub fn set_comment(self: &Self, comment: &str) -> StringWriteError {
        unsafe {
            CString::new(comment).map(|cstr| {
                let comment_ptr = cstr.as_ptr();
                taglib_tag_set_comment(self.tag, comment_ptr);
            })
        }
    }

    pub fn set_genre(self: &Self, genre: &str) -> StringWriteError {
        unsafe {
            CString::new(genre).map(|cstr| {
                let genre_ptr = cstr.as_ptr();
                taglib_tag_set_genre(self.tag, genre_ptr);
            })
        }
    }

    pub fn set_year(self: &Self, year: u32) -> () { 
        unsafe {
            taglib_tag_set_year(self.tag, year);
        }
    }

    pub fn set_track(self: &Self, track: u32) -> () { 
        unsafe {
            taglib_tag_set_track(self.tag, track);
        }
    }
}
