/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! A small, guest-local subset of iOS Keychain Services.

use crate::dyld::{export_c_func, ConstantExports, FunctionExports, HostConstant};
use crate::frameworks::foundation::ns_string;
use crate::frameworks::foundation::NSUInteger;
use crate::fs::{GuestPath, GuestPathBuf};
use crate::mem::{ConstVoidPtr, MutPtr};
use crate::objc::{id, msg, msg_class, nil, retain, Class};
use crate::Environment;
use serde::{Deserialize, Serialize};

type OSStatus = i32;

const ERR_SEC_SUCCESS: OSStatus = 0;
const ERR_SEC_PARAM: OSStatus = -50;
const ERR_SEC_DUPLICATE_ITEM: OSStatus = -25299;
const ERR_SEC_ITEM_NOT_FOUND: OSStatus = -25300;
const ERR_SEC_UNIMPLEMENTED: OSStatus = -4;

const STORE_SCHEMA: u32 = 1;
const STORE_PREFIX: &str = ".touchHLE-keychain-k1-";
const STORE_SUFFIX: &str = ".binary.plist";

const K_SEC_ATTR_ACCOUNT: &str = "acct";
const K_SEC_ATTR_AUTHENTICATION_TYPE: &str = "atyp";
const K_SEC_ATTR_AUTHENTICATION_TYPE_DEFAULT: &str = "dflt";
const K_SEC_ATTR_SECURITY_DOMAIN: &str = "sdmn";
const K_SEC_ATTR_SERVER: &str = "srvr";
const K_SEC_ATTR_TYPE: &str = "type";
const K_SEC_CLASS: &str = "class";
const K_SEC_CLASS_INTERNET_PASSWORD: &str = "inet";
const K_SEC_MATCH_LIMIT: &str = "m_Limit";
const K_SEC_MATCH_LIMIT_ONE: &str = "m_LimitOne";
const K_SEC_RETURN_DATA: &str = "r_Data";
const K_SEC_VALUE_DATA: &str = "v_Data";

pub const CONSTANTS: ConstantExports = &[
    ("_kSecAttrAccount", HostConstant::NSString(K_SEC_ATTR_ACCOUNT)),
    (
        "_kSecAttrAuthenticationType",
        HostConstant::NSString(K_SEC_ATTR_AUTHENTICATION_TYPE),
    ),
    (
        "_kSecAttrAuthenticationTypeDefault",
        HostConstant::NSString(K_SEC_ATTR_AUTHENTICATION_TYPE_DEFAULT),
    ),
    (
        "_kSecAttrSecurityDomain",
        HostConstant::NSString(K_SEC_ATTR_SECURITY_DOMAIN),
    ),
    ("_kSecAttrServer", HostConstant::NSString(K_SEC_ATTR_SERVER)),
    ("_kSecAttrType", HostConstant::NSString(K_SEC_ATTR_TYPE)),
    ("_kSecClass", HostConstant::NSString(K_SEC_CLASS)),
    (
        "_kSecClassInternetPassword",
        HostConstant::NSString(K_SEC_CLASS_INTERNET_PASSWORD),
    ),
    ("_kSecMatchLimit", HostConstant::NSString(K_SEC_MATCH_LIMIT)),
    (
        "_kSecMatchLimitOne",
        HostConstant::NSString(K_SEC_MATCH_LIMIT_ONE),
    ),
    ("_kSecReturnData", HostConstant::NSString(K_SEC_RETURN_DATA)),
    ("_kSecValueData", HostConstant::NSString(K_SEC_VALUE_DATA)),
];

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(SecItemAdd(_, _)),
    export_c_func!(SecItemCopyMatching(_, _)),
    export_c_func!(SecItemDelete(_)),
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct InternetPasswordRecord {
    account: Option<String>,
    authentication_type: Option<String>,
    security_domain: Option<String>,
    server: Option<String>,
    item_type: Option<String>,
    value_data: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
struct StoreFile {
    schema: u32,
    generation: u64,
    records: Vec<InternetPasswordRecord>,
}

#[derive(Default)]
struct Query {
    account: Option<String>,
    authentication_type: Option<String>,
    security_domain: Option<String>,
    server: Option<String>,
    item_type: Option<String>,
    value_data: Option<Vec<u8>>,
    return_data: bool,
}

fn k(env: &mut Environment, value: &'static str) -> id {
    ns_string::get_static_str(env, value)
}

fn get(env: &mut Environment, dictionary: id, key: &'static str) -> id {
    let key = k(env, key);
    msg![env; dictionary objectForKey:key]
}

fn is_instance_of(env: &mut Environment, object: id, class_name: &'static str) -> bool {
    if object == nil {
        return false;
    }
    let class: Class = match class_name {
        "NSDictionary" => msg_class![env; NSDictionary class],
        "NSString" => msg_class![env; NSString class],
        "NSNumber" => msg_class![env; NSNumber class],
        "NSData" => msg_class![env; NSData class],
        _ => unreachable!(),
    };
    msg![env; object isKindOfClass:class]
}

fn optional_string(
    env: &mut Environment,
    dictionary: id,
    key: &'static str,
) -> Result<Option<String>, OSStatus> {
    let object = get(env, dictionary, key);
    if object == nil {
        return Ok(None);
    }
    if !is_instance_of(env, object, "NSString") {
        return Err(ERR_SEC_PARAM);
    }
    Ok(Some(ns_string::to_rust_string(env, object).into_owned()))
}

fn optional_data(
    env: &mut Environment,
    dictionary: id,
    key: &'static str,
) -> Result<Option<Vec<u8>>, OSStatus> {
    let object = get(env, dictionary, key);
    if object == nil {
        return Ok(None);
    }
    if !is_instance_of(env, object, "NSData") {
        return Err(ERR_SEC_PARAM);
    }
    let length: NSUInteger = msg![env; object length];
    if length == 0 {
        return Ok(Some(Vec::new()));
    }
    let bytes: ConstVoidPtr = msg![env; object bytes];
    if bytes.is_null() {
        return Err(ERR_SEC_PARAM);
    }
    Ok(Some(env.mem.bytes_at(bytes.cast(), length).to_vec()))
}

fn parse_query(env: &mut Environment, dictionary: id) -> Result<Query, OSStatus> {
    if !is_instance_of(env, dictionary, "NSDictionary") {
        return Err(ERR_SEC_PARAM);
    }

    let class = get(env, dictionary, K_SEC_CLASS);
    if class == nil || !is_instance_of(env, class, "NSString") {
        return Err(ERR_SEC_PARAM);
    }
    let internet_password = k(env, K_SEC_CLASS_INTERNET_PASSWORD);
    let class_matches: bool = msg![env; class isEqual:internet_password];
    if !class_matches {
        return Err(ERR_SEC_UNIMPLEMENTED);
    }

    let match_limit = get(env, dictionary, K_SEC_MATCH_LIMIT);
    if match_limit != nil {
        if !is_instance_of(env, match_limit, "NSString") {
            return Err(ERR_SEC_PARAM);
        }
        let one = k(env, K_SEC_MATCH_LIMIT_ONE);
        let is_one: bool = msg![env; match_limit isEqual:one];
        if !is_one {
            return Err(ERR_SEC_UNIMPLEMENTED);
        }
    }

    let return_data_object = get(env, dictionary, K_SEC_RETURN_DATA);
    let return_data = if return_data_object == nil {
        false
    } else {
        if !is_instance_of(env, return_data_object, "NSNumber") {
            return Err(ERR_SEC_PARAM);
        }
        msg![env; return_data_object boolValue]
    };

    Ok(Query {
        account: optional_string(env, dictionary, K_SEC_ATTR_ACCOUNT)?,
        authentication_type: optional_string(env, dictionary, K_SEC_ATTR_AUTHENTICATION_TYPE)?,
        security_domain: optional_string(env, dictionary, K_SEC_ATTR_SECURITY_DOMAIN)?,
        server: optional_string(env, dictionary, K_SEC_ATTR_SERVER)?,
        item_type: optional_string(env, dictionary, K_SEC_ATTR_TYPE)?,
        value_data: optional_data(env, dictionary, K_SEC_VALUE_DATA)?,
        return_data,
    })
}

fn record_matches(record: &InternetPasswordRecord, query: &Query) -> bool {
    query.account.as_ref().is_none_or(|value| record.account.as_ref() == Some(value))
        && query
            .authentication_type
            .as_ref()
            .is_none_or(|value| record.authentication_type.as_ref() == Some(value))
        && query
            .security_domain
            .as_ref()
            .is_none_or(|value| record.security_domain.as_ref() == Some(value))
        && query.server.as_ref().is_none_or(|value| record.server.as_ref() == Some(value))
        && query
            .item_type
            .as_ref()
            .is_none_or(|value| record.item_type.as_ref() == Some(value))
}

fn record_identity_matches(a: &InternetPasswordRecord, b: &InternetPasswordRecord) -> bool {
    a.account == b.account
        && a.authentication_type == b.authentication_type
        && a.security_domain == b.security_domain
        && a.server == b.server
        && a.item_type == b.item_type
}

fn store_directory(env: &Environment) -> GuestPathBuf {
    env.fs.home_directory().join("Library").join("Preferences")
}

fn generation_from_name(name: &str) -> Option<u64> {
    let number = name.strip_prefix(STORE_PREFIX)?.strip_suffix(STORE_SUFFIX)?;
    u64::from_str_radix(number, 16).ok()
}

fn store_path(directory: &GuestPath, generation: u64) -> GuestPathBuf {
    directory.join(format!(
        "{STORE_PREFIX}{generation:016x}{STORE_SUFFIX}"
    ))
}

fn load_store(env: &Environment) -> StoreFile {
    let directory = store_directory(env);
    let newest = env
        .fs
        .enumerate(&directory)
        .ok()
        .and_then(|names| names.filter_map(generation_from_name).max());
    let Some(generation) = newest else {
        return StoreFile {
            schema: STORE_SCHEMA,
            generation: 0,
            records: Vec::new(),
        };
    };
    let path = store_path(&directory, generation);
    let Ok(bytes) = env.fs.read(&path) else {
        return StoreFile {
            schema: STORE_SCHEMA,
            generation: 0,
            records: Vec::new(),
        };
    };
    match plist::from_bytes::<StoreFile>(&bytes) {
        Ok(store) if store.schema == STORE_SCHEMA && store.generation == generation => store,
        _ => StoreFile {
            schema: STORE_SCHEMA,
            generation: 0,
            records: Vec::new(),
        },
    }
}

fn save_store(env: &mut Environment, store: &mut StoreFile) -> Result<(), OSStatus> {
    let directory = store_directory(env);
    let old_generation = store.generation;
    store.generation = old_generation.checked_add(1).ok_or(ERR_SEC_PARAM)?;
    let mut bytes = Vec::new();
    if plist::to_writer_binary(&mut bytes, store).is_err() {
        store.generation = old_generation;
        return Err(ERR_SEC_PARAM);
    }
    let final_path = store_path(&directory, store.generation);
    let temporary_path = directory.join(format!(
        "{STORE_PREFIX}{:016x}.tmp",
        store.generation
    ));
    if env.fs.write(&temporary_path, &bytes).is_err()
        || env.fs.rename(&temporary_path, &final_path).is_err()
    {
        let _ = env.fs.remove(&temporary_path);
        store.generation = old_generation;
        return Err(ERR_SEC_PARAM);
    }
    if old_generation != 0 {
        let _ = env.fs.remove(store_path(&directory, old_generation));
    }
    Ok(())
}

fn k1_enabled(env: &Environment) -> bool {
    let enabled = env
        .options
        .openfeint_compatibility_profile
        .guest_local_init_for_bundle(env.bundle.bundle_identifier());
    #[cfg(test)]
    {
        enabled
            || (env.options.openfeint_compatibility_profile
                == crate::options::OpenFeintCompatibilityProfile::GuestLocalInitV1
                && env.bundle.bundle_identifier() == "com.touchhle.test")
    }
    #[cfg(not(test))]
    {
        enabled
    }
}

fn SecItemAdd(env: &mut Environment, attributes: id, result: MutPtr<id>) -> OSStatus {
    if !k1_enabled(env) {
        return ERR_SEC_UNIMPLEMENTED;
    }
    if !result.is_null() {
        env.mem.write(result, nil);
        return ERR_SEC_PARAM;
    }
    let query = match parse_query(env, attributes) {
        Ok(query) => query,
        Err(status) => return status,
    };
    let Some(value_data) = query.value_data else {
        return ERR_SEC_PARAM;
    };
    let record = InternetPasswordRecord {
        account: query.account,
        authentication_type: query.authentication_type,
        security_domain: query.security_domain,
        server: query.server,
        item_type: query.item_type,
        value_data,
    };
    let mut store = load_store(env);
    if store
        .records
        .iter()
        .any(|existing| record_identity_matches(existing, &record))
    {
        return ERR_SEC_DUPLICATE_ITEM;
    }
    store.records.push(record);
    match save_store(env, &mut store) {
        Ok(()) => ERR_SEC_SUCCESS,
        Err(status) => status,
    }
}

fn SecItemCopyMatching(
    env: &mut Environment,
    query_dictionary: id,
    result: MutPtr<id>,
) -> OSStatus {
    if !k1_enabled(env) {
        return ERR_SEC_UNIMPLEMENTED;
    }
    if !result.is_null() {
        env.mem.write(result, nil);
    }
    let query = match parse_query(env, query_dictionary) {
        Ok(query) => query,
        Err(status) => return status,
    };
    if !query.return_data || result.is_null() || query.value_data.is_some() {
        return ERR_SEC_PARAM;
    }
    let store = load_store(env);
    let Some(record) = store.records.iter().find(|record| record_matches(record, &query)) else {
        return ERR_SEC_ITEM_NOT_FOUND;
    };
    let length: NSUInteger = record.value_data.len().try_into().unwrap();
    let bytes = if length == 0 {
        ConstVoidPtr::null()
    } else {
        let guest_bytes = env.mem.alloc(length);
        env.mem
            .bytes_at_mut(guest_bytes.cast(), length)
            .copy_from_slice(&record.value_data);
        guest_bytes.cast_const()
    };
    let data: id = msg_class![env; NSData dataWithBytes:bytes length:length];
    if length != 0 {
        env.mem.free(bytes.cast_mut());
    }
    retain(env, data);
    env.mem.write(result, data);
    ERR_SEC_SUCCESS
}

fn SecItemDelete(env: &mut Environment, query_dictionary: id) -> OSStatus {
    if !k1_enabled(env) {
        return ERR_SEC_UNIMPLEMENTED;
    }
    let query = match parse_query(env, query_dictionary) {
        Ok(query) => query,
        Err(status) => return status,
    };
    if query.return_data || query.value_data.is_some() {
        return ERR_SEC_PARAM;
    }
    let mut store = load_store(env);
    let original_len = store.records.len();
    store.records.retain(|record| !record_matches(record, &query));
    if store.records.len() == original_len {
        return ERR_SEC_ITEM_NOT_FOUND;
    }
    match save_store(env, &mut store) {
        Ok(()) => ERR_SEC_SUCCESS,
        Err(status) => status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::Fs;
    use crate::objc::{msg, msg_class, release};
    use crate::options::OpenFeintCompatibilityProfile;
    use std::path::Path;

    fn test_root(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "idared-keychain-k1-{label}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ))
    }

    fn env_for(root: &Path) -> Environment {
        std::fs::create_dir_all(root).unwrap();
        let mut env = Environment::new_fake_for_test_with_thread();
        *env.fs = Fs::new_fake_fs_with_writable_host_root(root);
        env.bundle.set_fake_identity_for_test();
        env.options.openfeint_compatibility_profile =
            OpenFeintCompatibilityProfile::GuestLocalInitV1;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        env
    }

    fn query(env: &mut Environment, account: &str, server: &str, data: Option<&[u8]>) -> id {
        let dictionary: id = msg_class![env; NSMutableDictionary new];
        let class_key = k(env, K_SEC_CLASS);
        let class_value = k(env, K_SEC_CLASS_INTERNET_PASSWORD);
        let account_key = k(env, K_SEC_ATTR_ACCOUNT);
        let account_value = ns_string::from_rust_string(env, account.to_owned());
        let server_key = k(env, K_SEC_ATTR_SERVER);
        let server_value = ns_string::from_rust_string(env, server.to_owned());
        () = msg![env; dictionary setObject:class_value forKey:class_key];
        () = msg![env; dictionary setObject:account_value forKey:account_key];
        () = msg![env; dictionary setObject:server_value forKey:server_key];
        if let Some(data) = data {
            let length: NSUInteger = data.len().try_into().unwrap();
            let bytes = env.mem.alloc(length);
            env.mem
                .bytes_at_mut(bytes.cast(), length)
                .copy_from_slice(data);
            let const_bytes = bytes.cast_const();
            let value: id = msg_class![env; NSData dataWithBytes:const_bytes length:length];
            env.mem.free(bytes);
            let data_key = k(env, K_SEC_VALUE_DATA);
            () = msg![env; dictionary setObject:value forKey:data_key];
        }
        dictionary
    }

    fn request_data(env: &mut Environment, dictionary: id) {
        let return_key = k(env, K_SEC_RETURN_DATA);
        let yes: id = msg_class![env; NSNumber numberWithBool:true];
        let limit_key = k(env, K_SEC_MATCH_LIMIT);
        let limit_one = k(env, K_SEC_MATCH_LIMIT_ONE);
        () = msg![env; dictionary setObject:yes forKey:return_key];
        () = msg![env; dictionary setObject:limit_one forKey:limit_key];
    }

    fn empty_dictionary(env: &mut Environment) -> id {
        msg_class![env; NSMutableDictionary new]
    }

    fn set_object(env: &mut Environment, dictionary: id, object: id, key: id) {
        () = msg![env; dictionary setObject:object forKey:key];
    }

    fn copied_bytes(env: &mut Environment, data: id) -> Vec<u8> {
        let length: NSUInteger = msg![env; data length];
        if length == 0 {
            return Vec::new();
        }
        let bytes: ConstVoidPtr = msg![env; data bytes];
        env.mem.bytes_at(bytes.cast(), length).to_vec()
    }

    #[test]
    fn constants_and_exports_cover_exact_hookworld_surface() {
        assert_eq!(CONSTANTS.len(), 12);
        let names: Vec<_> = CONSTANTS.iter().map(|(name, _)| *name).collect();
        for required in [
            "_kSecAttrAccount",
            "_kSecAttrAuthenticationType",
            "_kSecAttrAuthenticationTypeDefault",
            "_kSecAttrSecurityDomain",
            "_kSecAttrServer",
            "_kSecAttrType",
            "_kSecClass",
            "_kSecClassInternetPassword",
            "_kSecMatchLimit",
            "_kSecMatchLimitOne",
            "_kSecReturnData",
            "_kSecValueData",
        ] {
            assert!(names.contains(&required));
        }
        assert_eq!(FUNCTIONS.len(), 3);
    }

    #[test]
    fn add_find_duplicate_delete_and_status_contract() {
        let root = test_root("crud");
        let mut env = env_for(&root);
        let add = query(&mut env, "player", "api.openfeint.com", Some(b"token-1"));
        assert_eq!(SecItemAdd(&mut env, add, MutPtr::null()), ERR_SEC_SUCCESS);
        assert_eq!(
            SecItemAdd(&mut env, add, MutPtr::null()),
            ERR_SEC_DUPLICATE_ITEM
        );

        let find = query(&mut env, "player", "api.openfeint.com", None);
        request_data(&mut env, find);
        let result = env.mem.alloc_and_write(nil);
        assert_eq!(
            SecItemCopyMatching(&mut env, find, result),
            ERR_SEC_SUCCESS
        );
        let returned = env.mem.read(result);
        assert_ne!(returned, nil);
        assert_eq!(copied_bytes(&mut env, returned), b"token-1");
        release(&mut env, returned);
        env.mem.free(result.cast());

        let delete = query(&mut env, "player", "api.openfeint.com", None);
        assert_eq!(SecItemDelete(&mut env, delete), ERR_SEC_SUCCESS);
        assert_eq!(SecItemDelete(&mut env, delete), ERR_SEC_ITEM_NOT_FOUND);
        let result = env.mem.alloc_and_write(nil);
        assert_eq!(
            SecItemCopyMatching(&mut env, find, result),
            ERR_SEC_ITEM_NOT_FOUND
        );
        assert_eq!(env.mem.read(result), nil);
        env.mem.free(result.cast());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn records_persist_across_environments_and_are_container_isolated() {
        let root_a = test_root("persist-a");
        let root_b = test_root("persist-b");
        {
            let mut env = env_for(&root_a);
            let add = query(&mut env, "player", "api.openfeint.com", Some(b"saved"));
            assert_eq!(SecItemAdd(&mut env, add, MutPtr::null()), ERR_SEC_SUCCESS);
        }
        {
            let mut env = env_for(&root_a);
            let find = query(&mut env, "player", "api.openfeint.com", None);
            request_data(&mut env, find);
            let result = env.mem.alloc_and_write(nil);
            assert_eq!(
                SecItemCopyMatching(&mut env, find, result),
                ERR_SEC_SUCCESS
            );
            let returned = env.mem.read(result);
            assert_eq!(copied_bytes(&mut env, returned), b"saved");
            release(&mut env, returned);
            env.mem.free(result.cast());
        }
        {
            let mut env = env_for(&root_b);
            let find = query(&mut env, "player", "api.openfeint.com", None);
            request_data(&mut env, find);
            let result = env.mem.alloc_and_write(nil);
            assert_eq!(
                SecItemCopyMatching(&mut env, find, result),
                ERR_SEC_ITEM_NOT_FOUND
            );
            assert_eq!(env.mem.read(result), nil);
            env.mem.free(result.cast());
        }
        std::fs::remove_dir_all(root_a).unwrap();
        std::fs::remove_dir_all(root_b).unwrap();
    }

    #[test]
    fn malformed_and_out_of_scope_queries_fail_without_writing() {
        let root = test_root("invalid");
        let mut env = env_for(&root);
        assert_eq!(SecItemAdd(&mut env, nil, MutPtr::null()), ERR_SEC_PARAM);
        let dictionary = empty_dictionary(&mut env);
        let class_key = k(&mut env, K_SEC_CLASS);
        let unsupported = ns_string::from_rust_string(&mut env, "genp".to_owned());
        set_object(&mut env, dictionary, unsupported, class_key);
        assert_eq!(
            SecItemAdd(&mut env, dictionary, MutPtr::null()),
            ERR_SEC_UNIMPLEMENTED
        );
        let directory = store_directory(&env);
        if let Ok(mut entries) = env.fs.enumerate(directory) {
            assert!(entries.all(|name| generation_from_name(name).is_none()));
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn baseline_profile_does_not_activate_k1() {
        let root = test_root("reversal");
        let mut env = env_for(&root);
        env.options.openfeint_compatibility_profile = OpenFeintCompatibilityProfile::Baseline;
        let add = query(&mut env, "player", "api.openfeint.com", Some(b"secret"));
        assert_eq!(
            SecItemAdd(&mut env, add, MutPtr::null()),
            ERR_SEC_UNIMPLEMENTED
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
