# Minimal MSF/PDB public-symbol extractor (S_PUB32) -> "va name" sorted. Claude opt profiler.
import sys,struct,mmap
f=open(sys.argv[1],'rb'); m=mmap.mmap(f.fileno(),0,access=mmap.ACCESS_READ)
bs,fpm,nblocks,ndirbytes,_,bmaddr=struct.unpack_from('<6I',m,32)
ndirblocks=(ndirbytes+bs-1)//bs
dirblocks=struct.unpack_from(f'<{ndirblocks}I',m,bmaddr*bs)
dirdata=b''.join(m[b*bs:(b+1)*bs] for b in dirblocks)[:ndirbytes]
ns=struct.unpack_from('<I',dirdata,0)[0]; sizes=struct.unpack_from(f'<{ns}I',dirdata,4)
pos=4+4*ns; blocks=[]
for s in sizes:
    n=0 if s==0xFFFFFFFF else (s+bs-1)//bs
    blocks.append(struct.unpack_from(f'<{n}I',dirdata,pos)); pos+=4*n
def stream(i):
    s=sizes[i]; return b''.join(m[b*bs:(b+1)*bs] for b in blocks[i])[:s]
dbi=stream(3)
(vsig,vhdr,age,gsi,build,psi,dllver,symrec,rbld,modsz,scsz,smsz,srcsz,tsmsz,mfc,dbgsz,ecsz,flags,mach,pad)=struct.unpack_from('<iIIHHHHHHiiiiiIiiHHI',dbi,0)
off=64+modsz+scsz+smsz+srcsz+tsmsz+ecsz
dbg=struct.unpack_from(f'<{dbgsz//2}H',dbi,off)
sech=stream(dbg[5]); secs=[struct.unpack_from('<I',sech,k+12)[0] for k in range(0,len(sech),40)]
d=stream(symrec); out=[]; i=0; base=0x140000000
while i+4<=len(d):
    ln,kind=struct.unpack_from('<HH',d,i)
    if ln<2: break
    if kind==0x110E:
        flags,o,seg=struct.unpack_from('<IIH',d,i+4)
        e=d.index(b'\0',i+14); name=d[i+14:e].decode('utf8','replace')
        if 1<=seg<=len(secs): out.append((base+secs[seg-1]+o,name))
    i+=2+ln
out.sort()
with open(sys.argv[2],'w') as w:
    for va,n in out: w.write(f"{va:016x} {n}\n")
print(len(out))
