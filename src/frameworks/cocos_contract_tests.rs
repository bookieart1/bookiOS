//! Focused guest-ABI reproductions for the Dev 2.24 Foundation/bitmap batch.
use crate::abi::{CallFromHost, GuestFunction};
use crate::frameworks::core_foundation::time::CFGregorianDate;
use crate::frameworks::foundation::ns_string::from_rust_string;
use crate::mem::{ConstPtr, MutVoidPtr};
use crate::objc::{id, msg, msg_class, nil, release, retain, SEL};
use crate::Environment;

fn api(env: &mut Environment, name: &str) -> GuestFunction {
    let &(symbol, function) = crate::dyld::DYLIB_LIST.iter()
        .flat_map(|lib| lib.function_exports.iter().flat_map(|list| list.iter()))
        .find(|(symbol, _)| *symbol == name).unwrap();
    env.dyld.create_guest_function(&mut env.mem, symbol, function)
}

// A real guest call can yield at each SVC. Supply a stack and a bounded
// cooperative executor, instead of using the host-only fake fixture directly.
fn with_guest_test(test: impl FnOnce(&mut Environment) + 'static) {
    use corosensei::{Coroutine, CoroutineResult};
    let mut env = Environment::new_fake_for_test_with_thread();
    let stack = env.mem.alloc(0x10000);
    env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x10000;
    env.threads[0].stack = Some(stack.to_bits()..=stack.to_bits() + 0xffff);
    let mut coroutine = Coroutine::new(move |yielder, mut env: Environment| {
        env.with_yielder(yielder, move |env| {
            let pool: id = msg_class![env; NSAutoreleasePool new];
            test(env);
            release(env, pool);
        });
        env
    });
    for _ in 0..10_000 {
        match coroutine.resume(env) {
            CoroutineResult::Return(_) => return,
            CoroutineResult::Yield(next) => {
                assert!(matches!(next.threads[0].blocked_by,
                    crate::environment::ThreadBlock::NotBlocked));
                env = next;
            }
        }
    }
    panic!("guest ABI fixture exceeded its yield budget");
}

#[test]
fn cocos_contract_signed_dates_use_guest_struct_return() {
    with_guest_test(|env| {
        let f = api(env, "_CFAbsoluteTimeGetGregorianDate");
        for (time, year, month, day, hours, minutes, seconds) in [
            (-3_061_152_000.0, 1904, 1, 1, 0, 0, 0.0),
            (-978_307_200.0, 1970, 1, 1, 0, 0, 0.0),
            (-0.25, 2000, 12, 31, 23, 59, 59.75),
            (0.0, 2001, 1, 1, 0, 0, 0.0),
            (1_262_304_000.5, 2041, 1, 1, 0, 0, 0.5),
        ] {
            let out = env.mem.alloc(16).cast::<CFGregorianDate>();
            // Darwin ARM returns this struct through an explicit r0 pointer.
            let _: () = f.call_from_host(env, (out, time, nil));
            let actual = env.mem.read(out);
            env.mem.free(out.cast());
            assert_eq!(actual, CFGregorianDate { year, month, day, hours, minutes, seconds });
        }
    });
}

#[test]
fn cocos_contract_unsigned_byte_boxing_round_trips() {
    with_guest_test(|env| {
        for value in [0u8, 127, 128, 255] {
            let number: id = msg_class![env; NSNumber numberWithUnsignedChar:value];
            let byte: u8 = msg![env; number unsignedCharValue];
            let wide: u32 = msg![env; number unsignedIntValue];
            assert_eq!((byte, wide), (value, u32::from(value)));
            let data: id = msg_class![env; NSKeyedArchiver archivedDataWithRootObject:number];
            let decoded: id = msg_class![env; NSKeyedUnarchiver unarchiveObjectWithData:data];
            let wide: u32 = msg![env; decoded unsignedIntValue];
            assert_eq!(wide, u32::from(value));
        }
    });
}

#[test]
fn cocos_contract_kvc_keeps_object_identity_for_guest_setter() {
    with_guest_test(|env| {
        let class: id = msg_class![env; NSObject class];
        let select = api(env, "_sel_registerName");
        let name = env.mem.alloc_and_write_cstr(b"setPayload:").cast_const();
        let selector: SEL = select.call_from_host(env, (name,));
        let output = env.mem.alloc_and_write(nil);
        let code = env.mem.alloc(16).cast::<u32>();
        // ARM setter: store its third argument verbatim, with no boxing or
        // retain.
        for (i, word) in [0xe59f_3004u32, 0xe583_2000, 0xe12f_ff1e, output.to_bits()].into_iter().enumerate() {
            env.mem.write(code + i as u32, word);
        }
        let implementation = GuestFunction::from_addr_with_thumb_bit(code.to_bits());
        let types: ConstPtr<u8> = env.mem.alloc_and_write_cstr(b"v12@0:4@8").cast_const();
        let add = api(env, "_class_addMethod");
        let added: bool = add.call_from_host(env, (class, selector, implementation, types));
        assert!(added);
        let object: id = msg![env; class new];
        let key = from_rust_string(env, "payload".into());
        let number: id = msg_class![env; NSNumber numberWithInt:255];
        let value: id = msg_class![env; NSValue valueWithCGPoint:(crate::frameworks::core_graphics::CGPoint { x: 1.0, y: 2.0 })];
        for input in [number, value, nil] {
            () = msg![env; object setValue:input forKey:key];
            assert_eq!(env.mem.read(output), input);
        }
        for (method, encoding, property, expected) in [
            ("setSigned:", "v12@0:4i8", "signed", -17i32 as u32),
            ("setUnsigned:", "v12@0:4I8", "unsigned", 255u32),
        ] {
            let name = env.mem.alloc_and_write_cstr(method.as_bytes()).cast_const();
            let selector: SEL = select.call_from_host(env, (name,));
            let types = env.mem.alloc_and_write_cstr(encoding.as_bytes()).cast_const();
            let added: bool = add.call_from_host(env, (class, selector, implementation, types));
            assert!(added);
            let number: id = msg_class![env; NSNumber numberWithInt:(expected as i32)];
            let key = from_rust_string(env, property.into());
            () = msg![env; object setValue:number forKey:key];
            assert_eq!(env.mem.read(output).to_bits(), expected);
            release(env, key);
        }
        release(env, key);
        release(env, object);
    });
}

#[test]
fn cocos_contract_primitive_archive_booleans_reuse_uids_and_survive_teardown() {
    with_guest_test(|env| {
        use plist::{Dictionary, Uid, Value};
        let mut objects = Dictionary::new();
        objects.insert("$version".into(), Value::Integer(100000.into()));
        objects.insert("$archiver".into(), Value::String("NSKeyedArchiver".into()));
        objects.insert("$objects".into(), Value::Array(vec![Value::String("$null".into()), Value::Boolean(true), Value::Boolean(false)]));
        let mut top = Dictionary::new();
        top.insert("yes".into(), Value::Uid(Uid::new(1)));
        top.insert("again".into(), Value::Uid(Uid::new(1)));
        top.insert("no".into(), Value::Uid(Uid::new(2)));
        objects.insert("$top".into(), Value::Dictionary(top));
        let mut bytes = Vec::new();
        Value::Dictionary(objects).to_writer_binary(&mut bytes).unwrap();
        let buffer: MutVoidPtr = env.mem.alloc(bytes.len() as u32);
        env.mem.bytes_at_mut(buffer.cast(), bytes.len() as u32).copy_from_slice(&bytes);
        let data: id = msg_class![env; NSData dataWithBytes:(buffer.cast_const()) length:(bytes.len() as u32)];
        let decoder: id = msg_class![env; NSKeyedUnarchiver alloc];
        let decoder: id = msg![env; decoder initForReadingWithData:data];
        let mut results = Vec::new();
        for name in ["yes", "again", "no"] {
            let key = from_rust_string(env, name.into());
            let value: id = msg![env; decoder decodeObjectForKey:key];
            results.push(value);
            retain(env, value);
            release(env, key);
        }
        assert_eq!(results[0], results[1]);
        release(env, decoder);
        let yes: bool = msg![env; (results[0]) boolValue];
        let no: bool = msg![env; (results[2]) boolValue];
        assert!(yes);
        assert!(!no);
        for value in results { release(env, value); }
        env.mem.free(buffer);
    });
}

#[test]
fn cocos_contract_alpha_bitmap_null_space_padding_and_image() {
    with_guest_test(|env| {
        use crate::frameworks::core_graphics::{CGPoint, CGRect, CGSize};
        let data = env.mem.alloc(8);
        env.mem.bytes_at_mut(data.cast(), 8).copy_from_slice(&[0, 0, 238, 238, 0, 0, 238, 238]);
        let create = api(env, "_CGBitmapContextCreate");
        let context: id = create.call_from_host(env, (data, 2u32, 2u32, 8u32, 4u32, nil, 7u32));
        let fill_color = api(env, "_CGContextSetRGBFillColor");
        let _: () = fill_color.call_from_host(env, (context, 1.0f32, 0.0f32, 0.0f32, 0.5f32));
        let rect = CGRect { origin: CGPoint { x: 0.0, y: 0.0 }, size: CGSize { width: 2.0, height: 2.0 } };
        let fill = api(env, "_CGContextFillRect");
        let _: () = fill.call_from_host(env, (context, rect));
        let bytes = env.mem.bytes_at(data.cast(), 8);
        assert!(bytes[0] == 127 || bytes[0] == 128);
        assert_eq!(bytes[0], bytes[1]);
        assert_eq!(&bytes[0..4], &bytes[4..8]);
        assert_eq!(&bytes[2..4], &[238, 238]);
        let make_image = api(env, "_CGBitmapContextCreateImage");
        let image: id = make_image.call_from_host(env, (context,));
        assert_ne!(image, nil);
        release(env, image);
        let clear = api(env, "_CGContextClearRect");
        let _: () = clear.call_from_host(env, (context, rect));
        assert_eq!(env.mem.bytes_at(data.cast(), 8), &[0, 0, 238, 238, 0, 0, 238, 238]);
        release(env, context);
        // Caller-owned storage remains valid after the context is destroyed.
        assert_eq!(env.mem.read(data.cast::<u8>()), 0);
        env.mem.free(data);
    });
}

#[test]
fn cocos_contract_offline_stream_callback_abi_and_lifetime() {
    with_guest_test(|env| {
        use crate::frameworks::core_foundation::cf_socket_stream::StreamError;
        let host = from_rust_string(env, "analytics.example".into());
        let read = env.mem.alloc_and_write(nil);
        let write = env.mem.alloc_and_write(nil);
        let create = api(env, "_CFStreamCreatePairWithSocketToHost");
        let _: () = create.call_from_host(env, (nil, host, 80u32, read, write));
        let stream = env.mem.read(read);
        let output = env.mem.alloc(20).cast::<u32>();
        for i in 0..5 { env.mem.write(output + i, 0u32); }
        // callback(stream,event,info): increment count; save stream and event.
        let words = [0xe592_3000u32, 0xe283_3001, 0xe582_3000, 0xe582_0004, 0xe582_1008, 0xe12f_ff1e];
        let code = env.mem.alloc(24).cast::<u32>();
        for (i, word) in words.into_iter().enumerate() { env.mem.write(code + i as u32, word); }
        // Context retain/release callbacks update distinct counters; retain
        // returns
        // info.
        let retain_code = env.mem.alloc(16).cast::<u32>();
        let release_code = env.mem.alloc(16).cast::<u32>();
        for (address, offset) in [(retain_code, 12u32), (release_code, 16u32)] {
            for (i, word) in [0xe590_1000 | offset, 0xe281_1001, 0xe580_1000 | offset, 0xe12f_ff1e].into_iter().enumerate() {
                env.mem.write(address + i as u32, word);
            }
        }
        let context = env.mem.alloc(20).cast::<u32>();
        for (i, word) in [0, output.to_bits(), retain_code.to_bits(), release_code.to_bits(), 0].into_iter().enumerate() {
            env.mem.write(context + i as u32, word);
        }
        let client = api(env, "_CFReadStreamSetClient");
        let ok: bool = client.call_from_host(env, (stream, 8u32, GuestFunction::from_addr_with_thumb_bit(code.to_bits()), context.cast::<std::ffi::c_void>()));
        assert!(ok);
        assert_eq!(env.mem.read(output + 3), 1);
        let run_loop: id = msg_class![env; NSRunLoop currentRunLoop];
        let mode = from_rust_string(env, "kCFRunLoopDefaultMode".into());
        let schedule = api(env, "_CFReadStreamScheduleWithRunLoop");
        let _: () = schedule.call_from_host(env, (stream, run_loop, mode));
        let open = api(env, "_CFReadStreamOpen");
        let opened: bool = open.call_from_host(env, (stream,));
        assert!(!opened);
        assert_eq!(env.mem.read(output), 0, "no synchronous callback from open");
        let get_error = api(env, "_CFReadStreamGetError");
        let out = env.mem.alloc(8).cast::<StreamError>();
        let _: () = get_error.call_from_host(env, (out, stream));
        let error = env.mem.read(out);
        env.mem.free(out.cast());
        let (domain, code) = (error.domain, error.error);
        assert_eq!((domain, code), (1, crate::libc::errno::ENETUNREACH));
        // Invoke the queued selector directly: the fake environment has no
        // window.
        () = msg![env; stream _deliverOfflineError:run_loop];
        () = msg![env; stream _deliverOfflineError:run_loop];
        assert_eq!(env.mem.read(output), 1, "one failure per stream");
        assert_eq!(env.mem.read(output + 1), stream.to_bits());
        assert_eq!(env.mem.read(output + 2), 8);
        assert_eq!(env.mem.read(output + 3), 2);
        assert_eq!(env.mem.read(output + 4), 1);
        let close = api(env, "_CFReadStreamClose");
        let _: () = close.call_from_host(env, (stream,));
        let ok: bool = client.call_from_host(env, (stream, 0u32, GuestFunction::from_addr_with_thumb_bit(0), MutVoidPtr::null()));
        assert!(ok);
        assert_eq!(env.mem.read(output + 4), 2);
        () = msg![env; run_loop cancelPerformSelectorsWithTarget:stream];
        release(env, stream);
        let write = env.mem.read(write);
        release(env, write);
        release(env, mode);
        release(env, host);
    });
}

#[test]
fn cocos_contract_iad_configuration_never_reports_loaded() {
    with_guest_test(|env| {
        use crate::frameworks::core_graphics::CGSize;
        let banner: id = msg_class![env; ADBannerView new];
        let identifier = from_rust_string(env, "ADBannerContentSizeIdentifier480x32".into());
        let size: CGSize = msg_class![env; ADBannerView sizeFromBannerContentSizeIdentifier:identifier];
        assert_eq!((size.width, size.height), (480.0, 32.0));
        () = msg![env; banner setCurrentContentSizeIdentifier:identifier];
        release(env, identifier);
        let current: id = msg![env; banner currentContentSizeIdentifier];
        assert_eq!(crate::frameworks::foundation::ns_string::to_rust_string(env, current), "ADBannerContentSizeIdentifier480x32");
        let loaded: bool = msg![env; banner isBannerLoaded];
        let action: bool = msg![env; banner isBannerViewActionInProgress];
        assert!(!loaded && !action);
        () = msg![env; banner cancelBannerViewAction];
        let delegate: id = msg_class![env; NSObject new];
        let class: id = msg![env; delegate class];
        let output = env.mem.alloc(8).cast::<u32>();
        env.mem.write(output, 0u32);
        env.mem.write(output + 1, 0u32);
        let code = env.mem.alloc(20).cast::<u32>();
        for (i, word) in [0xe59f_c008u32, 0xe58c_2000, 0xe58c_3004, 0xe12f_ff1e, output.to_bits()].into_iter().enumerate() {
            env.mem.write(code + i as u32, word);
        }
        let selector = env.objc.register_host_selector("bannerView:didFailToReceiveAdWithError:".into(), &mut env.mem);
        let types = env.mem.alloc_and_write_cstr(b"v16@0:4@8@12").cast_const();
        let add = api(env, "_class_addMethod");
        let added: bool = add.call_from_host(env, (class, selector, GuestFunction::from_addr_with_thumb_bit(code.to_bits()), types));
        assert!(added);
        () = msg![env; banner setDelegate:delegate];
        assert_eq!(env.mem.read(output), 0, "delegate must not run inline from setter");
        () = msg![env; banner _deliverUnavailableAd:nil];
        assert_eq!(env.mem.read(output), banner.to_bits());
        let error: id = crate::mem::Ptr::from_bits(env.mem.read(output + 1));
        let code: i32 = msg![env; error code];
        assert_eq!(code, 1);
        let domain: id = msg![env; error domain];
        assert_eq!(crate::frameworks::foundation::ns_string::to_rust_string(env, domain), "ADErrorDomain");
        let run_loop: id = msg_class![env; NSRunLoop mainRunLoop];
        () = msg![env; run_loop cancelPerformSelectorsWithTarget:banner];
        () = msg![env; banner setDelegate:nil];
        release(env, delegate);
        release(env, banner);
    });
}
