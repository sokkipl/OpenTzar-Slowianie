import argparse, hashlib, json, pathlib, re, zipfile
p=argparse.ArgumentParser()
p.add_argument('--zip', required=True); p.add_argument('--tag', required=True); p.add_argument('--version', required=True); p.add_argument('--previous'); p.add_argument('--out',required=True)
a=p.parse_args()
out=pathlib.Path(a.out); out.mkdir(parents=True,exist_ok=True)
base='https://github.com/sokkipl/OpenTzar-Slowianie/releases/download/'
def digest(b): return hashlib.sha256(b).hexdigest()
def valid(n): return not (n.startswith('/') or '\\' in n or ':' in n or any(v in ('','.','..') for v in n.split('/')))
with zipfile.ZipFile(a.zip) as z:
 names=[i.filename for i in z.infolist() if not i.is_dir()]
 assert all(valid(n) for n in names)
 assert len(set(n.lower() for n in names))==len(names)
 assert 'OpenTzar.exe' in names and '.ot-launcher-manifest.json' not in names
 files={n:digest(z.read(n)) for n in sorted(names)}
 m=dict(version=a.version,full_url=base+a.tag+'/'+pathlib.Path(a.zip).name,full_sha256=digest(pathlib.Path(a.zip).read_bytes()),files=files,notes='OpenTzar '+a.version+' — wymaga oryginalnej instalacji Tzara.')
 if a.previous and pathlib.Path(a.previous).is_file():
  old=json.loads(pathlib.Path(a.previous).read_text(encoding='utf-8-sig'))
  if old['version']!=a.version:
   name='delta-'+re.sub(r'[^A-Za-z0-9._-]','_',old['version'])+'-to-'+re.sub(r'[^A-Za-z0-9._-]','_',a.version)+'-'+m['full_sha256'][:12]+'.zip'
   d=out/name
   with zipfile.ZipFile(d,'w',zipfile.ZIP_DEFLATED) as dz:
    for n,h in files.items():
     if old['files'].get(n)!=h: dz.writestr(n,z.read(n))
   if d.stat().st_size<pathlib.Path(a.zip).stat().st_size:
    m.update(delta_from=old['version'],delta_url=base+'launcher-channel/'+name,delta_sha256=digest(d.read_bytes()))
   else: d.unlink()
 (out/'update.json').write_text(json.dumps(m,ensure_ascii=False,indent=2),encoding='utf-8')
