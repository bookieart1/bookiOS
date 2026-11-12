/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! NSXMLParser.
//!
//! "Real" iOS implementation probably uses libxml2 under the hood
//! (at least because error codes are identical to libxml,
//! at most because it does make sense).
//!
//! Our implementation is based instead on [quick-xml crate](https://docs.rs/quick-xml/latest/quick_xml) for convenience.
//! This is something to reconsider once we integrate
//! libxml dylib into the project.

use super::ns_string::{from_rust_string, get_static_str, to_rust_string};
use super::NSUInteger;
use crate::environment::Environment;
use crate::mem::ConstVoidPtr;
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, release, retain, ClassExports, HostObject,
    NSZonePtr, SEL,
};
use quick_xml::events::{BytesCData, BytesStart, Event};
use quick_xml::reader::Reader;

struct NSXMLParserHostObject {
    /// An internal representation of XML data to parse using NSData*
    data: id,
    /// An object conforming to NSXMLParserDelegateEventAdditions category
    /// or NSXMLParserDelegate protocol (which is equivalent)
    delegate: id,
    should_process_namespaces: bool,
    should_report_namespace_prefixes: bool,
    should_resolve_external_entities: bool,
    aborted: bool,
    /// NSError retained after the most recent failed parse. Native
    /// NSXMLParser keeps this available through `parserError`.
    parser_error: id,
}
impl HostObject for NSXMLParserHostObject {}

fn cdata_payload(_event: BytesCData<'_>) -> Vec<u8> {
    Vec::new()
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSXMLParser: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSXMLParserHostObject {
        data: nil,
        delegate: nil,
        should_process_namespaces: false,
        should_report_namespace_prefixes: false,
        should_resolve_external_entities: false,
        aborted: false,
        parser_error: nil,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

// TODO: more init methods
// TODO: more delegate messages to support

- (id)initWithContentsOfURL:(id)url { // NSURL *
    let data: id = msg_class![env; NSData dataWithContentsOfURL:url];
    msg![env; this initWithData:data]
}

- (id)initWithData:(id)data { // NSData *
    if data == nil {
        release(env, this);
        return nil;
    }
    retain(env, data);
    env.objc.borrow_mut::<NSXMLParserHostObject>(this).data = data;
    this
}

// weak/non-retaining
- (())setDelegate:(id)delegate {
    env.objc.borrow_mut::<NSXMLParserHostObject>(this).delegate = delegate;
}
- (id)delegate {
    env.objc.borrow::<NSXMLParserHostObject>(this).delegate
}

- (())setShouldResolveExternalEntities:(bool)should {
    env.objc.borrow_mut::<NSXMLParserHostObject>(this).should_resolve_external_entities = should;
}
- (())setShouldProcessNamespaces:(bool)should {
    env.objc.borrow_mut::<NSXMLParserHostObject>(this).should_process_namespaces = should;
}
- (())setShouldReportNamespacePrefixes:(bool)should {
    env.objc.borrow_mut::<NSXMLParserHostObject>(this).should_report_namespace_prefixes = should;
}
- (bool)shouldResolveExternalEntities {
    env.objc.borrow::<NSXMLParserHostObject>(this).should_resolve_external_entities
}
- (bool)shouldProcessNamespaces {
    env.objc.borrow::<NSXMLParserHostObject>(this).should_process_namespaces
}
- (bool)shouldReportNamespacePrefixes {
    env.objc.borrow::<NSXMLParserHostObject>(this).should_report_namespace_prefixes
}

- (())abortParsing {
    env.objc.borrow_mut::<NSXMLParserHostObject>(this).aborted = true;
}

- (bool)parse {
    let previous_error = {
        let host = env.objc.borrow_mut::<NSXMLParserHostObject>(this);
        host.aborted = false;
        std::mem::replace(&mut host.parser_error, nil)
    };
    release(env, previous_error);
    let data = env.objc.borrow::<NSXMLParserHostObject>(this).data;
    assert_ne!(data, nil);
    let bytes: ConstVoidPtr = msg![env; data bytes];
    let length: NSUInteger = msg![env; data length];
    log_dbg!("Parsing {:?}", env.mem.cstr_at_utf8(bytes.cast()));
    let bytes: &[u8] = env.mem.bytes_at_mut(bytes.cast().cast_mut(), length);

    // TODO: support partial parsing of XML
    let mut reader = Reader::from_reader(bytes);
    // TODO: parse and send delegate messages in one pass
    let mut events = Vec::new();
    let mut parse_failure = None;
    let mut open_elements = 0u32;
    loop {
        match reader.read_event() {
            Ok(Event::Eof) => {
                if open_elements != 0 {
                    parse_failure = Some((
                        reader.error_position(),
                        format!("unexpected end of document with {open_elements} open element(s)"),
                    ));
                }
                break;
            },
            Ok(e) => {
                match &e {
                    Event::Start(_) => open_elements += 1,
                    Event::End(_) => open_elements = open_elements.saturating_sub(1),
                    _ => {},
                }
                events.push(e.into_owned()); // TODO: avoid copying
            },
            Err(e) => {
                parse_failure = Some((reader.error_position(), e.to_string()));
                break;
            },
        }
    }

    let (delegate, should_process_namespaces) = {
        let host = env.objc.borrow::<NSXMLParserHostObject>(this);
        (host.delegate, host.should_process_namespaces)
    };
    let sel: SEL = env
        .objc
        .register_host_selector("parserDidStartDocument:".to_string(), &mut env.mem);
    let responds: bool = msg![env; delegate respondsToSelector:sel];
    if responds {
        () = msg![env; delegate parserDidStartDocument:this];
    }
    if env.objc.borrow::<NSXMLParserHostObject>(this).aborted {
        return false;
    }
    for event in events {
        match event {
            Event::Empty(e) => {
                let name_bytes = if should_process_namespaces {
                    e.local_name().as_ref().to_vec()
                } else {
                    e.name().as_ref().to_vec()
                };
                let name = String::from_utf8(name_bytes).unwrap();
                let name: id = from_rust_string(env, name);
                let name = autorelease(env, name);
                let sel: SEL = env
                    .objc
                    .register_host_selector(
                        "parser:didStartElement:namespaceURI:qualifiedName:attributes:".to_string(),
                        &mut env.mem
                    );
                let responds: bool = msg![env; delegate respondsToSelector:sel];
                if responds {
                    let dict = build_attributes_dict(env, e);
                    () = msg![env; delegate parser:this
                                   didStartElement:name
                                      namespaceURI:nil
                                     qualifiedName:nil
                                        attributes:dict];
                }
                if env.objc.borrow::<NSXMLParserHostObject>(this).aborted {
                    return false;
                }
                let sel: SEL = env
                    .objc
                    .register_host_selector(
                        "parser:didEndElement:namespaceURI:qualifiedName:".to_string(),
                        &mut env.mem
                    );
                let responds: bool = msg![env; delegate respondsToSelector:sel];
                if responds {
                    () = msg![env; delegate parser:this
                                     didEndElement:name
                                      namespaceURI:nil
                                     qualifiedName:nil];
                }
            }
            Event::Text(e) => {
                let text = e.decode().unwrap().to_string();
                // FIXME: skipping the end of the parsed string?
                if text != "\0" {
                    let sel: SEL = env
                        .objc
                        .register_host_selector("parser:foundCharacters:".to_string(), &mut env.mem);
                    let responds: bool = msg![env; delegate respondsToSelector:sel];
                    if responds {
                        let chars = from_rust_string(env, text);
                        let chars = autorelease(env, chars);
                        () = msg![env; delegate parser:this foundCharacters:chars];
                    }
                }
            }
            Event::Start(e) => {
                let name_bytes = if should_process_namespaces {
                    e.local_name().as_ref().to_vec()
                } else {
                    e.name().as_ref().to_vec()
                };
                let name = String::from_utf8(name_bytes).unwrap();
                let sel: SEL = env
                    .objc
                    .register_host_selector(
                        "parser:didStartElement:namespaceURI:qualifiedName:attributes:".to_string(),
                        &mut env.mem
                    );
                let responds: bool = msg![env; delegate respondsToSelector:sel];
                if responds {
                    let name: id = from_rust_string(env, name);
                    let name = autorelease(env, name);
                    let dict = build_attributes_dict(env, e);
                    () = msg![env; delegate parser:this
                                   didStartElement:name
                                      namespaceURI:nil
                                     qualifiedName:nil
                                        attributes:dict];
                }
            }
            Event::End(e) => {
                let name_bytes = if should_process_namespaces {
                    e.local_name().as_ref().to_vec()
                } else {
                    e.name().as_ref().to_vec()
                };
                let name = String::from_utf8(name_bytes).unwrap();
                let sel: SEL = env
                    .objc
                    .register_host_selector(
                        "parser:didEndElement:namespaceURI:qualifiedName:".to_string(),
                        &mut env.mem
                    );
                let responds: bool = msg![env; delegate respondsToSelector:sel];
                if responds {
                    let name: id = from_rust_string(env, name);
                    let name = autorelease(env, name);
                    () = msg![env; delegate parser:this
                                     didEndElement:name
                                      namespaceURI:nil
                                     qualifiedName:nil];
                }
            }
            Event::CData(e) => {
                let sel: SEL = env
                    .objc
                    .register_host_selector("parser:foundCDATA:".to_string(), &mut env.mem);
                let responds: bool = msg![env; delegate respondsToSelector:sel];
                if responds {
                    todo!("Implement parser:foundCDATA: delegate call");
                } else {
                    let sel: SEL = env
                        .objc
                        .register_host_selector("parser:foundCharacters:".to_string(), &mut env.mem);
                    let responds: bool = msg![env; delegate respondsToSelector:sel];
                    if responds {
                        let text = e.decode().unwrap().to_string();
                        let text = from_rust_string(env, text);
                        let text = autorelease(env, text);
                        () = msg![env; delegate parser:this foundCharacters:text];
                    }
                }
            }
            Event::Comment(e) => {
                let comment = e.decode().unwrap().to_string();
                let sel: SEL = env
                    .objc
                    .register_host_selector("parser:foundComment:".to_string(), &mut env.mem);
                let responds: bool = msg![env; delegate respondsToSelector:sel];
                if responds {
                    let comment = from_rust_string(env, comment);
                    let comment = autorelease(env, comment);
                    () = msg![env; delegate parser:this foundComment:comment];
                }
            }
            Event::Decl(_) => {
                let sel: SEL = env
                    .objc
                    .register_host_selector("parser:foundElementDeclarationWithName:model:".to_string(), &mut env.mem);
                let responds: bool = msg![env; delegate respondsToSelector:sel];
                assert!(!responds); // TODO
            }
            e => unimplemented!("{:?}", e)
        }
        if env.objc.borrow::<NSXMLParserHostObject>(this).aborted {
            return false;
        }
    }
    if let Some((position, description)) = parse_failure {
        log!(
            "NSXMLParser failed at byte {}: {}",
            position,
            description
        );
        let domain = get_static_str(env, "NSXMLParserErrorDomain");
        // The underlying parser is not libxml, so do not fabricate a precise
        // libxml error code. Code 1 is the historical generic parser-error
        // value; the bounded log retains the useful host detail.
        let error: id = msg_class![env; NSError errorWithDomain:domain code:1i32 userInfo:nil];
        retain(env, error);
        env.objc.borrow_mut::<NSXMLParserHostObject>(this).parser_error = error;

        let sel = env.objc.register_host_selector(
            "parser:parseErrorOccurred:".to_string(),
            &mut env.mem,
        );
        let responds: bool = msg![env; delegate respondsToSelector:sel];
        if responds {
            () = msg![env; delegate parser:this parseErrorOccurred:error];
        }
        return false;
    }
    let sel: SEL = env
        .objc
        .register_host_selector("parserDidEndDocument:".to_string(), &mut env.mem);
    let responds: bool = msg![env; delegate respondsToSelector:sel];
    if responds {
        () = msg![env; delegate parserDidEndDocument:this];
    }
    true
}

- (id)parserError {
    env.objc.borrow::<NSXMLParserHostObject>(this).parser_error
}

- (())dealloc {
    let &NSXMLParserHostObject { data, parser_error, .. } = env.objc.borrow(this);
    release(env, data);
    release(env, parser_error);
    env.objc.dealloc_object(this, &mut env.mem);
}

@end

};

/// A helper function to build an attributes NSDictionary from an XML tag.
/// Each key/value pair is copied and retained in the dict.
fn build_attributes_dict(env: &mut Environment, e: BytesStart) -> id {
    let pairs = e.attributes().map(|a| a.unwrap()).map(|a| {
        (
            String::from_utf8(a.key.local_name().as_ref().to_vec()).unwrap(),
            a.unescape_value().unwrap().to_string(),
        )
    });
    let dict: id = msg_class![env; NSMutableDictionary new];
    for (x, y) in pairs {
        let key = from_rust_string(env, x);
        let val = from_rust_string(env, y);
        () = msg![env; dict setObject:val forKey:key];
        release(env, key);
        release(env, val);
    }
    log_dbg!("attributes {}", {
        let desc = msg![env; dict description];
        to_rust_string(env, desc)
    });
    // TODO: return an immutable copy
    autorelease(env, dict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn development_255_cdata_delegate_payload_preserves_raw_bytes() {
        let mut reader = Reader::from_str("<root><![CDATA[Potty <&> Racers]]></root>");
        loop {
            match reader.read_event().unwrap() {
                Event::CData(event) => {
                    assert_eq!(cdata_payload(event), b"Potty <&> Racers");
                    break;
                }
                Event::Eof => panic!("CDATA event was not produced"),
                _ => {}
            }
        }
    }

    #[test]
    fn namespace_configuration_is_stable_per_parser() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let parser: id = msg_class![env; NSXMLParser new];
        () = msg![env; parser setShouldProcessNamespaces:true];
        () = msg![env; parser setShouldReportNamespacePrefixes:true];
        () = msg![env; parser setShouldResolveExternalEntities:true];
        let process: bool = msg![env; parser shouldProcessNamespaces];
        let report: bool = msg![env; parser shouldReportNamespacePrefixes];
        let resolve: bool = msg![env; parser shouldResolveExternalEntities];
        assert!(process && report && resolve);
        release(env, parser);
    }

    #[test]
    fn stale_abort_flag_is_reset_by_a_new_parse() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let xml = b"<root />";
        let length = xml.len() as u32;
        let bytes = env.mem.alloc(length);
        env.mem
            .bytes_at_mut(bytes.cast(), length)
            .copy_from_slice(xml);
        let bytes = bytes.cast_const();
        let data: id = msg_class![env; NSData dataWithBytes:bytes length:length];
        let parser: id = msg_class![env; NSXMLParser alloc];
        let parser: id = msg![env; parser initWithData:data];
        () = msg![env; parser abortParsing];
        // A new parse resets a prior, out-of-callback abort just as native
        // NSXMLParser does; callback-triggered abort behavior is exercised by
        // the state checks in the event loop.
        let parsed: bool = msg![env; parser parse];
        assert!(parsed);
        release(env, parser);
    }

    #[test]
    fn dev_244_malformed_xml_returns_false_and_persists_parser_error() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(1024);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 1024;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let xml = b"<root>";
        let length = xml.len() as u32;
        let bytes = env.mem.alloc(length);
        env.mem.bytes_at_mut(bytes.cast(), length).copy_from_slice(xml);
        let data: id = msg_class![env; NSData dataWithBytes:(bytes.cast_const()) length:length];
        let parser: id = msg_class![env; NSXMLParser alloc];
        let parser: id = msg![env; parser initWithData:data];

        let parsed: bool = msg![env; parser parse];
        assert!(!parsed);
        let error: id = msg![env; parser parserError];
        assert_ne!(error, nil);

        release(env, parser);
        release(env, pool);
    }
}
