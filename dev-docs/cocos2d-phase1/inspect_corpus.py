"""Read-only IPA/log census. Standard library only; never executes guest code.
Outputs metadata and bounded evidence, not executables or game resources.
"""
import collections, gzip, hashlib, json, plistlib, re, struct, sys, zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent
IPAS = Path(r'C:\Users\booki\Documents\Codex\iDared-IPAs\cocos2d-iphone IPAs')
LOGS = Path(r'C:\Users\booki\.codex\codex-remote-attachments\01a18bbd-7a8b-7890-808e-11d591147e3b\50FCDE2D-AF2B-4773-8BFB-94A03DADF83E')

def save(name, obj):
    data=(json.dumps(obj, indent=2, ensure_ascii=False, default=str)+'\n').encode('utf8')
    if name.endswith('.gz'):
        (ROOT/name).write_bytes(gzip.compress(data, mtime=0))
    else:
        (ROOT/name).write_bytes(data)

def cstr(b, p):
    if p < 0 or p >= len(b): return ''
    end=b.find(b'\0',p)
    return b[p:end if end>=0 else len(b)].decode('utf8','replace')

def macho(blob):
    if blob[:4] == b'\xca\xfe\xba\xbe':
        chunks=[struct.unpack_from('>5I',blob,8+i*20)[2:4] for i in range(struct.unpack_from('>I',blob,4)[0])]
    else: chunks=[(0,len(blob))]
    results=[]
    for off, size in chunks:
        b=blob[off:off+size]
        if b[:4] not in [b'\xce\xfa\xed\xfe',b'\xcf\xfa\xed\xfe']:
            results.append({'error':'unsupported Mach-O magic','magic':b[:4].hex()}); continue
        wide=b[:4]==b'\xcf\xfa\xed\xfe'
        cpu,sub,_,ncmds=struct.unpack_from('<4I',b,4)
        r={'architecture':{(12,6):'armv6',(12,9):'armv7',(12,11):'armv7s',(16777228,0):'arm64'}.get((cpu,sub & 0xffffff),f'{cpu}:{sub}'), 'encryption':[], 'dylibs':[], 'sections':[], 'imports':[], 'defined_symbols':[]}
        p=32 if wide else 28; sym=None
        for _ in range(ncmds):
            cmd,sz=struct.unpack_from('<II',b,p)
            if sz<8 or p+sz>len(b): raise ValueError('invalid load command')
            if cmd==2: sym=struct.unpack_from('<4I',b,p+8)
            if cmd in [0x21,0x2c]: r['encryption'].append(dict(zip(['offset','size','cryptid'],struct.unpack_from('<3I',b,p+8))))
            if cmd & 0x7fffffff in [12,24,31,32,35]: r['dylibs'].append(cstr(b,p+struct.unpack_from('<I',b,p+8)[0]))
            if cmd==1:
                for j in range(struct.unpack_from('<I',b,p+48)[0]):
                    q=p+56+j*68
                    r['sections'].append(dict(name=cstr(b[q:q+16],0),segment=cstr(b[q+16:q+32],0),addr=struct.unpack_from('<I',b,q+32)[0],size=struct.unpack_from('<I',b,q+36)[0],offset=struct.unpack_from('<I',b,q+40)[0]))
            p+=sz
        if sym:
            so,n,st,ss=sym
            strings_table=b[st:st+ss]
            for i in range(n):
                q=so+i*(16 if wide else 12)
                idx,typ,sec,desc,val=struct.unpack_from('<IBBHQ' if wide else '<IBBHI',b,q)
                name=cstr(strings_table,idx)
                if not name: continue
                if typ & 0xe ==0 and not typ & 0xe0: r['imports'].append(name)
                elif not typ & 0xe0: r['defined_symbols'].append({'address':hex(val),'name':name})
        def va(a):
            for s in r['sections']:
                if s['addr']<=a<s['addr']+s['size']: return s['offset']+a-s['addr']
            raise ValueError('unmapped VA')
        def u32(a): return struct.unpack_from('<I',b,va(a))[0]
        classes=[]
        # ObjC2 class_t -> class_ro_t.name on 32-bit. ObjC1 names remain in section evidence.
        for s in r['sections']:
            if s['name']=='__objc_classlist' and not wide:
                for q in range(s['offset'],s['offset']+s['size'],4):
                    try:
                        cl=struct.unpack_from('<I',b,q)[0]; ro=u32(cl+16)&~3
                        classes.append(cstr(b,va(u32(ro+16))))
                    except (ValueError,struct.error): pass
        strings=[]; selectors=[]; metadata_names=[]
        for s in r['sections']:
            if s['name'] in ['__cstring','__objc_methname','__objc_classname','__class_names','__meth_var_names']:
                vals=[x.decode('utf8','replace') for x in b[s['offset']:s['offset']+s['size']].split(b'\0') if x]
                if s['name'] in ['__objc_methname','__meth_var_names']: selectors+=vals
                elif s['name'] in ['__objc_classname','__class_names']: metadata_names+=vals
                else: strings+=vals
        r['classes']=sorted(set(classes)); r['class_name_strings']=sorted(set(metadata_names)); r['selectors']=sorted(set(selectors))
        pool=sorted(set(strings+classes+metadata_names+[s['name'] for s in r['defined_symbols']]+r['imports']))
        patterns={'cocos2d':r'cocos2d|^CCDirector|^CCSprite|^CCNode|^Director$|^CocosNode$', 'CocosDenshion':r'CocosDenshion|SimpleAudioEngine|^CDSound|^CDAudio', 'Box2D':r'Box2D|b2World|b2Body', 'Chipmunk':r'chipmunk|cpSpace|cpBody', 'OpenFeint':r'OpenFeint|OFRequest|OFGame', 'Crystal':r'CrystalSession|CrystalPlayer|Chillingo|CCPrivateSession', 'GameKit':r'GKLocalPlayer|GKScore|GKMatch|GameCenter', 'ads':r'ADBanner|AdMob|Chartboost|RevMob|Flurry|Tapjoy|iAd|Mobclix', 'ES2':r'^_gl(CreateShader|CreateProgram|ShaderSource|UseProgram|LinkProgram)|CCGLProgram|CCShader', 'OpenAL':r'^_al(Source|Buffer|Gen|Delete)|^_alc(Open|Create)', 'other_engines':r'Unity|SIO2|irrlicht|cocos2d-x'}
        r['markers']={k:[s[:220] for s in pool if re.search(pat,s,re.I if k not in ['ES2','OpenAL'] else 0)][:30] for k,pat in patterns.items()}
        r['cocos_version_strings']=[s[:240] for s in strings if re.search(r'cocos2d.*(v\d|\d\.\d)|^cocos2d v',s,re.I)]
        r['useful_strings']=[s[:260] for s in strings if re.search(r'cocos|texture|PVRTC|PVR|OpenGL|framebuffer|assert|shader|OpenFeint|CocosDenshion',s,re.I)][:100]
        results.append(r)
    return results

def inspect_ipa(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for b in iter(lambda:f.read(1024*1024),b''): h.update(b)
    r={'filename':path.name,'sha256':h.hexdigest(),'bytes':path.stat().st_size}
    with zipfile.ZipFile(path) as z:
        names=z.namelist(); pn=next(n for n in names if re.match(r'^Payload/[^/]+\.app/Info.plist$',n))
        info=plistlib.loads(z.read(pn)); base=pn[:-10]; exe=base+info['CFBundleExecutable']
        r.update(info=info,executable=exe,entries=len(names),extensions=dict(collections.Counter(Path(n).suffix.lower() for n in names if not n.endswith('/'))))
        r['resource_examples']=[n[len(base):] for n in names if re.search(r'\.(pvr|ccz|pkm|ktx|tmx|fnt|ttf|otf|caf|ogg|mp3|m4a|aac|wav)$',n,re.I)][:100]
        r['macho']=macho(z.read(exe))
        # CRC every member, bounded to streaming decompression; no extraction.
        r['zip_crc_failure']=z.testzip()
    return r

def logs():
    result=[]
    paths=[(p,p.name) for p in sorted(LOGS.glob('*-Log.txt'))]
    clown=LOGS.parent/'127582EE-3D3B-4F73-8609-B5A44C8125B4'/'1-Log.txt'
    if clown.exists(): paths.append((clown,'3-Clowning-Around-Log.txt'))
    for path, archived_name in paths:
        raw=path.read_bytes(); text=raw.decode('utf-8-sig',errors='replace'); lines=text.splitlines()
        (ROOT/'logs').mkdir(exist_ok=True)
        (ROOT/'logs'/archived_name).write_bytes(raw)
        starts=[i for i,l in enumerate(lines) if l.startswith('Picked:')]
        for n,a in enumerate(starts):
            end=starts[n+1] if n+1<len(starts) else len(lines); ls=lines[a:end]
            def field(key): return next((l[len(key):] for l in ls if l.startswith(key)),'')
            panics=[(a+i+1,l) for i,l in enumerate(ls) if l.startswith('Panic')]
            # Context includes last live events plus first failure; full segments preserved below.
            p=next((i for i,l in enumerate(ls) if l.startswith('Panic')),len(ls))
            selected=[(a+i+1,l) for i,l in enumerate(ls) if re.search(r'flight recorder|flight summary|flight progress|counter|checkpoint|main.loop|snapshot reason|encrypted|Could not|Error opening|error opening|Unhandled|unimplemented|HomeButton|CPU emulation begins',l,re.I)]
            r={'source':str(path),'log':archived_name,'log_sha256':hashlib.sha256(raw).hexdigest(),'start_line':a+1,'end_line':end,'build_header':lines[:2],'ipa_filename':ls[0].rsplit('/',1)[-1],'display_name':field('- Display name: '),'bundle_id':field('- Identifier: '),'version':field('- Version: '),'panics':panics,'pre_failure':[(a+i+1,ls[i]) for i in range(max(0,p-22),min(len(ls),p+18))], 'highlights':selected,'home':'Handling HomeButton event' in '\n'.join(ls),'cpu_started':'CPU emulation begins now.' in ls}
            slug=re.sub(r'[^a-z0-9]+','-',r['display_name'].lower()).strip('-') or f'unknown-{n}'
            r['segment']=f'logs/{Path(archived_name).stem}-{slug}.txt'
            (ROOT/r['segment']).write_text('\n'.join(ls)+'\n',encoding='utf8')
            result.append(r)
    save('log-index.json',result)
    print('LOGS',len(result))
    for r in result: print(r['log'],r['start_line'],r['display_name'],r['bundle_id'],r['panics'][0][1] if r['panics'] else 'NO PANIC', 'HOME' if r['home'] else '')

if __name__=='__main__':
    logs()
    if '--logs-only' in sys.argv: sys.exit(0)
    results=[]
    for path in sorted(IPAS.glob('*.ipa')):
        try: r=inspect_ipa(path)
        except Exception as e: r={'filename':path.name,'error':repr(e)}
        results.append(r)
        print('IPA',path.name,'OK' if 'error' not in r else r['error'],flush=True)
    save('ipa-index.json.gz',results)
