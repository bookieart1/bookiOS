/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSFileManager` etc.

use super::{ns_array, ns_string, NSInteger, NSUInteger};
use crate::dyld::{export_c_func, ConstantExports, FunctionExports, HostConstant};
use crate::frameworks::foundation::ns_error::{
    NSCocoaErrorDomain, NSFilePathErrorKey, NSFileReadInvalidFileNameError,
    NSFileReadNoSuchFileError, NSFileReadUnknownError, NSFileWriteFileExistsError,
    NSFileWriteInvalidFileNameError, NSFileWriteUnknownError,
};
use crate::frameworks::foundation::ns_string::get_static_str;
use crate::fs::{FsError, FsNodeType, GuestPath, GuestPathBuf};
use crate::mem::{ConstPtr, MutPtr, Ptr};
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, release, retain, ClassExports, HostObject,
};
use crate::Environment;

type NSSearchPathDirectory = NSUInteger;
const NSApplicationDirectory: NSSearchPathDirectory = 1;
const NSLibraryDirectory: NSSearchPathDirectory = 5;
const NSDocumentDirectory: NSSearchPathDirectory = 9;
const NSCachesDirectory: NSSearchPathDirectory = 13;
const NSApplicationSupportDirectory: NSSearchPathDirectory = 14;
const NSAllApplicationsDirectory: NSSearchPathDirectory = 100;

type NSSearchPathDomainMask = NSUInteger;
const NSUserDomainMask: NSSearchPathDomainMask = 1;

pub const NSFileModificationDate: &str = "NSFileModificationDate";
pub const NSFileCreationDate: &str = "NSFileCreationDate";
pub const NSFileSize: &str = "NSFileSize";
pub const NSFilePosixPermissions: &str = "NSFilePosixPermissions";
const NSFileSystemFreeSize: &str = "NSFileSystemFreeSize";
pub const NSFileType: &str = "NSFileType";
pub const NSFileTypeDirectory: &str = "NSFileTypeDirectory";
pub const NSFileTypeRegular: &str = "NSFileTypeRegular";
const NSFileOwnerAccountName: &str = "NSFileOwnerAccountName";
const NSFileOwnerAccountID: &str = "NSFileOwnerAccountID";
const NSFileGroupOwnerAccountName: &str = "NSFileGroupOwnerAccountName";
const NSFileGroupOwnerAccountID: &str = "NSFileGroupOwnerAccountID";
const NSFileHFSCreatorCode: &str = "NSFileHFSCreatorCode";
const NSFileSystemSize: &str = "NSFileSystemSize";
const NSFileTypeSymbolicLink: &str = "NSFileTypeSymbolicLink";

pub const CONSTANTS: ConstantExports = &[
    (
        "_NSFileOwnerAccountName",
        HostConstant::NSString(NSFileOwnerAccountName),
    ),
    (
        "_NSFileOwnerAccountID",
        HostConstant::NSString(NSFileOwnerAccountID),
    ),
    (
        "_NSFileGroupOwnerAccountName",
        HostConstant::NSString(NSFileGroupOwnerAccountName),
    ),
    (
        "_NSFileGroupOwnerAccountID",
        HostConstant::NSString(NSFileGroupOwnerAccountID),
    ),
    (
        "_NSFileHFSCreatorCode",
        HostConstant::NSString(NSFileHFSCreatorCode),
    ),
    (
        "_NSFileSystemSize",
        HostConstant::NSString(NSFileSystemSize),
    ),
    (
        "_NSFileTypeSymbolicLink",
        HostConstant::NSString(NSFileTypeSymbolicLink),
    ),
    (
        "_NSFileModificationDate",
        HostConstant::NSString(NSFileModificationDate),
    ),
    (
        "_NSFileCreationDate",
        HostConstant::NSString(NSFileCreationDate),
    ),
    ("_NSFileSize", HostConstant::NSString(NSFileSize)),
    (
        "_NSFilePosixPermissions",
        HostConstant::NSString(NSFilePosixPermissions),
    ),
    (
        "_NSFileSystemFreeSize",
        HostConstant::NSString(NSFileSystemFreeSize),
    ),
    ("_NSFileType", HostConstant::NSString(NSFileType)),
    (
        "_NSFileTypeDirectory",
        HostConstant::NSString(NSFileTypeDirectory),
    ),
    (
        "_NSFileTypeRegular",
        HostConstant::NSString(NSFileTypeRegular),
    ),
];

fn NSSearchPathForDirectoriesInDomains(
    env: &mut Environment,
    directory: NSSearchPathDirectory,
    domain_mask: NSSearchPathDomainMask,
    expand_tilde: bool,
) -> id {
    // iPhone applications can only observe their sandbox's user domain. A
    // mask may name several domains (including NSAllDomainsMask); unavailable
    // domains simply contribute no entries.
    if domain_mask & NSUserDomainMask == 0 {
        let empty = ns_array::from_vec(env, Vec::new());
        return autorelease(env, empty);
    }

    let dir = match directory {
        NSApplicationDirectory | NSAllApplicationsDirectory => {
            // This might not actually be correct. I haven't bothered to
            // test it because I can't think of a good reason an iPhone OS app
            // would have to request this;
            // Wolfenstein 3D requests it but never uses it.
            // NSAllApplicationsDirectory is the union of all locations where
            // applications may be installed. iDared models one user-domain
            // applications root, so both directory values resolve to it.
            GuestPath::new(crate::fs::APPLICATIONS).to_owned()
        }
        NSDocumentDirectory => env.fs.home_directory().join("Documents"),
        NSLibraryDirectory => env.fs.home_directory().join("Library"),
        NSCachesDirectory => env.fs.home_directory().join("Library").join("Caches"),
        NSApplicationSupportDirectory => env
            .fs
            .home_directory()
            .join("Library")
            .join("Application Support"),
        _ => todo!("NSSearchPathDirectory {}", directory),
    };
    let absolute = String::from(dir);
    let path = if expand_tilde {
        absolute
    } else {
        let home_directory = env.fs.home_directory();
        let home = home_directory.as_str();
        absolute
            .strip_prefix(home)
            .map(|suffix| format!("~{suffix}"))
            .unwrap_or(absolute)
    };
    let dir = ns_string::from_rust_string(env, path);
    let dir_list = ns_array::from_vec(env, vec![dir]);
    autorelease(env, dir_list)
}

fn NSHomeDirectory(env: &mut Environment) -> id {
    let dir = env.fs.home_directory();
    let dir = ns_string::from_rust_string(env, String::from(dir.as_str()));
    autorelease(env, dir)
}

/// Check [crate::fs::Fs::new] for more info for
/// how temporary folder is setup on startup
fn NSTemporaryDirectory(env: &mut Environment) -> id {
    let dir = env.fs.home_directory().join("tmp");
    let dir = ns_string::from_rust_string(env, String::from(dir.as_str()));
    autorelease(env, dir)
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(NSHomeDirectory()),
    export_c_func!(NSTemporaryDirectory()),
    export_c_func!(NSSearchPathForDirectoriesInDomains(_, _, _)),
];

#[cfg(test)]
mod file_attribute_constant_tests {
    use super::*;

    #[test]
    fn historical_file_attribute_keys_are_exported() {
        let names: Vec<_> = CONSTANTS.iter().map(|(name, _)| *name).collect();
        for required in [
            "_NSFileOwnerAccountName",
            "_NSFileOwnerAccountID",
            "_NSFileGroupOwnerAccountName",
            "_NSFileGroupOwnerAccountID",
            "_NSFileHFSCreatorCode",
            "_NSFileSystemSize",
            "_NSFileTypeSymbolicLink",
        ] {
            assert!(names.contains(&required));
        }
    }
}

#[derive(Default)]
pub struct State {
    default_manager: Option<id>,
}

type NSDirectoryEnumerationOptions = NSUInteger;
const NSDirectoryEnumerationSkipsSubdirectoryDescendants: NSDirectoryEnumerationOptions = 1;
const NSDirectoryEnumerationSkipsPackageDescendants: NSDirectoryEnumerationOptions = 2;
const NSDirectoryEnumerationSkipsHiddenFiles: NSDirectoryEnumerationOptions = 4;

/// A directory being walked: its path relative to the enumerator's base, its
/// entries (name, is it a directory) and the index of the next one.
type WalkedDir = (String, Vec<(String, bool)>, usize);

/// Walks a directory lazily, in name order, so `skipDescendents` can work.
struct NSDirectoryEnumeratorHostObject {
    base: GuestPathBuf,
    stack: Vec<WalkedDir>,
    /// The directory last returned, whose contents come next unless skipped.
    pending_dir: Option<String>,
    /// The path last returned, relative to `base`.
    last: Option<String>,
    options: NSDirectoryEnumerationOptions,
    /// Return absolute `NSURL`s rather than relative paths.
    as_urls: bool,
}
impl HostObject for NSDirectoryEnumeratorHostObject {}

/// The entries of a directory, sorted by name as iPhone OS returns them.
fn list_dir(
    env: &Environment,
    base: &GuestPath,
    rel: &str,
    options: NSDirectoryEnumerationOptions,
) -> Vec<(String, bool)> {
    let dir = if rel.is_empty() {
        base.to_owned()
    } else {
        base.join(rel)
    };
    let Ok(entries) = env.fs.enumerate_with_types(&dir) else {
        return Vec::new();
    };
    let mut entries: Vec<_> = entries
        .filter(|(name, _)| {
            options & NSDirectoryEnumerationSkipsHiddenFiles == 0 || !name.starts_with('.')
        })
        .map(|(name, kind)| (name.to_string(), matches!(kind, FsNodeType::Directory)))
        .collect();
    entries.sort();
    entries
}

/// Directories that iPhone OS shows as a single file (bundles).
fn is_package(name: &str) -> bool {
    let ext = name.rsplit_once('.').map(|(_, ext)| ext);
    matches!(ext, Some("app" | "bundle" | "framework" | "plugin"))
}

fn new_directory_enumerator(
    env: &mut Environment,
    base: GuestPathBuf,
    options: NSDirectoryEnumerationOptions,
    as_urls: bool,
) -> id {
    let entries = list_dir(env, &base, "", options);
    let host_object = Box::new(NSDirectoryEnumeratorHostObject {
        base,
        stack: vec![(String::new(), entries, 0)],
        pending_dir: None,
        last: None,
        options,
        as_urls,
    });
    let class = env
        .objc
        .get_known_class("NSDirectoryEnumerator", &mut env.mem);
    let enumerator = env.objc.alloc_object(class, host_object, &mut env.mem);
    autorelease(env, enumerator)
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSFileManager: NSObject

+ (id)defaultManager {
    if let Some(existing) = env.framework_state.foundation.ns_file_manager.default_manager {
        existing
    } else {
        let new: id = msg![env; this new];
        env.framework_state.foundation.ns_file_manager.default_manager = Some(new);
        new
    }
}

- (id)URLsForDirectory:(NSSearchPathDirectory)directory
             inDomains:(NSSearchPathDomainMask)domain_mask {
    let paths = NSSearchPathForDirectoriesInDomains(env, directory, domain_mask, true);
    let count: NSUInteger = msg![env; paths count];
    let mut urls = Vec::with_capacity(count as usize);
    for index in 0..count {
        let path: id = msg![env; paths objectAtIndex:index];
        let url: id = msg_class![env; NSURL alloc];
        let url: id = msg![env; url initFileURLWithPath:path isDirectory:true];
        urls.push(url);
    }
    let urls = ns_array::from_vec(env, urls);
    autorelease(env, urls)
}

- (id)currentDirectoryPath {
    ns_string::from_rust_string(env, env.fs.working_directory().as_str().to_string())
}

- (bool)changeCurrentDirectoryPath:(id)path {
    let path = ns_string::to_rust_string(env, path); // TODO: avoid copy
    let path = GuestPath::new(&path);
    match env.fs.change_working_directory(path) {
        Ok(_) => true,
        Err(()) => false
    }
}

- (bool)fileExistsAtPath:(id)path { // NSString*
    let res_exists = if path == nil {
        false
    } else {
        let path = ns_string::to_rust_string(env, path); // TODO: avoid copy
        // fileExistsAtPath: will return true for directories
        // hence Fs::exists() rather than Fs::is_file() is appropriate.
        env.fs.exists(GuestPath::new(&path))
    };
    log_dbg!("[(NSFileManager*) {:?} fileExistsAtPath:{:?}] => {}", this, path, res_exists);
    res_exists
}

- (bool)fileExistsAtPath:(id)path // NSString*
             isDirectory:(MutPtr<bool>)is_dir {
    let (res_exists, res_is_dir) = if path == nil {
        (false, false)
    } else {
        // TODO: mutualize with fileExistsAtPath:
        let path = ns_string::to_rust_string(env, path); // TODO: avoid copy
        let guest_path = GuestPath::new(&path);
        (env.fs.exists(guest_path), !env.fs.is_file(guest_path))
    };

    if !is_dir.is_null() {
        env.mem.write(is_dir, res_is_dir);
    }

    log_dbg!("[(NSFileManager*) {:?} fileExistsAtPath:{:?} isDirectory:{:?}] => {}", this, path, res_is_dir, res_exists);
    res_exists
}

- (bool)createFileAtPath:(id)path // NSString*
                contents:(id)data // NSData*
              attributes:(id)attributes { // NSDictionary*
    assert!(attributes == nil); // TODO
    if data == nil {
        let empty: id = msg_class![env; NSData new];
        let res: bool = msg![env; empty writeToFile:path atomically:false];
        release(env, empty);
        res
    } else {
        msg![env; data writeToFile:path atomically:false]
    }
}

- (bool)removeItemAtPath:(id)path // NSString*
                   error:(MutPtr<id>)out_error { // NSError**
    // TODO: call delegate
    let path = ns_string::to_rust_string(env, path); // TODO: avoid copy
    match env.fs.remove(GuestPath::new(&path)) {
        Ok(()) => true,
        Err(err) => {
            if !out_error.is_null() {
                match err {
                    FsError::DoesNotExist => {
                        let domain = get_static_str(env, NSCocoaErrorDomain);
                        let error = msg_class![env; NSError alloc];
                        let error = msg![env; error initWithDomain:domain code:NSFileReadNoSuchFileError userInfo:nil];
                        autorelease(env, error);
                        env.mem.write(out_error, error);
                    }
                    _ => unimplemented!()
                }
            }
            false
        }
    }
}

- (bool)moveItemAtPath:(id)path // NSString*
                toPath:(id)toPath // NSString*
                 error:(MutPtr<id>)error { // NSError**
    // TODO: call delegate
    let path = ns_string::to_rust_string(env, path); // TODO: avoid copy
    let toPath = ns_string::to_rust_string(env, toPath); // TODO: avoid copy
    match env.fs.rename(GuestPath::new(&path), GuestPath::new(&toPath)) {
        Ok(()) => true,
        Err(_) => {
            if !error.is_null() {
               todo!(); // TODO: create an NSError if requested
            }
            false
        }
    }
}

- (bool)createDirectoryAtPath:(id)path // NSString *
                   attributes:(id)attributes { // NSDictionary*
    let error: MutPtr<id> = Ptr::null();
    msg![env; this createDirectoryAtPath:path
             withIntermediateDirectories:false
                              attributes:attributes
                                   error:error]
}

- (bool)createDirectoryAtPath:(id)path // NSString *
  withIntermediateDirectories:(bool)with_intermediates
                   attributes:(id)attributes // NSDictionary*
                        error:(MutPtr<id>)error { // NSError**
    if attributes != nil {
        // iDared's virtual filesystem does not currently persist Unix mode
        // bits, but callers are allowed to supply NSFilePosixPermissions when
        // creating a directory. Decode and validate the historical NSNumber
        // contract instead of rejecting the whole operation.
        let permissions_key = get_static_str(env, NSFilePosixPermissions);
        let permissions: id = msg![env; attributes objectForKey:permissions_key];
        if permissions != nil {
            let permissions: i16 = msg![env; permissions shortValue];
            log_dbg!(
                "createDirectoryAtPath requested POSIX permissions {:04o}; virtual filesystem permissions are fixed",
                (permissions as u16) & 0o7777,
            );
        }
    }

    let path_str = ns_string::to_rust_string(env, path); // TODO: avoid copy
    let res = if with_intermediates {
        env.fs.create_dir_all(GuestPath::new(&path_str))
    } else {
        env.fs.create_dir(GuestPath::new(&path_str))
    };
    match res {
        Ok(()) => {
            log_dbg!("createDirectoryAtPath {} => true", path_str);
            true
        }
        Err(err) => {
            assert!(error.is_null()); // TODO
            log!(
                "Warning: createDirectoryAtPath {} failed with {:?}, returning false",
                path_str,
                err,
            );
            false
        }
    }
}

- (id)enumeratorAtPath:(id)path { // NSString*
    let path = GuestPathBuf::from(ns_string::to_rust_string(env, path).into_owned());
    if !env.fs.is_dir(&path) {
        return nil;
    }
    new_directory_enumerator(env, path, 0, false)
}

- (id)subpathsOfDirectoryAtPath:(id)path // NSString*
                            error:(MutPtr<id>)error { // NSError**
    let enumerator: id = msg![env; this enumeratorAtPath:path];
    if enumerator == nil {
        if !error.is_null() {
            let domain = get_static_str(env, NSCocoaErrorDomain);
            let file_error: id = msg_class![env; NSError errorWithDomain:domain
                                                             code:NSFileReadNoSuchFileError
                                                         userInfo:nil];
            env.mem.write(error, file_error);
        }
        return nil;
    }
    let mut subpaths = Vec::new();
    loop {
        let subpath: id = msg![env; enumerator nextObject];
        if subpath == nil {
            break;
        }
        // NSDirectoryEnumerator returns an autoreleased object. `from_vec`
        // adopts already-owned elements, so retain the string while
        // transferring it into the returned NSArray.
        subpaths.push(retain(env, subpath));
    }
    let result = ns_array::from_vec(env, subpaths);
    autorelease(env, result)
}

- (id)enumeratorAtURL:(id)url // NSURL*
includingPropertiesForKeys:(id)_keys // NSArray*, only prefetching
                   options:(NSDirectoryEnumerationOptions)options
              errorHandler:(id)error_handler { // block
    let path: id = msg![env; url path];
    let path = GuestPathBuf::from(ns_string::to_rust_string(env, path).into_owned());
    // Like on iOS, a URL that isn't a directory gives an enumerator that
    // returns nothing.
    if !env.fs.is_dir(&path) && error_handler != nil {
        log!("TODO: enumeratorAtURL: errorHandler not called for {:?}", path);
    }
    new_directory_enumerator(env, path, options, true)
}

- (id)directoryContentsAtPath:(id)path /* NSString* */ { // NSArray*
    let path = ns_string::to_rust_string(env, path); // TODO: avoid copy
    let Ok(paths) = env.fs.enumerate(GuestPath::new(&path)) else {
        return nil;
    };
    let paths: Vec<GuestPathBuf> = paths
        .map(|path| GuestPathBuf::from(GuestPath::new(path)))
        .collect();
    log_dbg!("directoryContentsAtPath {}: {:?}", path, paths);
    let path_strings = paths
        .iter()
        .map(|name| ns_string::from_rust_string(env, name.as_str().to_string()))
        .collect();
    let res = ns_array::from_vec(env, path_strings);
    autorelease(env, res)
}

- (id)contentsOfDirectoryAtPath:(id)path /* NSString* */
                          error:(MutPtr<id>)error { // NSError**
    let contents: id = msg![env; this directoryContentsAtPath:path];
    if contents == nil && !error.is_null() {
        let domain = get_static_str(env, NSCocoaErrorDomain);
        let file_error: id = msg_class![env; NSError errorWithDomain:domain
                                                         code:NSFileReadNoSuchFileError
                                                     userInfo:nil];
        env.mem.write(error, file_error);
    }
    contents
}

- (bool)isReadableFileAtPath:(id)path { // NSString*
    let (_, readable, _, _) = {
        let path = ns_string::to_rust_string(env, path); // TODO: avoid copy
        env.fs.access(GuestPath::new(&path))
    };
    readable
}

- (bool)isWritableFileAtPath:(id)path { // NSString*
    let (_, _, writable, _) = {
        let path = ns_string::to_rust_string(env, path); // TODO: avoid copy
        env.fs.access(GuestPath::new(&path))
    };
    writable
}

- (bool)isDeletableFileAtPath:(id)path { // NSString*
    let is_file = {
        let path = ns_string::to_rust_string(env, path); // TODO: avoid copy
        env.fs.is_file(GuestPath::new(&path))
    };

    if is_file {
        return msg![env; this isWritableFileAtPath:path];
    }

    let directory_enumerator: id = msg![env; this enumeratorAtPath:path];

    let mut is_deletable = true;
    loop {
        let path: id = msg![env; directory_enumerator nextObject];
        if path == nil {
            break;
        }
        let is_path_deletable: bool = msg![env; this isDeletableFileAtPath:path];
        is_deletable &= is_path_deletable;
        if !is_deletable {
            break;
        }
    }
    is_deletable
}

- (id)contentsAtPath:(id)path { // NSString *
    // TODO: return nil if path is directory
    // TODO: handle non-absolute paths?
    assert!(msg![env; path isAbsolutePath]);
    msg_class![env; NSData dataWithContentsOfFile:path]
}

- (bool)copyItemAtPath:(id)src // NSString*
                toPath:(id)dst // NSString*
                 error:(MutPtr<id>)error { // NSError**
    if !error.is_null() {
        env.mem.write(error, nil);
    }
    if src == nil || dst == nil {
        let code = if src == nil {
            NSFileReadInvalidFileNameError
        } else {
            NSFileWriteInvalidFileNameError
        };
        if !error.is_null() {
            let domain = get_static_str(env, NSCocoaErrorDomain);
            let file_error: id = msg_class![env; NSError errorWithDomain:domain
                                                             code:code
                                                         userInfo:nil];
            env.mem.write(error, file_error);
        }
        env.diagnostic_campaign_event(
            "file-copy", "invalid-nil-path manager,source,destination,error-code",
            [this.to_bits(), src.to_bits(), dst.to_bits(), code as u32, 0, 0, 0, 0]
        );
        return false;
    }
    let src_object = src;
    let dst_object = dst;
    let src = ns_string::to_rust_string(env, src_object);
    let dst = ns_string::to_rust_string(env, dst_object);
    let data = match env.fs.read(GuestPath::new(src.as_ref())) {
        Ok(d) => d,
        Err(_) => {
            env.diagnostic_campaign_event(
                "file-copy", "source-read-failed manager,source,destination,error-code",
                [this.to_bits(), src_object.to_bits(), dst_object.to_bits(), NSFileReadNoSuchFileError as u32, 0, 0, 0, 0]
            );
            populate_copy_error(env, error, src_object, NSFileReadNoSuchFileError);
            return false;
        }
    };
    if env.fs.exists(GuestPath::new(dst.as_ref())) {
        env.diagnostic_campaign_event(
            "file-copy", "destination-exists manager,source,destination,error-code",
            [this.to_bits(), src_object.to_bits(), dst_object.to_bits(), NSFileWriteFileExistsError as u32, 0, 0, 0, 0]
        );
        populate_copy_error(env, error, dst_object, NSFileWriteFileExistsError);
        return false;
    }
    if env.fs.write(GuestPath::new(dst.as_ref()), &data).is_err() {
        env.diagnostic_campaign_event(
            "file-copy", "destination-write-failed manager,source,destination,error-code",
            [this.to_bits(), src_object.to_bits(), dst_object.to_bits(), NSFileWriteUnknownError as u32, 0, 0, 0, 0]
        );
        populate_copy_error(env, error, dst_object, NSFileWriteUnknownError);
        return false;
    }
    env.diagnostic_campaign_event(
        "file-copy", "success manager,source,destination,bytes",
        [this.to_bits(), src_object.to_bits(), dst_object.to_bits(), data.len() as u32, 0, 0, 0, 0]
    );
    true
}

- (ConstPtr<u8>)fileSystemRepresentationWithPath:(id)path { // NSString*
    let length: NSUInteger = msg![env; path length];
    assert!(length > 0);
    // TODO: throw an exception if conversion fails
    msg![env; path UTF8String]
}

- (id)destinationOfSymbolicLinkAtPath:(id)path // NSString*
                                error:(MutPtr<id>)error { // NSError**
    if !error.is_null() {
        env.mem.write(error, nil);
    }

    if path == nil {
        if !error.is_null() {
            let domain = get_static_str(env, NSCocoaErrorDomain);
            let file_error: id = msg_class![env; NSError errorWithDomain:domain
                                                             code:NSFileReadInvalidFileNameError
                                                         userInfo:nil];
            env.mem.write(error, file_error);
        }
        return nil;
    }

    // The guest filesystem currently resolves host links while constructing
    // its node tree and cannot contain a symbolic-link node. Preserve the
    // historical Cocoa result for a path that is not a symbolic link: nil and
    // an NSError, rather than treating the resolved file as its own target or
    // terminating on the selector. Once link nodes are represented, the
    // success case can return their stored destination without changing this
    // failure contract.
    let path_string = ns_string::to_rust_string(env, path);
    let code = if env.fs.exists(GuestPath::new(path_string.as_ref())) {
        NSFileReadUnknownError
    } else {
        NSFileReadNoSuchFileError
    };
    populate_copy_error(env, error, path, code);
    nil
}

- (id)fileAttributesAtPath:(id)path // NSString *
              traverseLink:(bool)traverse {
    // TODO: other attributes
    log_once!("Warning: NSFileManager fileAttributesAtPath:traverseLink: returns only NSFileType, NSFileCreationDate, NSFileModificationDate and NSFileSize attributes!");

    let path = ns_string::to_rust_string(env, path); // TODO: avoid copy
    // TODO: traverse link
    log_dbg!("[(NSFileManager *){:?} fileAttributesAtPath:{} traverse:{}]", this, path, traverse);
    let guest_path = GuestPath::new(&path);

    file_attributes_common(env, guest_path)
}

- (id)attributesOfItemAtPath:(id)path // NSString *
                       error:(MutPtr<id>)error { // NSError **
    if !error.is_null() {
        // NSError ** out parameters are initialized to nil on success paths.
        env.mem.write(error, nil);
    }

    // TODO: other attributes
    log_once!("Warning: NSFileManager attributesOfItemAtPath:error: returns only NSFileType, NSFileCreationDate, NSFileModificationDate and NSFileSize attributes!");

    let path_object = path;
    let path = ns_string::to_rust_string(env, path_object); // TODO: avoid copy
    // TODO: traverse link
    log_dbg!("[(NSFileManager *){:?} attributesOfItemAtPath:{} error:{:?}]", this, path, error);
    let guest_path = GuestPath::new(&path);

    let attributes = file_attributes_common(env, guest_path);
    if attributes == nil && !error.is_null() {
        let domain = get_static_str(env, NSCocoaErrorDomain);
        let path_key = get_static_str(env, super::ns_error::NSFilePathErrorKey);
        let user_info: id =
            msg_class![env; NSDictionary dictionaryWithObject:path_object forKey:path_key];
        let file_error: id = msg_class![env; NSError errorWithDomain:domain
                                                         code:NSFileReadNoSuchFileError
                                                     userInfo:user_info];
        env.mem.write(error, file_error);
    }
    attributes
}

- (bool)setAttributes:(id)attributes // NSDictionary *
       ofItemAtPath:(id)path // NSString *
                error:(MutPtr<id>)error { // NSError **
    if !error.is_null() {
        env.mem.write(error, nil);
    }
    if path == nil || attributes == nil {
        return false;
    }
    let path_string = ns_string::to_rust_string(env, path);
    if !env.fs.exists(GuestPath::new(&path_string)) {
        return false;
    }

    // The virtual filesystem currently has fixed Unix metadata.  Validate the
    // historical attributes that old games use and accept them for existing
    // nodes, just as createDirectoryAtPath:attributes: already does for mode
    // bits.  Unknown attributes remain visible as a failure.
    let keys: id = msg![env; attributes allKeys];
    let count: NSUInteger = msg![env; keys count];
    for index in 0..count {
        let key: id = msg![env; keys objectAtIndex:index];
        let known = [NSFilePosixPermissions, NSFileModificationDate, NSFileCreationDate]
            .iter()
            .any(|name| {
                let known_key = get_static_str(env, name);
                msg![env; key isEqual:known_key]
            });
        if !known {
            log!("Warning: setAttributes:ofItemAtPath:error: unsupported attribute {:?}", key);
            return false;
        }
        let value: id = msg![env; attributes objectForKey:key];
        if value == nil {
            return false;
        }
    }
    log_dbg!(
        "setAttributes:ofItemAtPath:error: validated {} fixed-metadata attributes for {}",
        count,
        path_string,
    );
    true
}

// Deprecated in iPhone OS 2.0.
- (id)fileSystemAttributesAtPath:(id)path { // NSString*
    msg![env; this attributesOfFileSystemForPath:path error:(MutPtr::<id>::null())]
}

- (id)attributesOfFileSystemForPath:(id)_path
                              error:(MutPtr<id>)error {
    // TODO: other attributes
    log_once!("Warning: NSFileManager attributesOfFileSystemForPath:error: returns only NSFileSystemFreeSize attribute!");

    assert!(error.is_null()); // TODO

    let dict = msg_class![env; NSMutableDictionary new];

    // Reporting 1 Gb of free space should be enough
    // TODO: unify with `statfs`
    // TODO: account for path
    let size: u64 = 1024 * 1024 * 1024;
    let size_num: id = msg_class![env; NSNumber numberWithUnsignedLongLong:size];

    let fs_free_size_key = get_static_str(env, NSFileSystemFreeSize);
    () = msg![env; dict setObject:size_num forKey:fs_free_size_key];

    let dict_imm = msg![env; dict copy];
    release(env, dict);
    autorelease(env, dict_imm)
}

@end

@implementation NSDirectoryEnumerator: NSEnumerator

- (id)nextObject {
    let host_obj = env.objc.borrow_mut::<NSDirectoryEnumeratorHostObject>(this);
    if let Some(dir) = host_obj.pending_dir.take() {
        let (base, options) = (host_obj.base.clone(), host_obj.options);
        let entries = list_dir(env, &base, &dir, options);
        let host_obj = env.objc.borrow_mut::<NSDirectoryEnumeratorHostObject>(this);
        host_obj.stack.push((dir, entries, 0));
    }
    let host_obj = env.objc.borrow_mut::<NSDirectoryEnumeratorHostObject>(this);
    let rel = loop {
        let Some((dir, entries, next)) = host_obj.stack.last_mut() else {
            host_obj.last = None;
            return nil;
        };
        let Some((name, is_dir)) = entries.get(*next) else {
            host_obj.stack.pop();
            continue;
        };
        *next += 1;
        let rel = if dir.is_empty() {
            name.clone()
        } else {
            format!("{dir}/{name}")
        };
        let options = host_obj.options;
        if *is_dir
            && options & NSDirectoryEnumerationSkipsSubdirectoryDescendants == 0
            && (options & NSDirectoryEnumerationSkipsPackageDescendants == 0 || !is_package(name))
        {
            host_obj.pending_dir = Some(rel.clone());
        }
        break rel;
    };
    host_obj.last = Some(rel.clone());
    if host_obj.as_urls {
        let path = host_obj.base.join(&rel);
        let path = ns_string::from_rust_string(env, String::from(path));
        let path = autorelease(env, path);
        msg_class![env; NSURL fileURLWithPath:path]
    } else {
        let rel = ns_string::from_rust_string(env, rel);
        autorelease(env, rel)
    }
}

// Not to be confused with skipDescendants (iOS 4), which does the same.
- (())skipDescendents {
    env.objc.borrow_mut::<NSDirectoryEnumeratorHostObject>(this).pending_dir = None;
}
- (())skipDescendants {
    env.objc.borrow_mut::<NSDirectoryEnumeratorHostObject>(this).pending_dir = None;
}

- (NSUInteger)level {
    let host_obj = env.objc.borrow::<NSDirectoryEnumeratorHostObject>(this);
    host_obj.last.as_ref().map_or(0, |rel| rel.split('/').count() as NSUInteger)
}

- (id)fileAttributes {
    let host_obj = env.objc.borrow::<NSDirectoryEnumeratorHostObject>(this);
    let Some(rel) = &host_obj.last else {
        return nil;
    };
    let path = host_obj.base.join(rel);
    file_attributes_common(env, &path)
}

- (id)directoryAttributes {
    let base = env.objc.borrow::<NSDirectoryEnumeratorHostObject>(this).base.clone();
    file_attributes_common(env, &base)
}

@end

};

fn populate_copy_error(env: &mut Environment, out_error: MutPtr<id>, path: id, code: NSInteger) {
    if out_error.is_null() {
        return;
    }
    let domain = get_static_str(env, NSCocoaErrorDomain);
    let path_key = get_static_str(env, NSFilePathErrorKey);
    let user_info: id = msg_class![env; NSDictionary dictionaryWithObject:path forKey:path_key];
    let error: id = msg_class![env; NSError errorWithDomain:domain code:code userInfo:user_info];
    env.mem.write(out_error, error);
}

/// Helper function for `fileAttributesAtPath:traverseLink:` and
/// `attributesOfItemAtPath:error:`
fn file_attributes_common(env: &mut Environment, guest_path: &GuestPath) -> id {
    if !env.fs.exists(guest_path) {
        log!(
            "file_attributes_common() called with file that does not exist: {:?}, Returning nil",
            guest_path
        );
        return nil;
    }

    // TODO: support more attributes
    let unix_timestamp: f64 = env.fs.modified(guest_path).unwrap() as f64;
    let unix_ref_date: id = msg_class![env; NSDate dateWithTimeIntervalSince1970:0f64];
    let unix_date: id =
        msg_class![env; NSDate dateWithTimeInterval:unix_timestamp sinceDate:unix_ref_date];

    let size = env.fs.size(guest_path).unwrap();
    let size_num: id = msg_class![env; NSNumber numberWithUnsignedLongLong:size];

    let dict = msg_class![env; NSMutableDictionary new];

    let modif_date_key = get_static_str(env, NSFileModificationDate);
    () = msg![env; dict setObject:unix_date forKey:modif_date_key];

    // The virtual filesystem currently exposes one timestamp. Use it for the
    // creation date too; legacy applications generally use these values for
    // cache invalidation and need a stable NSDate rather than exact metadata.
    let creation_date_key = get_static_str(env, NSFileCreationDate);
    () = msg![env; dict setObject:unix_date forKey:creation_date_key];

    let size_key = get_static_str(env, NSFileSize);
    () = msg![env; dict setObject:size_num forKey:size_key];

    let file_type_key = get_static_str(env, NSFileType);
    // TODO: other types
    if env.fs.is_file(guest_path) {
        let file_type_regular = get_static_str(env, NSFileTypeRegular);
        () = msg![env; dict setObject:file_type_regular forKey:file_type_key];
    } else if env.fs.is_dir(guest_path) {
        let file_type_directory = get_static_str(env, NSFileTypeDirectory);
        () = msg![env; dict setObject:file_type_directory forKey:file_type_key];
    }

    let dict_imm = msg![env; dict copy];
    release(env, dict);
    autorelease(env, dict_imm)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::guest_size_of;

    #[test]
    fn application_support_search_path_uses_the_user_library() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let array = NSSearchPathForDirectoriesInDomains(
            &mut env,
            NSApplicationSupportDirectory,
            NSUserDomainMask,
            true,
        );
        let path: id = msg![&mut env; array objectAtIndex:0u32];
        assert!(ns_string::to_rust_string(&mut env, path).ends_with("/Library/Application Support"));
    }

    #[test]
    fn domain_masks_return_only_available_user_paths() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let all_domains =
            NSSearchPathForDirectoriesInDomains(&mut env, NSDocumentDirectory, 0xffff, true);
        let count: NSUInteger = msg![&mut env; all_domains count];
        assert_eq!(count, 1);

        let system_only =
            NSSearchPathForDirectoriesInDomains(&mut env, NSDocumentDirectory, 1 << 3, true);
        let count: NSUInteger = msg![&mut env; system_only count];
        assert_eq!(count, 0);
    }

    #[test]
    fn development_255_subpaths_keep_enumerated_strings_alive_after_pool_drain() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let host_root = std::env::temp_dir().join(format!(
            "idared-subpaths-ownership-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&host_root);
        std::fs::create_dir_all(&host_root).unwrap();
        *env.fs = crate::fs::Fs::new_fake_fs_with_writable_host_root(&host_root);
        let env = &mut env;
        let directory = GuestPathBuf::from("/subpaths-ownership".to_owned());
        env.fs.create_dir_all(&directory).unwrap();
        env.fs.write(&directory.join("entry.txt"), b"fixture").unwrap();

        let outer_pool: id = msg_class![env; NSAutoreleasePool new];
        let manager: id = msg_class![env; NSFileManager defaultManager];
        let path = ns_string::from_rust_string(env, directory.as_str().to_owned());
        let paths: id = msg![env; manager subpathsOfDirectoryAtPath:path
                                          error:(MutPtr::<id>::null())];
        let paths: id = msg![env; paths retain];
        () = msg![env; outer_pool drain];

        let count: NSUInteger = msg![env; paths count];
        assert_eq!(count, 1);
        let entry: id = msg![env; paths objectAtIndex:0u32];
        assert_eq!(ns_string::to_rust_string(env, entry), "entry.txt");
        release(env, paths);
    }

    #[test]
    fn development_255_symbolic_link_destination_reports_bounded_errors_for_unrepresented_links() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let manager: id = msg_class![env; NSFileManager defaultManager];
        let error_out: MutPtr<id> = env.mem.alloc(guest_size_of::<id>()).cast();

        let missing = ns_string::from_rust_string(env, "/definitely/missing/link".to_owned());
        env.mem.write(error_out, id::from_bits(0x1234));
        let destination: id = msg![env; manager destinationOfSymbolicLinkAtPath:missing
                                                       error:error_out];
        assert_eq!(destination, nil);
        let error = env.mem.read(error_out);
        assert_ne!(error, nil);
        let code: NSInteger = msg![env; error code];
        assert_eq!(code, NSFileReadNoSuchFileError);

        let ordinary = ns_string::from_rust_string(env, "/".to_owned());
        env.mem.write(error_out, id::from_bits(0x1234));
        let destination: id = msg![env; manager destinationOfSymbolicLinkAtPath:ordinary
                                                       error:error_out];
        assert_eq!(destination, nil);
        let error = env.mem.read(error_out);
        assert_ne!(error, nil);
        let code: NSInteger = msg![env; error code];
        assert_eq!(code, NSFileReadUnknownError);

        env.mem.write(error_out, id::from_bits(0x1234));
        let destination: id = msg![env; manager destinationOfSymbolicLinkAtPath:nil
                                                       error:error_out];
        assert_eq!(destination, nil);
        let error = env.mem.read(error_out);
        assert_ne!(error, nil);
        let code: NSInteger = msg![env; error code];
        assert_eq!(code, NSFileReadInvalidFileNameError);

        // A caller that declines error details still gets the same nil result.
        let destination: id = msg![env; manager destinationOfSymbolicLinkAtPath:ordinary
                                                       error:(MutPtr::<id>::null())];
        assert_eq!(destination, nil);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
    }

    #[test]
    fn development_255_file_copy_rejects_nil_paths_with_errors() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let manager: id = msg_class![env; NSFileManager defaultManager];
        let ordinary = ns_string::from_rust_string(env, "/".to_owned());
        let error_out: MutPtr<id> = env.mem.alloc(guest_size_of::<id>()).cast();

        let copied: bool = msg![env; manager copyItemAtPath:nil
                                                  toPath:ordinary
                                                   error:error_out];
        assert!(!copied);
        let error = env.mem.read(error_out);
        let code: NSInteger = msg![env; error code];
        assert_eq!(code, NSFileReadInvalidFileNameError);

        let copied: bool = msg![env; manager copyItemAtPath:ordinary
                                                  toPath:nil
                                                   error:error_out];
        assert!(!copied);
        let error = env.mem.read(error_out);
        let code: NSInteger = msg![env; error code];
        assert_eq!(code, NSFileWriteInvalidFileNameError);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
    }
}
