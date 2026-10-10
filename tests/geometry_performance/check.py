#!/usr/bin/env python3
"""Reproduce ABC478 G; no external judge submissions. Fixed seeds and exact brute oracle."""
import argparse,json,math,random,platform,subprocess,tempfile,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent

def build(out,name,library,driver):
    source=out/(name+'.rs')
    source.write_text('#![allow(dead_code)]\n'+library+'\n'+driver)
    subprocess.run(['rustc','--edition=2021','-O',str(source),'-o',str(out/name)],check=True)
    return out/name

def libraries(baseline):
    if baseline:return 'include!('+json.dumps(str(HERE/'baseline.rs'))+');'
    return '\n'.join('include!('+json.dumps(str(ROOT/p))+');' for p in ['src/NumberTheory/ratio.rs','src/Gemetory/geometry.rs','src/Gemetory/convexhull.rs'])

def case(points,p=5,q=7):return f'{len(points)} {p} {q}\n'+''.join(f'{x} {y}\n' for x,y in points)
def run(binary,data):
    t=time.perf_counter()
    result=subprocess.run([str(binary)],input=data,text=True,capture_output=True,check=True,timeout=45)
    return result.stdout.strip(),time.perf_counter()-t

def brute(points,p,q):
    ps=sorted(set((q*x+p*u,q*y+p*v) for i,(x,y) in enumerate(points) for u,v in points[i+1:]))
    def cross(a,b,c):return (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
    if len(ps)<3:return '0 1'
    def chain(ps):
        h=[]
        for point in ps:
            while len(h)>1 and cross(h[-2],h[-1],point)<=0:h.pop()
            h.append(point)
        return h
    h=chain(ps)[:-1]+chain(ps[::-1])[:-1]
    area=abs(sum(a[0]*b[1]-a[1]*b[0] for a,b in zip(h,h[1:]+h[:1])))
    den=2*(p+q)**2;g=math.gcd(area,den)
    return f'{area//g} {den//g}'

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--n',type=int,default=100000);ap.add_argument('--small',type=int,default=200);ap.add_argument('--repeat',type=int,default=1);ap.add_argument('--output',type=Path,default=HERE/'results.json');args=ap.parse_args()
    rng=random.Random(478)
    rational=(HERE/'rational_driver.rs').read_text()
    integer=(ROOT/'examples/abc478_g.rs').read_text()
    integer=integer[integer.index('// (元の点群'):]
    with tempfile.TemporaryDirectory(prefix='arcaki_geometry_') as folder:
        out=Path(folder)
        variants={
            'submitted_rational':build(out,'submitted_rational',libraries(True),rational),
            'improved_rational':build(out,'improved_rational',libraries(False),rational),
            'integer_previous_library':build(out,'integer_previous_library',libraries(True),integer),
            'integer_improved_library':build(out,'integer_improved_library',libraries(False),integer),
        }
        # Separate the common-denominator benefit from reusing the input hulls.
        rebuilt=integer[integer.index('fn main()'):]
        rebuilt=rebuilt.replace('let (_, hull) = dfs(&points, p, q);','let hull = dfs_rebuild(&points, p, q);')
        rebuild_fn='''fn dfs_rebuild(points:&[Point<i128>],p:i128,q:i128)->ConvexHull<i128>{
if points.len()==1{return ConvexHull::new(&[]);}
let m=points.len()/2;
let left=dfs_rebuild(&points[..m],p,q);let right=dfs_rebuild(&points[m..],p,q);
let cross=ConvexHull::new(&points[..m]).weighted_minkowski_sum(q,&ConvexHull::new(&points[m..]),p);
left.merge(&right).merge(&cross)
}'''
        variants['integer_rebuild_hulls']=build(out,'integer_rebuild_hulls',libraries(False),rebuild_fn+rebuilt)
        samples=[('4 1 2\n-2 4\n1 1\n-3 -2\n4 0\n','59 6'),('3 3 4\n11 -7\n2 8\n-4 18\n','0 1'),('10 5 7\n-3677189 -8603516\n3324282 8371380\n-4445534 -865546\n-1235744 -6761030\n2732080 -9194939\n8510740 4384715\n-7326815 2451526\n-2079912 1054509\n2072791 -8408897\n-4437202 -5184973\n','6746232144385823 48')]
        for data,expected in samples:
            for binary in variants.values():assert run(binary,data)[0]==expected
        for _ in range(args.small):
            points=[(rng.randrange(-30,31),rng.randrange(-30,31)) for _ in range(rng.randrange(2,26))]
            p=rng.randrange(1,10);q=rng.randrange(p+1,11)
            data=case(points,p,q);expected=brute(points,p,q)
            for binary in variants.values():assert run(binary,data)[0]==expected
        print(f'Validated 3 samples and {args.small} brute cases across {len(variants)} variants',flush=True)
        n=args.n
        circle=[(round(10**7*math.cos(2*math.pi*i/n)),round(10**7*math.sin(2*math.pi*i/n))) for i in range(n)]
        shuffled=circle.copy();rng.shuffle(shuffled)
        shapes={
            'random':[(rng.randrange(-10**7,10**7+1),rng.randrange(-10**7,10**7+1)) for _ in range(n)],
            'circle_ordered':circle,
            'circle_shuffled':shuffled,
            'collinear':[(i-n//2,2*(i-n//2)) for i in range(n)],
            'all_equal':[(10**7,-10**7)]*n,
        }
        results={'n':n,'small_cases':args.small,'repeat':args.repeat,'profiles':'rustc --edition=2021 -O','environment':{'platform':platform.platform(),'machine':platform.machine(),'rustc':subprocess.run(['rustc','--version'],capture_output=True,text=True,check=True).stdout.strip()},'cases':{}}
        for label,points in shapes.items():
            data=case(points);expected=None;result={}
            for name,binary in variants.items():
                elapsed=[]
                for _ in range(args.repeat):
                    answer,t=run(binary,data);elapsed.append(t)
                    if expected is None:expected=answer
                    assert answer==expected,(label,name,answer,expected)
                result[name]={'seconds':elapsed,'answer':answer}
                print(label,name,','.join(f'{t:.4f}s' for t in elapsed),answer,flush=True)
            results['cases'][label]=result
            args.output.write_text(json.dumps(results,ensure_ascii=False,indent=2)+'\n')
        print('Saved',args.output,flush=True)
if __name__=='__main__':main()
