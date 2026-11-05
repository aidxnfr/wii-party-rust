import sys,bisect,collections,re,subprocess
samples,syms=sys.argv[1],sys.argv[2]
def load(path):
    A=[];N=[]
    for l in open(path):
        a,n=l.split(' ',1); A.append(int(a,16)); N.append(n.strip())
    return A,N
addrs,names=load(syms)
haddrs,hnames=load(sys.argv[3]) if len(sys.argv)>3 else ([],[])
mods=[];threads={};hist=collections.defaultdict(collections.Counter)
for l in open(samples):
    p=l.split()
    if p[0]=='MOD': mods.append((int(p[1],16),int(p[2],16),' '.join(p[3:])))
    elif p[0]=='THREAD': threads[p[1]]=(int(p[3]),int(p[5]))
    elif p[0]=='R': hist[p[1]][int(p[2],16)]+=int(p[3])
game=[m for m in mods if m[2].lower()=='wiiparty_game.dll'][0]
def sym(r):
    if game[0]<=r<game[0]+game[1]:
        v=r-game[0]+0x180000000; i=bisect.bisect_right(addrs,v)-1; return names[i] if i>=0 else '?'
    for b,s,n in mods:
        if b<=r<b+s:
            if haddrs and n.lower()=='wii-party-hybrid.exe':
                v=r-b+0x140000000; i=bisect.bisect_right(haddrs,v)-1
                if i>=0: return 'host:'+hnames[i]
            return '['+n+']'
    return '[unknown]'
def cat(n):
    if n.startswith('['): return n
    if n.startswith('host:'):
        for k in ['bevy','wgpu','naga','wprt_gx','winit','std','core','alloc','hashbrown','gpu_alloc','d3d12','windows']:
            if k in n: return 'host-'+k
        return 'host-other'
    if re.match(r'^(_ZN.*)?f_[0-9a-f]{8}',n) or re.search(r'\bf_[0-9a-f]{8}',n) or 'wpgame' in n: return 'guest-code'
    for k in ['wprt_gx','wprt_hw','wprt','wpgame','bevy','wgpu','naga','dsp','DSP','gx','std','core','alloc','hashbrown']:
        if k in n: return k
    return 'other'
busy=sorted(threads.items(),key=lambda x:-x[1][1])[:9]
for tid,(ns,user) in busy:
    tot=sum(hist[tid].values())
    if tot==0: continue
    c=collections.Counter(); cc=collections.Counter()
    for r,k in hist[tid].items(): s=sym(r); c[s]+=k; cc[cat(s)]+=k
    print(f'== thread {tid} samples {tot} user_ms {user}')
    print('  by category:', ', '.join(f'{k} {100*v/tot:.1f}%' for k,v in cc.most_common(12)))
    for s,k in c.most_common(45): print(f'  {100*k/tot:5.1f}%  {s[:150]}')
